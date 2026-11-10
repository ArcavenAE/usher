//! The Claude Code adapter, inventory only (Story 1.5, FR-43).
//!
//! It lists every Claude Code settings source on a host: where harness
//! knowledge says one may be, whether or not anything reads it. It reads
//! derived facts only: presence, path, size, modified time and the top-level
//! key names of a JSON source. A value is parsed in memory to reach the key
//! names and dropped; nothing is written, no `claude` command is run, and no
//! symlink below a source root is followed (AD-8, AD-9).
//!
//! Locations and the setting names usher recognizes come from harness
//! knowledge, not from this crate (AD-6).

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use usher_core::UsherError;
use usher_knowledge::{KnowledgeFile, KnowledgeVersion, Location, Platform, Scope, Shape, Version};

/// The Claude Code version the knowledge was checked against. Inventory does
/// not run `claude`, so the version on the host is not detected.
pub const ASSUMED_VERSION: &str = "2.1.317";

/// The harness name in harness knowledge.
const HARNESS: &str = "claude-code";

/// The status every found source has until environments are mapped (Stories 1.9 and 1.14).
const NO_ENVIRONMENT: &str = "no environment is mapped";

/// A source file larger than this is listed but its keys are not read.
const MAX_READ_BYTES: u64 = 16 * 1024 * 1024;

const STEP: &str = "inventory Claude Code settings";

/// Where to look: the home, the config-dir override, a system root and project roots.
#[derive(Debug, Clone)]
pub struct Environment {
    home: PathBuf,
    config_dir: Option<PathBuf>,
    system_root: PathBuf,
    platform: Platform,
    roots: Vec<PathBuf>,
}

impl Environment {
    /// An environment for a given home, with the real system root and platform.
    pub fn new(home: PathBuf) -> Self {
        Self {
            home,
            config_dir: None,
            system_root: PathBuf::from("/"),
            platform: host_platform(),
            roots: Vec::new(),
        }
    }

    /// Reads the process environment: `HOME`, `CLAUDE_CONFIG_DIR`, and
    /// `USHER_SYSTEM_ROOT`, a test hook that moves the system directories
    /// (such as `/etc/claude-code`) under another root.
    pub fn from_process(
        get: impl Fn(&str) -> Option<std::ffi::OsString>,
    ) -> Result<Self, UsherError> {
        let home = get("HOME").filter(|h| !h.is_empty()).ok_or_else(|| {
            UsherError::new(
                STEP,
                "the HOME variable is set",
                "HOME is not set",
                "set HOME to the home directory whose Claude Code settings to list",
            )
        })?;
        let mut env = Self::new(PathBuf::from(home));
        env.config_dir = get("CLAUDE_CONFIG_DIR")
            .filter(|d| !d.is_empty())
            .map(PathBuf::from);
        if let Some(system) = get("USHER_SYSTEM_ROOT").filter(|d| !d.is_empty()) {
            env.system_root = PathBuf::from(system);
        }
        Ok(env)
    }

    /// Looks for project and local files under `root` as well.
    #[must_use]
    pub fn with_root(mut self, root: PathBuf) -> Self {
        if !self.roots.contains(&root) {
            self.roots.push(root);
        }
        self
    }

    /// Moves the system directories under `root` (for scratch homes).
    #[must_use]
    pub fn with_system_root(mut self, root: PathBuf) -> Self {
        self.system_root = root;
        self
    }

    /// Overrides the platform whose managed directories are looked at.
    #[must_use]
    pub fn with_platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    /// Sets the `CLAUDE_CONFIG_DIR` override.
    #[must_use]
    pub fn with_config_dir(mut self, dir: PathBuf) -> Self {
        self.config_dir = Some(dir);
        self
    }

    fn config_home(&self) -> PathBuf {
        self.config_dir
            .clone()
            .unwrap_or_else(|| self.home.join(".claude"))
    }

    fn global_config_home(&self) -> PathBuf {
        self.config_dir.clone().unwrap_or_else(|| self.home.clone())
    }

    fn system_prefix(&self) -> String {
        let text = self.system_root.to_string_lossy().into_owned();
        text.trim_end_matches('/').to_string()
    }

