//! The `inventory` verb: reads the environment, asks the adapter for the sources,
//! records the run in the store and prints what it found.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use usher_adapter_claude_code::{ClaudeCodeAdapter, VersionReading, read_version};
use usher_core::{
    KeyNames, LaunchContext, ListedSource, SeatLine, SourceLister, SourceListing, SourceStatus,
    SystemRoot, UsherError, environment_key, key_variables,
};
use usher_knowledge::{Knowledge, KnowledgeFile, Version};
use usher_store::{RunRecord, SeatRecord, SourceRecord, Store};

use crate::time::rfc3339;
use crate::{InventoryEnv, LaunchEnvironment, inventory_platform};

const HARNESS: &str = "claude-code";
const CONFIG_DIR_TOKEN: &str = "${CONFIG_DIR}";
const GLOBAL_STATE_TOKEN: &str = "${GLOBAL_STATE_DIR}";

/// Runs `usher inventory` against `env`, placing absolute knowledge paths under `root`,
/// and returns the text to print. Only tests pass a root other than the host's.
pub fn inventory(env: &InventoryEnv, root: SystemRoot) -> Result<String, UsherError> {
    inventory_with(env, root, &Knowledge::embedded()?)
}

/// As [`inventory`], over a given knowledge, for tests that need a fact the seed does not
/// hold. A [`Knowledge`] is the embedded file or a signed update, so no unsigned
/// knowledge reaches the verb.
pub fn inventory_with(
    env: &InventoryEnv,
    root: SystemRoot,
    knowledge: &Knowledge,
) -> Result<String, UsherError> {
    let file = knowledge.current();

    // One list of names feeds the key: the variables knowledge names for the harness
    // across every version range, kept only where set and non-empty.
    let names = file.launch_variables(HARNESS);
    let kept = key_variables(&names, |name| env.launch.get(name).map(str::to_string));
    let launch: Vec<(&str, Option<&str>)> = kept
        .iter()
        .map(|(name, value)| (name.as_str(), Some(value.as_str())))
        .collect();
    let key = environment_key(&env.host, &env.user, HARNESS, &env.launch_dir, &launch)?;
    let key_text = key.to_string();

    let mut context = LaunchContext {
        key,
        home: env.home.clone(),
        config_dir: config_dir(file, None, env),
        global_state_dir: global_state_dir(file, None, env),
        set_variables: set_variables(file, env),
        repo: find_repo(&env.launch_dir),
        user: env.user.clone(),
        harness_version: None,
    };
    let reading = read_harness_version(file, &context, env);
    if let VersionReading::Known(version) = &reading {
        context.harness_version = Some(version.clone());
        if let Ok(parsed) = Version::parse(version) {
            context.config_dir = config_dir(file, Some(&parsed), env);
            context.global_state_dir = global_state_dir(file, Some(&parsed), env);
        }
    }

    let adapter = ClaudeCodeAdapter::new(file.clone(), root, env.platform);
    let listing = adapter.list_sources(&context)?;

    record(env, file, &listing, &key_text)?;
    Ok(render(file, &listing, &reading))
}

/// The directory the config token takes: the variable knowledge binds to it, or its
/// default while the variable is unset or empty.
fn config_dir(file: &KnowledgeFile, version: Option<&Version>, env: &InventoryEnv) -> PathBuf {
    bound_dir(
        file,
        version,
        env,
        CONFIG_DIR_TOKEN,
        env.home.join(".claude"),
    )
}

/// The directory the global state token takes, by the same rule as the config directory.
fn global_state_dir(
    file: &KnowledgeFile,
    version: Option<&Version>,
    env: &InventoryEnv,
) -> PathBuf {
    bound_dir(file, version, env, GLOBAL_STATE_TOKEN, env.home.clone())
}

