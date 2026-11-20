//! The seat around the harness (Story 1.5, FR-43): one line per `seat` item knowledge
//! states, read as far as the item's read rule allows and no further. Presence stats a
//! path, a listing names one directory's entries, a names read walks listed key paths and
//! keeps key names, an env read asks whether named variables are set, and a keys-present
//! read asks whether top-level keys exist. No value is kept, and nothing is read at or
//! under a credential key.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::{fs, io};

use serde_json::Value;
use usher_core::{LaunchContext, SeatLine, SystemRoot};
use usher_knowledge::{Item, KnowledgeFile, Platform, SeatFact, SeatPart, SeatRead, Version};

use crate::HARNESS;
use crate::paths::resolve;

/// The two `.mcp.json` items. Claude Code's pages do not say which directory it loads the
/// file from after a launch in a subdirectory, so both are listed and, when both exist and
/// differ, each says so.
const MCP_LAUNCH_DIR_ITEM: &str = "claude-code.seat-mcp-launch-dir";
const MCP_REPO_ROOT_ITEM: &str = "claude-code.seat-mcp-repo-root";
const MCP_BLIND_SPOT: &str =
    "which of the two Claude Code loads from a subdirectory launch is not stated";

/// Reads every seat item that applies to `installed`, in part order then item id.
pub(crate) fn read_seat(
    knowledge: &KnowledgeFile,
    context: &LaunchContext,
    system_root: &SystemRoot,
    platform: Platform,
    installed: Option<&Version>,
    variable: &str,
) -> Vec<SeatLine> {
    let Some(installed) = installed else {
        return Vec::new();
    };
    let mut items: Vec<(&Item, &SeatFact)> = knowledge
        .items_for(HARNESS, installed)
        .filter_map(|item| item.seat().map(|fact| (item, fact)))
        .filter(|(_, fact)| fact.platforms().is_none_or(|list| list.contains(&platform)))
        .collect();
    items.sort_by_key(|(item, fact)| (part_order(fact.part()), item.id().to_string()));

    let mut lines: Vec<SeatLine> = Vec::new();
    for (item, fact) in items {
        let Some(pattern) = fact.path() else {
            lines.push(read_env(item, fact, context));
            continue;
        };
        if pattern.contains("${REPO}") && context.repo.is_none() {
            continue;
        }
        let path = match resolve(pattern, context, system_root, variable) {
            Ok(path) => path,
            Err(reason) => {
                lines.push(SeatLine {
                    part: part_name(fact.part()).to_string(),
                    item_id: item.id().to_string(),
                    read: read_name(fact.read()).to_string(),
                    path: None,
                    present: None,
                    entries: Vec::new(),
                    names: Vec::new(),
                    members: Vec::new(),
                    note: Some(reason),
                });
                continue;
            }
        };
        // One file is one line: a later item of the same part and rule that places at the
        // same path adds none.
        if lines.iter().any(|line| {
            line.part == part_name(fact.part())
                && line.read == read_name(fact.read())
                && line.path.as_deref() == Some(path.as_path())
        }) {
            continue;
        }
        lines.push(read_path(knowledge, item, fact, path));
    }
    stamp_mcp_blind_spot(&mut lines);
    lines
}

fn base(item: &Item, fact: &SeatFact, path: Option<PathBuf>) -> SeatLine {
    SeatLine {
        part: part_name(fact.part()).to_string(),
        item_id: item.id().to_string(),
        read: read_name(fact.read()).to_string(),
        path,
        present: None,
        entries: Vec::new(),
        names: Vec::new(),
        members: Vec::new(),
        note: None,
    }
}

/// Which listed variables are set. A variable counts when the context says it is set and
/// non-empty; no value is ever held.
fn read_env(item: &Item, fact: &SeatFact, context: &LaunchContext) -> SeatLine {
    let mut line = base(item, fact, None);
    line.members = fact
        .variables()
        .iter()
        .map(|name| (name.clone(), context.set_variables.contains(name)))
        .collect();
    line
}