    fn expand(&self, template: &str, root: Option<&Path>) -> Option<PathBuf> {
        if template.contains("{root}") && root.is_none() {
            return None;
        }
        let root_text = root.map(|r| r.to_string_lossy().into_owned());
        let text = template
            .replace("{home}", &self.home.to_string_lossy())
            .replace("{config_dir}", &self.config_home().to_string_lossy())
            .replace(
                "{global_config_dir}",
                &self.global_config_home().to_string_lossy(),
            )
            .replace("{system}", &self.system_prefix())
            .replace("{root}", root_text.as_deref().unwrap_or(""));
        Some(PathBuf::from(text))
    }
}

fn host_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::Macos
    } else {
        Platform::Linux
    }
}

/// What usher can say about a source's role. Environments are mapped by later
/// stories, so every source found is `Unknown` for now, with the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Status {
    /// Usher cannot tell, and says why.
    Unknown(String),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Unknown(reason) => write!(f, "unknown ({reason})"),
        }
    }
}

/// The key names found in a source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keys {
    /// Top-level key names, split by whether knowledge lists them.
    Names {
        /// Names harness knowledge lists for the assumed version.
        recognized: Vec<String>,
        /// Names it does not list; never dropped.
        unrecognized: Vec<String>,
        /// False when no list applies to this kind of file, so every name is unclassified.
        classified: bool,
    },
    /// The names could not be read, and why.
    NotRead(String),
}

/// One settings source that exists on the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Where it is.
    pub path: PathBuf,
    /// The scope (layer) it would belong to.
    pub scope: Scope,
    /// What usher can say about it.
    pub status: Status,
    /// Size in bytes.
    pub size: u64,
    /// Modified time as seconds since the Unix epoch, when the system gives one.
    pub modified: Option<u64>,
    /// Its top-level key names.
    pub keys: Keys,
}

/// Everything the inventory found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    /// The Claude Code version knowledge was read for.
    pub assumed_version: &'static str,
    /// The version of the knowledge file used.
    pub knowledge: KnowledgeVersion,
    /// Sources that exist, in the order knowledge lists their locations.
    pub sources: Vec<Source>,
    /// Locations looked at where nothing exists.
    pub not_present: Vec<(PathBuf, Scope)>,
    /// Whether project and local locations were looked at (a root was given).
    pub looked_in_roots: bool,
}

/// Lists the Claude Code settings sources in `env`.
pub fn inventory(env: &Environment, knowledge: &KnowledgeFile) -> Result<Inventory, UsherError> {
    let assumed = Version::parse(ASSUMED_VERSION).map_err(|why| {
        UsherError::new(
            STEP,
            "the assumed version parses",
            why,
            "fix ASSUMED_VERSION",
        )
    })?;
    let mut recognized_keys: BTreeSet<&str> = BTreeSet::new();
    let mut locations: Vec<&Location> = Vec::new();
    for item in knowledge.items_for(HARNESS, &assumed) {
        recognized_keys.extend(item.keys().iter().map(String::as_str));
        locations.extend(item.locations());
    }
    if locations.is_empty() {
        return Err(UsherError::new(
            STEP,
            "harness knowledge names Claude Code settings locations",
            format!(
                "{}: no location for Claude Code {ASSUMED_VERSION}",
                knowledge.version()
            ),
            "update harness knowledge, or run a build whose knowledge covers Claude Code",
        ));
    }

    let mut sources: Vec<Source> = Vec::new();
    let mut not_present: Vec<(PathBuf, Scope)> = Vec::new();
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    for location in locations {
        if location.platform() != Platform::Any && location.platform() != env.platform {
            continue;
        }
        let roots: Vec<Option<&Path>> = if location.template().contains("{root}") {
            env.roots.iter().map(|r| Some(r.as_path())).collect()
        } else {
            vec![None]
        };
        for root in roots {
            let Some(path) = env.expand(location.template(), root) else {
                continue;
            };
            if !seen.insert(path.clone()) {
                continue;
            }
            match location.shape() {
                Shape::File => {
                    read_file_source(
                        &path,
                        location.scope(),
                        &recognized_keys,
                        &mut sources,
                        &mut not_present,
                    );
                }
                Shape::JsonDir => {
                    read_dir_sources(
                        &path,
                        location.scope(),
                        &recognized_keys,
                        &mut sources,
                        &mut not_present,
                    );
                }
                _ => {}
            }
        }
    }
    // Highest precedence first, as the vendor documents it; within a scope the
    // order is knowledge's, which puts the cached server copy before the files.
    sources.sort_by_key(|source| precedence(source.scope));
    Ok(Inventory {
        assumed_version: ASSUMED_VERSION,
        knowledge: knowledge.version(),
        sources,
        not_present,
        looked_in_roots: !env.roots.is_empty(),
    })
}