/// The variable that moves a directory, and the default the directory takes while it is
/// unset or empty. At a known version the version-scoped binding is used; with none, any
/// binder knowledge lists for the harness. With no binder at all, `fallback` is used.
fn bound_dir(
    file: &KnowledgeFile,
    version: Option<&Version>,
    env: &InventoryEnv,
    token: &str,
    fallback: PathBuf,
) -> PathBuf {
    let binder = version
        .and_then(|v| {
            file.binding(HARNESS, v, token)
                .map(|b| (b.variable().to_string(), b.default().to_string()))
        })
        .or_else(|| {
            file.items()
                .iter()
                .filter(|item| item.harness() == HARNESS)
                .filter_map(|item| item.variable())
                .find(|variable| variable.binds() == Some(token))
                .and_then(|variable| {
                    variable
                        .default()
                        .map(|default| (variable.name().to_string(), default.to_string()))
                })
        });
    let Some((variable, default)) = binder else {
        return fallback;
    };
    match env.launch.get(&variable).filter(|value| !value.is_empty()) {
        Some(value) => PathBuf::from(value),
        None => PathBuf::from(
            default
                .replace("${HOME}", &env.home.to_string_lossy())
                .replace("${USER}", &env.user),
        ),
    }
}

/// The names knowledge lists that are set and non-empty in the launch environment: the
/// variables the seat reads and the ones that bind a path token. Only names, never values.
fn set_variables(file: &KnowledgeFile, env: &InventoryEnv) -> std::collections::BTreeSet<String> {
    let mut listed: Vec<&str> = file.launch_variables(HARNESS);
    for item in file.items().iter().filter(|item| item.harness() == HARNESS) {
        if let Some(seat) = item.seat() {
            listed.extend(seat.variables().iter().map(String::as_str));
        }
    }
    listed
        .into_iter()
        .filter(|name| env.launch.get(name).is_some_and(|value| !value.is_empty()))
        .map(str::to_string)
        .collect()
}

fn read_harness_version(
    file: &KnowledgeFile,
    context: &LaunchContext,
    env: &InventoryEnv,
) -> VersionReading {
    let from = file
        .items()
        .iter()
        .filter(|item| item.harness() == HARNESS)
        .find_map(|item| item.version_from());
    match from {
        Some(from) => read_version(from, context, env.path.as_deref()),
        None => VersionReading::Unknown {
            reason: "knowledge names no way to read the version".to_string(),
        },
    }
}

/// The root of the repository that holds `dir`: the nearest ancestor with a `.git`.
fn find_repo(dir: &Path) -> Option<PathBuf> {
    dir.ancestors()
        .find(|candidate| candidate.join(".git").exists())
        .map(Path::to_path_buf)
}

fn record(
    env: &InventoryEnv,
    file: &KnowledgeFile,
    listing: &SourceListing,
    key: &str,
) -> Result<(), UsherError> {
    struct Row {
        item_id: String,
        path: String,
        layer: String,
        present: bool,
        size: Option<u64>,
        modified: Option<String>,
        names: Vec<String>,
    }
    let mut rows: Vec<Row> = listing
        .sources
        .iter()
        .map(|source| Row {
            item_id: source.item_id.clone(),
            path: source.path.to_string_lossy().into_owned(),
            layer: source.layer.clone(),
            present: true,
            size: source.size,
            modified: source.modified.map(rfc3339),
            names: match &source.keys {
                KeyNames::Names { names, .. } => names.clone(),
                _ => Vec::new(),
            },
        })
        .collect();
    for absent in &listing.absent_layers {
        for checked in &absent.checked {
            rows.push(Row {
                item_id: checked.item_id.clone(),
                path: checked.path.to_string_lossy().into_owned(),
                layer: absent.layer.clone(),
                present: false,
                size: None,
                modified: None,
                names: Vec::new(),
            });
        }
    }
    let name_refs: Vec<Vec<&str>> = rows
        .iter()
        .map(|row| row.names.iter().map(String::as_str).collect())
        .collect();
    let sources: Vec<SourceRecord<'_>> = rows
        .iter()
        .zip(&name_refs)
        .map(|(row, names)| SourceRecord {
            item_id: &row.item_id,
            path: &row.path,
            layer: &row.layer,
            present: row.present,
            size: row.size,
            modified: row.modified.as_deref(),
            key_names: names,
        })
        .collect();
    let version = file.version();
    let run = RunRecord {
        recorded_at: &env.now,
        harness: HARNESS,
        harness_version: listing.harness_version.as_deref(),
        knowledge_schema: version.schema_version(),
        knowledge_sequence: version.sequence(),
        environment_key: key,
    };
    let mut store = Store::open(&env.state_dir)?;
    let seat_names: Vec<Vec<Vec<String>>> = listing
        .seat
        .iter()
        .map(|line| match line.read.as_str() {
            "listing" => line.entries.iter().map(|e| vec![e.clone()]).collect(),
            _ => line.names.clone(),
        })
        .collect();
    let seat: Vec<SeatRecord<'_>> = listing
        .seat
        .iter()
        .zip(&seat_names)
        .map(|(line, names)| SeatRecord {
            part: &line.part,
            item_id: &line.item_id,
            present: line.present,
            names,
            members: &line.members,
            note: line.note.as_deref(),
        })
        .collect();
    store.record_inventory_with_seat(&run, &sources, &seat)?;
    Ok(())
}

