//! The `map` verb: prints which value is in effect for each setting, from which layer and
//! why, and records the run.
//!
//! The first block is the text `usher inventory` prints for the same environment, byte
//! for byte. The settings block follows it, one line per reading, then what the map does
//! not observe. The run is recorded once, with its readings, through the store's map
//! record.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use usher_core::{
    NotObserved, SettingReading, SettingsMap, SettingsMapper, SystemRoot, UsherError,
};
use usher_knowledge::Knowledge;
use usher_store::SettingRecord;

use crate::InventoryEnv;
use crate::inventory::{HARNESS, observe, record, render};

const MAP_NEXT: &str = "run `usher map <directory>` with a directory that exists";

/// Runs `usher map <dir>` against `env`, placing absolute knowledge paths under `root`,
/// and returns the text to print. Only tests pass a root other than the host's.
pub fn map(env: &InventoryEnv, dir: &Path, root: SystemRoot) -> Result<String, UsherError> {
    map_with(env, dir, root, &Knowledge::embedded()?)
}

/// As [`map`], over a given knowledge, for tests that need a fact the seed does not hold.
pub fn map_with(
    env: &InventoryEnv,
    dir: &Path,
    root: SystemRoot,
    knowledge: &Knowledge,
) -> Result<String, UsherError> {
    if !dir.is_dir() {
        let why = if dir.exists() {
            "is not a directory"
        } else {
            "does not exist"
        };
        return Err(UsherError::new(
            "map the settings of a directory",
            "the directory exists",
            format!("{} {why}", dir.display()),
            MAP_NEXT,
        ));
    }
    // The directory is the launch directory as given: it keys the run and decides the
    // project layers, and a symlink is not resolved for either.
    let mut env = env.clone();
    env.launch_dir = dir.to_path_buf();
    let observed = observe(&env, root, knowledge)?;
    let map = observed
        .adapter
        .map_settings(&observed.listing, &observed.context)?;
    let launch_dir = dir.to_string_lossy();
    let keys: Vec<String> = map
        .readings
        .iter()
        .map(|reading| segments_text(&segments(&reading.key)))
        .collect();
    let values: Vec<Option<String>> = map
        .readings
        .iter()
        .map(|reading| reading.value.as_ref().map(Value::to_string))
        .collect();
    let paths: Vec<String> = map
        .readings
        .iter()
        .map(|reading| reading.source_path.to_string_lossy().into_owned())
        .collect();
    let records: Vec<SettingRecord<'_>> = map
        .readings
        .iter()
        .enumerate()
        .map(|(i, reading)| SettingRecord {
            harness: HARNESS,
            launch_dir: &launch_dir,
            key: &keys[i],
            layer: &reading.layer,
            source_path: &paths[i],
            method: reading.method.token(),
            status: reading.status.token(),
            entry_index: reading.entry_index,
            value_text: values[i].as_deref(),
            note: reading.note.as_deref(),
        })
        .collect();
    record(
        &env,
        knowledge.current(),
        &observed.listing,
        &observed.key_text,
        Some(&records),
    )?;

    let mut out = render(knowledge.current(), &observed.listing, &observed.reading);
    out.push_str("\n\n");
    if let Ok(target) = std::fs::canonicalize(dir)
        && target != dir
    {
        let _ = writeln!(
            out,
            "settings of {} (the directory resolves to {})",
            dir.display(),
            target.display()
        );
    }
    out.push_str(&settings_block(&map));
    Ok(out.trim_end().to_string())
}

/// The readings, the layers that took no part, then what the map does not observe.
fn settings_block(map: &SettingsMap) -> String {
    let mut out = String::new();
    for reading in &map.readings {
        let _ = writeln!(out, "{}", row(reading));
    }
    for skipped in &map.skipped_layers {
        let _ = writeln!(out, "not used: {} ({})", skipped.layer, skipped.reason);
    }
    out.push('\n');
    for item in &map.not_observed {
        let _ = writeln!(out, "{}", not_observed_text(item));
    }
    out
}

/// One reading on one line: the key path (with `[index]` on a list entry), the status
/// words, the value when there is one, `(<layer>: <file>)`, then `; <note>` when there is
/// a note. Fields are two spaces apart.
fn row(reading: &SettingReading) -> String {
    let mut line = reading.key.clone();
    if let Some(index) = reading.entry_index {
        let _ = write!(line, "[{index}]");
    }
    let _ = write!(line, "  {}", reading.status.display());
    if let Some(value) = &reading.value {
        match value {
            Value::String(text) => {
                let _ = write!(line, "  {text}");
            }
            other => {
                let _ = write!(line, "  {other}");
            }
        }
    }
    let _ = write!(
        line,
        "  ({}: {})",
        reading.layer,
        reading.source_path.display()
    );
    if let Some(note) = &reading.note {
        let _ = write!(line, "; {note}");
    }
    line
}

fn not_observed_text(item: &NotObserved) -> String {
    match item {
        NotObserved::CommandLine => {
            "command-line settings (--settings, flags) at launch are not observable".to_string()
        }
        NotObserved::EnvValues => "settings `env` values are not read".to_string(),
        NotObserved::ServerManagedValues => {
            "managed values held server-side may differ from the cache".to_string()
        }
        NotObserved::OrganizationDefaultModel => "an organization default model is not observed; if an admin set one, it applies over `model` in user, project and local settings".to_string(),
        NotObserved::BuiltInClassifierRules => "built-in classifier rules are not observed; an array without \"$defaults\" may replace them".to_string(),
        NotObserved::LayerMissingFromStack(name) => {
            format!("{name} is not a layer of the settings stack, so its file gives no map rows")
        }
        _ => "something else is not observed (a form this build cannot print)".to_string(),
    }
}

/// The segments of a dotted key path as the reader writes it: a segment that holds a dot,
/// a quote or whitespace is a JSON string, so splitting is unambiguous.
fn segments(key: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = key;
    while !rest.is_empty() {
        if rest.starts_with('"') {
            let mut end = 1;
            let bytes = rest.as_bytes();
            while end < bytes.len() {
                match bytes[end] {
                    b'\\' => end += 2,
                    b'"' => break,
                    _ => end += 1,
                }
            }
            let quoted = rest.get(..=end).unwrap_or(rest);
            let text =
                serde_json::from_str::<String>(quoted).unwrap_or_else(|_| quoted.to_string());
            out.push(text);
            rest = rest.get(end + 1..).unwrap_or("");
        } else {
            let end = rest.find('.').unwrap_or(rest.len());
            out.push(rest[..end].to_string());
            rest = &rest[end..];
        }
        rest = rest.strip_prefix('.').unwrap_or(rest);
    }
    out
}

/// The stored form of a key path: the JSON array of its segments.
fn segments_text(segments: &[String]) -> String {
    Value::from(segments.to_vec()).to_string()
}