/// Where a scope sits in the vendor's precedence order, highest first. The
/// harness's own state file is not a settings layer and goes last.
fn precedence(scope: Scope) -> u8 {
    match scope {
        Scope::Managed => 0,
        Scope::Local => 1,
        Scope::Project => 2,
        Scope::User => 3,
        _ => 4,
    }
}

fn seconds(meta: &fs::Metadata) -> Option<u64> {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

fn unknown(reason: impl Into<String>) -> Status {
    Status::Unknown(reason.into())
}

fn io_reason(err: &io::Error) -> String {
    match err.kind() {
        io::ErrorKind::PermissionDenied => "permission denied".to_string(),
        other => format!("{other}"),
    }
}

/// Finds one file source. The path itself is followed if it is a symlink;
/// nothing below it is.
fn read_file_source(
    path: &Path,
    scope: Scope,
    recognized: &BTreeSet<&str>,
    sources: &mut Vec<Source>,
    not_present: &mut Vec<(PathBuf, Scope)>,
) {
    match fs::metadata(path) {
        Ok(meta) if meta.is_file() => sources.push(describe(path, scope, &meta, recognized)),
        Ok(_) => sources.push(Source {
            path: path.to_path_buf(),
            scope,
            status: unknown("not a regular file"),
            size: 0,
            modified: None,
            keys: Keys::NotRead("not a regular file".to_string()),
        }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            not_present.push((path.to_path_buf(), scope));
        }
        Err(err) => sources.push(Source {
            path: path.to_path_buf(),
            scope,
            status: unknown(format!("cannot stat: {}", io_reason(&err))),
            size: 0,
            modified: None,
            keys: Keys::NotRead(io_reason(&err)),
        }),
    }
}

/// Finds the visible `*.json` files of a directory, in name order. An entry
/// that is a symlink is listed and not followed.
fn read_dir_sources(
    dir: &Path,
    scope: Scope,
    recognized: &BTreeSet<&str>,
    sources: &mut Vec<Source>,
    not_present: &mut Vec<(PathBuf, Scope)>,
) {
    let listing = match fs::read_dir(dir) {
        Ok(listing) => listing,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            not_present.push((dir.to_path_buf(), scope));
            return;
        }
        Err(err) => {
            sources.push(Source {
                path: dir.to_path_buf(),
                scope,
                status: unknown(format!("cannot list: {}", io_reason(&err))),
                size: 0,
                modified: None,
                keys: Keys::NotRead(io_reason(&err)),
            });
            return;
        }
    };
    let mut names: Vec<std::ffi::OsString> = listing
        .filter_map(Result::ok)
        .map(|entry| entry.file_name())
        .filter(|name| {
            let text = name.to_string_lossy();
            text.ends_with(".json") && !text.starts_with('.')
        })
        .collect();
    names.sort();
    for name in names {
        let path = dir.join(&name);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_symlink() => sources.push(Source {
                path,
                scope,
                status: unknown("a symlink, not followed"),
                size: 0,
                modified: seconds(&meta),
                keys: Keys::NotRead("a symlink, not followed".to_string()),
            }),
            Ok(meta) if meta.is_file() => sources.push(describe(&path, scope, &meta, recognized)),
            _ => {}
        }
    }
}

/// Builds the record for a regular file, reading its key names.
fn describe(path: &Path, scope: Scope, meta: &fs::Metadata, recognized: &BTreeSet<&str>) -> Source {
    let size = meta.len();
    let keys = read_keys(path, size, scope, recognized);
    let status = match &keys {
        Keys::NotRead(reason) => unknown(format!("keys not read: {reason}")),
        Keys::Names { .. } => unknown(NO_ENVIRONMENT),
    };
    Source {
        path: path.to_path_buf(),
        scope,
        status,
        size,
        modified: seconds(meta),
        keys,
    }
}