fn render(file: &KnowledgeFile, listing: &SourceListing, reading: &VersionReading) -> String {
    let knowledge = &listing.knowledge_version;
    let covered = listing
        .harness_version
        .as_deref()
        .and_then(|text| Version::parse(text).ok())
        .is_some_and(|version| file.items_for(HARNESS, &version).next().is_some());
    let mut out = String::new();
    let _ = match (&listing.harness_version, covered) {
        (Some(version), true) => writeln!(
            out,
            "Claude Code {version}, {knowledge}: every status below assumes these"
        ),
        (Some(version), false) => writeln!(
            out,
            "Claude Code {version}, knowledge for it not known ({knowledge} loaded): every status below reads unknown"
        ),
        (None, _) => {
            let why = match reading {
                VersionReading::Unknown { reason } => reason.as_str(),
                VersionReading::Known(_) => "not read",
            };
            writeln!(
                out,
                "Claude Code version unknown ({why}), knowledge for it not known ({knowledge} loaded): every status below reads unknown"
            )
        }
    };
    out.push('\n');
    for source in &listing.sources {
        let _ = writeln!(out, "{}", source_row(source));
        let _ = writeln!(out, "    {}", keys_line(&source.keys));
    }
    for absent in &listing.absent_layers {
        for checked in &absent.checked {
            let _ = writeln!(
                out,
                "no file: {} ({})",
                layer_label(&absent.layer),
                checked.path.display()
            );
        }
    }
    for layer in &listing.not_checked {
        let _ = writeln!(
            out,
            "not checked: {} ({})",
            layer_label(&layer.layer),
            layer.reason
        );
    }
    if !listing.seat.is_empty() {
        out.push('\n');
        for line in &listing.seat {
            let _ = writeln!(out, "{}", seat_row(line));
        }
    }
    out.trim_end().to_string()
}

/// One seat line: the part, where it looked, what it found as far as the read rule allows,
/// and the knowledge item behind it. Never a value.
fn seat_row(line: &SeatLine) -> String {
    let tail = format!("(knowledge item {})", line.item_id);
    let part = &line.part;
    if line.read == "env" {
        let vars: Vec<String> = line
            .members
            .iter()
            .map(|(name, set)| format!("{name} {}", if *set { "present" } else { "absent" }))
            .collect();
        return format!("{part}  env  {} {tail}", vars.join(", "));
    }
    let Some(path) = &line.path else {
        let why = line.note.as_deref().unwrap_or("not checked");
        return format!("{part}  not checked ({why}) {tail}");
    };
    let path = path.display();
    let state = match line.present {
        None => {
            let why = line.note.as_deref().unwrap_or("not checked");
            return format!("{part}  {path}  not checked ({why}) {tail}");
        }
        Some(false) => "absent".to_string(),
        Some(true) => match line.read.as_str() {
            "keys-present" => line
                .members
                .iter()
                .map(|(name, has)| format!("{name} {}", if *has { "present" } else { "absent" }))
                .collect::<Vec<_>>()
                .join(", "),
            "listing" => detail("present", "entries", &line.entries.join(", "), line),
            "names" => {
                let names: Vec<String> = line.names.iter().map(|n| dotted(n)).collect();
                detail("present", "names", &names.join(", "), line)
            }
            _ => detail("present", "", "", line),
        },
    };
    format!("{part}  {path}  {state} {tail}")
}

/// `present`, then the names found and the note, in one parenthesis: names first.
fn detail(base: &str, label: &str, found: &str, line: &SeatLine) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !found.is_empty() {
        parts.push(format!("{label}: {found}"));
    }
    if let Some(note) = &line.note {
        parts.push(note.clone());
    }
    if parts.is_empty() {
        base.to_string()
    } else {
        format!("{base} ({})", parts.join("; "))
    }
}