fn read_path(knowledge: &KnowledgeFile, item: &Item, fact: &SeatFact, path: PathBuf) -> SeatLine {
    let mut line = base(item, fact, Some(path.clone()));
    let meta = match fs::metadata(&path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            line.present = Some(false);
            if fact.read() == SeatRead::KeysPresent {
                line.members = fact.keys().iter().map(|key| (key.clone(), false)).collect();
            }
            return line;
        }
        Err(error) => {
            line.note = Some(format!("could not be checked: {:?}", error.kind()));
            return line;
        }
    };
    line.present = Some(true);
    match fact.read() {
        SeatRead::Listing => {
            if let Ok(entries) = fs::read_dir(&path) {
                let mut names: Vec<String> = entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect();
                names.sort();
                line.entries = names;
            }
        }
        SeatRead::Names => {
            if let Some(Value::Object(root)) = parse(&path, &meta) {
                let root = Value::Object(root);
                let mut found: BTreeSet<Vec<String>> = BTreeSet::new();
                for key_path in fact.key_paths() {
                    let segments: Vec<&str> = key_path.split('.').collect();
                    walk(knowledge, &root, &segments, &[], &mut found);
                }
                line.names = found.into_iter().collect();
            }
        }
        SeatRead::KeysPresent => {
            let top = match parse(&path, &meta) {
                Some(Value::Object(top)) => Some(top),
                _ => None,
            };
            line.members = fact
                .keys()
                .iter()
                .map(|key| {
                    let has = top.as_ref().is_some_and(|object| object.contains_key(key));
                    (key.clone(), has)
                })
                .collect();
        }
        _ => {}
    }
    line
}

/// Parses a regular file as JSON in memory. Nothing of its content leaves the caller.
fn parse(path: &PathBuf, meta: &fs::Metadata) -> Option<Value> {
    if !meta.is_file() {
        return None;
    }
    let file = fs::File::open(path).ok()?;
    serde_json::from_reader(io::BufReader::new(file)).ok()
}

/// Collects the key names a key path reaches. A wildcard stands for one whole level: each
/// key it matches is listed by name, and the walk goes on under it unless the key is a
/// credential key, which is listed and never entered. At the end of the path, the keys of
/// the object found there are listed; a trailing wildcard lists the keys of its parent.
fn walk(
    knowledge: &KnowledgeFile,
    value: &Value,
    rest: &[&str],
    prefix: &[String],
    out: &mut BTreeSet<Vec<String>>,
) {
    let Value::Object(object) = value else {
        return;
    };
    let join = |key: &str| -> Vec<String> {
        let mut path = prefix.to_vec();
        path.push(key.to_string());
        path
    };
    match rest {
        [] | ["*"] => {
            for key in object.keys() {
                out.insert(join(key));
            }
        }
        ["*", tail @ ..] => {
            for (key, child) in object {
                out.insert(join(key));
                if !KnowledgeFile::is_credential_key(key) {
                    walk(knowledge, child, tail, &join(key), out);
                }
            }
        }
        [literal, tail @ ..] => {
            if let Some(child) = object.get(*literal) {
                walk(knowledge, child, tail, &join(literal), out);
            }
        }
    }
}

/// When both `.mcp.json` lines exist at different paths, each says which question is open.
fn stamp_mcp_blind_spot(lines: &mut [SeatLine]) {
    let pair: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| {
            line.item_id == MCP_LAUNCH_DIR_ITEM || line.item_id == MCP_REPO_ROOT_ITEM
        })
        .map(|(i, _)| i)
        .collect();
    if pair.len() == 2 && pair.iter().all(|&i| lines[i].present == Some(true)) {
        for i in pair {
            lines[i].note = Some(MCP_BLIND_SPOT.to_string());
        }
    }
}

fn part_name(part: SeatPart) -> &'static str {
    match part {
        SeatPart::Mcp => "mcp",
        SeatPart::Packs => "packs",
        SeatPart::FleetClient => "fleet-client",
        SeatPart::AuthRoute => "auth-route",
        SeatPart::Toolchain => "toolchain",
        _ => "other",
    }
}

/// The declaration order of the parts, which is the order the lines print in.
fn part_order(part: SeatPart) -> u8 {
    match part {
        SeatPart::Mcp => 0,
        SeatPart::Packs => 1,
        SeatPart::FleetClient => 2,
        SeatPart::AuthRoute => 3,
        SeatPart::Toolchain => 4,
        _ => 5,
    }
}

fn read_name(read: SeatRead) -> &'static str {
    match read {
        SeatRead::Presence => "presence",
        SeatRead::Listing => "listing",
        SeatRead::Names => "names",
        SeatRead::Env => "env",
        SeatRead::KeysPresent => "keys-present",
        _ => "other",
    }
}