/// Parses a JSON source in memory and keeps only its top-level key names.
/// Values are dropped; a failure is a named reason, never a crash.
fn read_keys(path: &Path, size: u64, scope: Scope, recognized: &BTreeSet<&str>) -> Keys {
    if size > MAX_READ_BYTES {
        return Keys::NotRead(format!("larger than {MAX_READ_BYTES} bytes"));
    }
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => return Keys::NotRead(format!("cannot read: {}", io_reason(&err))),
    };
    let names: Vec<String> = match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(serde_json::Value::Object(map)) => map.keys().cloned().collect(),
        Ok(_) => return Keys::NotRead("the top level is not a JSON object".to_string()),
        Err(err) => return Keys::NotRead(format!("not valid JSON ({})", err.classify_name())),
    };
    // The settings-key list does not describe the harness's own state file.
    if scope == Scope::GlobalConfig {
        return Keys::Names {
            recognized: Vec::new(),
            unrecognized: names,
            classified: false,
        };
    }
    let (recognized_names, unrecognized): (Vec<String>, Vec<String>) = names
        .into_iter()
        .partition(|name| recognized.contains(name.as_str()));
    Keys::Names {
        recognized: recognized_names,
        unrecognized,
        classified: true,
    }
}

/// Names a serde_json error by kind with its position, without echoing any file content.
trait ClassifyName {
    fn classify_name(&self) -> String;
}

impl ClassifyName for serde_json::Error {
    fn classify_name(&self) -> String {
        let kind = match self.classify() {
            serde_json::error::Category::Io => "i/o",
            serde_json::error::Category::Syntax => "syntax",
            serde_json::error::Category::Data => "data",
            serde_json::error::Category::Eof => "unexpected end",
        };
        format!(
            "{kind} error at line {} column {}",
            self.line(),
            self.column()
        )
    }
}