/// A path of segments joined by dots. A segment that holds a dot, whitespace or a quote
/// is printed as a JSON string, so `a.b` stays one segment.
fn dotted(segments: &[String]) -> String {
    segments
        .iter()
        .map(|segment| {
            if segment.contains(['.', '"']) || segment.chars().any(char::is_whitespace) {
                format!("{segment:?}")
            } else {
                segment.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// How a layer reads in a line of output. A presentation table, not a harness fact.
fn layer_label(layer: &str) -> String {
    match layer {
        "global-state" => "global state".to_string(),
        "server-managed-cache" => "server-managed cache".to_string(),
        other => format!("{other} settings"),
    }
}

fn source_row(source: &ListedSource) -> String {
    format!(
        "{}  {}  {}  {}",
        source.layer,
        source.harness,
        source.path.display(),
        status_text(&source.status)
    )
}

fn status_text(status: &SourceStatus) -> String {
    match status {
        SourceStatus::InEffect => "in effect".to_string(),
        SourceStatus::Shadowed { by } => format!("shadowed by {}", by.display()),
        SourceStatus::Unread { reason } => format!("unread ({reason})"),
        SourceStatus::Unknown { reason } => format!("unknown ({reason})"),
        _ => "unknown (a status this build cannot print)".to_string(),
    }
}

fn keys_line(keys: &KeyNames) -> String {
    match keys {
        KeyNames::Names {
            names,
            classification,
        } => {
            let list = if names.is_empty() {
                "none".to_string()
            } else {
                names.join(", ")
            };
            format!("keys: {list} (names only; {classification})")
        }
        KeyNames::NotRead { reason } => format!("keys: not read ({reason})"),
        _ => "keys: not read (a form this build cannot print)".to_string(),
    }
}

/// Reads this process's environment for the verb.
pub(crate) fn env_from_process() -> Result<InventoryEnv, UsherError> {
    let knowledge = Knowledge::embedded()?;
    let names = knowledge.current().launch_variables(HARNESS);
    let home = usher_store::account_home()?;
    let platform = inventory_platform();
    let store_platform = match platform {
        usher_knowledge::Platform::Macos => usher_store::Platform::Macos,
        _ => usher_store::Platform::Linux,
    };
    let launch_dir = std::env::current_dir().map_err(|why| {
        UsherError::new(
            "read the launch directory",
            "the current directory can be read",
            why.to_string(),
            "run `usher inventory` from a directory that exists",
        )
    })?;
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    Ok(InventoryEnv {
        host: host_name()?,
        user: usher_store::account_name()?,
        state_dir: usher_store::state_dir_under(&home, store_platform),
        home,
        launch_dir,
        launch: launch_from_process(knowledge.current(), &names)?,
        path: std::env::var_os("PATH"),
        now: rfc3339(seconds),
        platform,
    })
}

/// The launch environment of this process: the variables that bind a path token keep their
/// values, and a variable only the seat reads is recorded as set, never read.
fn launch_from_process(
    file: &KnowledgeFile,
    binders: &[&str],
) -> Result<LaunchEnvironment, UsherError> {
    let mut launch = LaunchEnvironment::from_process(binders)?;
    for item in file.items().iter().filter(|item| item.harness() == HARNESS) {
        let Some(seat) = item.seat() else { continue };
        for name in seat.variables() {
            if binders.contains(&name.as_str()) {
                continue;
            }
            if std::env::var_os(name).is_some_and(|value| !value.is_empty()) {
                launch.mark_set(name);
            }
        }
    }
    Ok(launch)
}

fn host_name() -> Result<String, UsherError> {
    let fail = |status: String| {
        UsherError::new(
            "name this host",
            "the system host name can be read",
            status,
            "fix the system host name so it can be read, then run `usher inventory` again",
        )
    };
    let raw = nix::unistd::gethostname().map_err(|why| fail(why.to_string()))?;
    let name = raw.to_string_lossy().trim().to_string();
    if name.is_empty() {
        Err(fail("the system host name is empty".to_string()))
    } else {
        Ok(name)
    }
}