impl fmt::Display for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Claude Code settings sources (assuming Claude Code {}, not detected; {})",
            self.assumed_version, self.knowledge
        )?;
        writeln!(
            f,
            "note: the list of recognized keys is partial; a key not on it is shown as unrecognized, not as invalid"
        )?;
        if !self.looked_in_roots {
            writeln!(
                f,
                "note: no --root was given, so project and local settings files were not looked for"
            )?;
        }
        writeln!(f)?;
        if self.sources.is_empty() {
            writeln!(f, "No settings source was found.")?;
        }
        for source in &self.sources {
            writeln!(f, "{}", source.path.display())?;
            write!(
                f,
                "  layer: {}  harness: claude-code  status: {}",
                source.scope.name(),
                source.status
            )?;
            writeln!(f)?;
            write!(f, "  size: {} bytes", source.size)?;
            match source.modified {
                Some(secs) => writeln!(f, "  modified: {secs} (seconds since the epoch)")?,
                None => writeln!(f)?,
            }
            match &source.keys {
                Keys::Names {
                    recognized,
                    unrecognized,
                    classified,
                } => {
                    if *classified {
                        if !recognized.is_empty() {
                            writeln!(f, "  keys: {}", recognized.join(", "))?;
                        }
                        if !unrecognized.is_empty() {
                            writeln!(f, "  unrecognized keys: {}", unrecognized.join(", "))?;
                        }
                    } else if !unrecognized.is_empty() {
                        writeln!(
                            f,
                            "  keys (not classified for this file): {}",
                            unrecognized.join(", ")
                        )?;
                    }
                }
                Keys::NotRead(reason) => writeln!(f, "  keys not read: {reason}")?,
            }
        }
        if !self.not_present.is_empty() {
            writeln!(f)?;
            writeln!(f, "Looked for, not present:")?;
            for (path, scope) in &self.not_present {
                writeln!(f, "  {} ({})", path.display(), scope.name())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::{PermissionsExt, symlink};

    use usher_testkit::{ScratchHome, digest_tree};

    use super::*;

    /// A fixture value that is plainly not a credential; tests assert it never
    /// reaches the output.
    const FAKE_VALUE: &str = "FIXTURE-VALUE-NOT-A-SECRET";

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        fs::write(path, text).expect("write");
    }

    fn knowledge() -> KnowledgeFile {
        KnowledgeFile::embedded().expect("embedded")
    }

    /// An environment inside a scratch home: home, system root and platform.
    fn env(scratch: &ScratchHome, platform: Platform) -> Environment {
        Environment::new(scratch.home().to_path_buf())
            .with_system_root(scratch.root().join("system"))
            .with_platform(platform)
    }

    fn paths(inventory: &Inventory) -> Vec<String> {
        inventory
            .sources
            .iter()
            .map(|s| s.path.to_string_lossy().into_owned())
            .collect()
    }

    fn source<'a>(inventory: &'a Inventory, suffix: &str) -> &'a Source {
        inventory
            .sources
            .iter()
            .find(|s| s.path.to_string_lossy().ends_with(suffix))
            .unwrap_or_else(|| panic!("no source ending {suffix}: {:?}", paths(inventory)))
    }

    #[test]
    fn every_source_is_listed_with_location_layer_and_an_unknown_status() {
        let scratch = ScratchHome::new().expect("scratch");
        let home = scratch.home();
        let root = scratch.root().join("project");
        let managed = scratch
            .root()
            .join("system/Library/Application Support/ClaudeCode");
        write(&managed.join("managed-settings.json"), r#"{"model":"m"}"#);
        write(&managed.join("managed-settings.d/20-b.json"), "{}");
        write(&managed.join("managed-settings.d/10-a.json"), "{}");
        write(&home.join(".claude/settings.json"), r#"{"env":{}}"#);
        write(&home.join(".claude/remote-settings.json"), "{}");
        write(
            &home.join(".claude.json"),
            &format!(r#"{{"oauthAccount":"{FAKE_VALUE}","projects":{{}}}}"#),
        );
        write(&root.join(".claude/settings.json"), "{}");
        write(&root.join(".claude/settings.local.json"), "{}");
        let found = inventory(
            &env(&scratch, Platform::Macos).with_root(root),
            &knowledge(),
        )
        .expect("inventory");
        let layers: Vec<(&str, &str)> = found
            .sources
            .iter()
            .map(|s| {
                let name = s.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                (name, s.scope.name())
            })
            .collect();
        assert_eq!(
            layers,
            [
                ("remote-settings.json", "managed"),
                ("managed-settings.json", "managed"),
                ("10-a.json", "managed"),
                ("20-b.json", "managed"),
                ("settings.local.json", "local"),
                ("settings.json", "project"),
                ("settings.json", "user"),
                (".claude.json", "global-config"),
            ]
        );
        assert!(
            found
                .sources
                .iter()
                .all(|s| s.status == Status::Unknown(NO_ENVIRONMENT.to_string()))
        );
        assert!(found.not_present.is_empty(), "{:?}", found.not_present);
        assert!(found.looked_in_roots);
    }

    #[test]
    fn a_known_location_with_no_file_is_not_listed_as_present() {
        let scratch = ScratchHome::new().expect("scratch");
        let found = inventory(&env(&scratch, Platform::Linux), &knowledge()).expect("inventory");
        assert!(found.sources.is_empty());
        let looked: Vec<String> = found
            .not_present
            .iter()
            .map(|(p, _)| p.to_string_lossy().into_owned())
            .collect();
        assert!(looked.iter().any(|p| p.ends_with("/.claude/settings.json")));
        assert!(
            looked
                .iter()
                .any(|p| p.ends_with("/etc/claude-code/managed-settings.json"))
        );
        assert!(
            looked
                .iter()
                .any(|p| p.ends_with("/etc/claude-code/managed-settings.d"))
        );
        assert!(
            !looked.iter().any(|p| p.contains("Library")),
            "macOS path on Linux"
        );
    }

    #[test]
    fn unrecognized_keys_are_listed_never_dropped() {
        let scratch = ScratchHome::new().expect("scratch");
        write(
            &scratch.home().join(".claude/settings.json"),
            r#"{"model":"m","permissions":{},"brandNewKey":true,"another":1}"#,
        );
        let found = inventory(&env(&scratch, Platform::Linux), &knowledge()).expect("inventory");
        match &source(&found, "/.claude/settings.json").keys {
            Keys::Names {
                recognized,
                unrecognized,
                classified,
            } => {
                assert!(*classified);
                assert_eq!(recognized, &["model", "permissions"]);
                assert_eq!(unrecognized, &["another", "brandNewKey"]);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_global_config_file_lists_names_without_classifying_them() {
        let scratch = ScratchHome::new().expect("scratch");
        write(
            &scratch.home().join(".claude.json"),
            r#"{"projects":{},"theme":"x"}"#,
        );
        let found = inventory(&env(&scratch, Platform::Linux), &knowledge()).expect("inventory");
        match &source(&found, "/.claude.json").keys {
            Keys::Names {
                unrecognized,
                classified,
                ..
            } => {
                assert!(!*classified);
                assert_eq!(unrecognized, &["projects", "theme"]);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn an_unreadable_or_invalid_source_is_listed_with_a_reason_and_nothing_aborts() {
        let scratch = ScratchHome::new().expect("scratch");
        let home = scratch.home();
        write(&home.join(".claude/settings.json"), "{ // a comment\n}");
        write(&home.join(".claude.json"), "not json at all");
        write(&home.join(".claude/remote-settings.json"), "[1, 2]");
        let locked = home.join(".claude/locked.json");
        write(&locked, "{}");
        let found = inventory(&env(&scratch, Platform::Linux), &knowledge()).expect("inventory");
        for suffix in ["/.claude/settings.json", "/.claude.json"] {
            let found_source = source(&found, suffix);
            match &found_source.keys {
                Keys::NotRead(reason) => assert!(reason.starts_with("not valid JSON"), "{reason}"),
                other => panic!("{other:?}"),
            }
            assert!(
                matches!(&found_source.status, Status::Unknown(r) if r.starts_with("keys not read"))
            );
        }
        assert_eq!(
            source(&found, "remote-settings.json").keys,
            Keys::NotRead("the top level is not a JSON object".to_string())
        );
        // A file that cannot be read is named, too (a permission change on a
        // source usher is pointed at).
        let user = home.join(".claude/settings.json");
        fs::write(&user, "{}").expect("rewrite");
        fs::set_permissions(&user, fs::Permissions::from_mode(0o000)).expect("chmod");
        let found = inventory(&env(&scratch, Platform::Linux), &knowledge()).expect("inventory");
        assert_eq!(
            source(&found, "/.claude/settings.json").keys,
            Keys::NotRead("cannot read: permission denied".to_string())
        );
        fs::set_permissions(&user, fs::Permissions::from_mode(0o600)).expect("restore");
    }

    #[test]
    fn claude_config_dir_moves_the_user_files() {
        let scratch = ScratchHome::new().expect("scratch");
        let alt = scratch.root().join("alt-config");
        write(&alt.join("settings.json"), "{}");
        write(&alt.join(".claude.json"), "{}");
        write(&alt.join("remote-settings.json"), "{}");
        write(&scratch.home().join(".claude/settings.json"), "{}");
        let found = inventory(
            &env(&scratch, Platform::Linux).with_config_dir(alt.clone()),
            &knowledge(),
        )
        .expect("inventory");
        let listed = paths(&found);
        for name in ["settings.json", ".claude.json", "remote-settings.json"] {
            assert!(
                listed.contains(&alt.join(name).to_string_lossy().into_owned()),
                "{listed:?}"
            );
        }
        assert!(
            !listed.iter().any(|p| p.contains("/home/.claude/")),
            "{listed:?}"
        );
    }

    #[test]
    fn managed_directory_files_are_in_name_order_and_hidden_or_other_files_are_ignored() {
        let scratch = ScratchHome::new().expect("scratch");
        let dir = scratch
            .root()
            .join("system/etc/claude-code/managed-settings.d");
        for name in [
            "30-c.json",
            "10-a.json",
            ".hidden.json",
            "notes.txt",
            "20-b.json",
        ] {
            write(&dir.join(name), "{}");
        }
        let found = inventory(&env(&scratch, Platform::Linux), &knowledge()).expect("inventory");
        let names: Vec<String> = found
            .sources
            .iter()
            .filter_map(|s| s.path.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();
        assert_eq!(names, ["10-a.json", "20-b.json", "30-c.json"]);
    }

    #[test]
    fn a_symlink_root_is_followed_once_and_nothing_below_it() {
        let scratch = ScratchHome::new().expect("scratch");
        let real = scratch.root().join("dotfiles/settings.json");
        write(&real, r#"{"model":"m"}"#);
        fs::create_dir_all(scratch.home().join(".claude")).expect("mkdir");
        symlink(&real, scratch.home().join(".claude/settings.json")).expect("link");
        let dir = scratch
            .root()
            .join("system/etc/claude-code/managed-settings.d");
        write(&dir.join("10-a.json"), "{}");
        symlink(&real, dir.join("20-link.json")).expect("link");
        let found = inventory(&env(&scratch, Platform::Linux), &knowledge()).expect("inventory");
        assert!(matches!(
            &source(&found, "/.claude/settings.json").keys,
            Keys::Names { recognized, .. } if recognized == &["model"]
        ));
        assert_eq!(
            source(&found, "20-link.json").keys,
            Keys::NotRead("a symlink, not followed".to_string())
        );
        assert!(matches!(
            &source(&found, "10-a.json").keys,
            Keys::Names { .. }
        ));
    }

    #[test]
    fn project_files_are_only_looked_for_under_a_given_root() {
        let scratch = ScratchHome::new().expect("scratch");
        write(&scratch.root().join("proj/.claude/settings.json"), "{}");
        let without = inventory(&env(&scratch, Platform::Linux), &knowledge()).expect("inventory");
        assert!(without.sources.is_empty());
        assert!(!without.looked_in_roots);
        let with = inventory(
            &env(&scratch, Platform::Linux).with_root(scratch.root().join("proj")),
            &knowledge(),
        )
        .expect("inventory");
        assert_eq!(with.sources.len(), 1);
        assert_eq!(with.sources[0].scope, Scope::Project);
    }

    #[test]
    fn a_run_changes_nothing_and_prints_no_value() {
        let scratch = ScratchHome::new().expect("scratch");
        write(
            &scratch.home().join(".claude.json"),
            &format!(r#"{{"oauthAccount":"{FAKE_VALUE}"}}"#),
        );
        write(
            &scratch.home().join(".claude/settings.json"),
            &format!(r#"{{"env":{{"TOKEN":"{FAKE_VALUE}"}}}}"#),
        );
        let before = digest_tree(scratch.root()).expect("digest");
        let found = inventory(&env(&scratch, Platform::Linux), &knowledge()).expect("inventory");
        let text = found.to_string();
        assert_eq!(digest_tree(scratch.root()).expect("digest"), before);
        assert!(
            !text.contains(FAKE_VALUE),
            "a value reached the output: {text}"
        );
        assert!(
            text.contains("oauthAccount"),
            "the key name is listed: {text}"
        );
        assert!(text.contains("partial"));
        assert!(text.contains("no --root was given"));
        assert!(text.contains("no environment is mapped"));
    }

    #[test]
    fn missing_knowledge_for_the_harness_is_an_error_with_a_next_action() {
        let scratch = ScratchHome::new().expect("scratch");
        let text = "schema_version = 1\nsequence = 1\n";
        let empty = KnowledgeFile::parse(text).expect("parses");
        let err = inventory(&env(&scratch, Platform::Linux), &empty).expect_err("no locations");
        assert_eq!(err.step(), STEP);
        assert!(!err.next_action().is_empty());
    }

    #[test]
    fn the_process_environment_needs_a_home() {
        let err = Environment::from_process(|_| None).expect_err("no HOME");
        assert_eq!(err.status(), "HOME is not set");
        let env = Environment::from_process(|name| match name {
            "HOME" => Some("/h".into()),
            "CLAUDE_CONFIG_DIR" => Some("/c".into()),
            _ => None,
        })
        .expect("env");
        assert_eq!(env.config_home(), PathBuf::from("/c"));
        assert_eq!(env.global_config_home(), PathBuf::from("/c"));
    }
}
