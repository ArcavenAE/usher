//! Story 1.6, PR D: the `usher map <dir>` verb. It prints the 1.5 sources block, then which
//! value is in effect for each setting and why, then what it does not observe; it records
//! one run with verb `map`; and no value of an unlisted, sensitive or overridden key reaches
//! the output or the store. Public API, over a scratch home.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use usher_cli::{InventoryEnv, LaunchEnvironment, inventory, map, run};
use usher_core::{ReadingStatus, SystemRoot};
use usher_knowledge::Platform;
use usher_store::Store;
use usher_testkit::{NamedHome, RealHomeGuard, RealHomes, ScratchHome};

const NOW: &str = "2026-11-20T12:00:00Z";

struct Rig {
    scratch: ScratchHome,
    system: PathBuf,
    repo: PathBuf,
}

impl Rig {
    fn new() -> Self {
        let scratch = ScratchHome::new().expect("scratch");
        let system = scratch.root().join("system");
        let repo = scratch.home().join("work/repo");
        fs::create_dir_all(&system).expect("system root");
        fs::create_dir_all(repo.join(".git")).expect("repo");
        let rig = Rig {
            scratch,
            system,
            repo,
        };
        rig.install("2.1.317");
        rig
    }

    fn env(&self) -> InventoryEnv {
        InventoryEnv {
            host: "host-1".to_string(),
            user: "alex".to_string(),
            home: self.scratch.home().to_path_buf(),
            launch_dir: self.repo.clone(),
            launch: LaunchEnvironment::default(),
            path: Some(self.scratch.home().join(".local/bin").into_os_string()),
            state_dir: self.scratch.home().join(".local/state/usher"),
            now: NOW.to_string(),
            platform: Platform::Linux,
        }
    }

    fn map(&self, env: &InventoryEnv) -> String {
        map(env, &self.repo, SystemRoot::at(self.system.clone())).expect("a map")
    }

    fn write(&self, path: &Path, text: &str) -> PathBuf {
        fs::create_dir_all(path.parent().expect("a parent")).expect("dirs");
        fs::write(path, text).expect("file");
        path.to_path_buf()
    }

    fn install(&self, version: &str) {
        let target = self
            .scratch
            .home()
            .join(".local/share/claude/versions")
            .join(version);
        self.write(&target, "binary");
        fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).expect("mode");
        let launcher = self.scratch.home().join(".local/bin/claude");
        fs::create_dir_all(launcher.parent().expect("a parent")).expect("dirs");
        symlink(&target, launcher).expect("symlink");
    }

    fn user_file(&self) -> PathBuf {
        self.scratch.home().join(".claude/settings.json")
    }

    fn project_file(&self) -> PathBuf {
        self.repo.join(".claude/settings.json")
    }
}

fn line_with<'a>(output: &'a str, needle: &str) -> &'a str {
    output
        .lines()
        .find(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("no line holds `{needle}` in:\n{output}"))
}

/// Every file under a directory, for a search of what a run stored.
fn stored_bytes(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found = Vec::new();
    let mut pending = if dir.exists() {
        vec![dir.to_path_buf()]
    } else {
        Vec::new()
    };
    while let Some(next) = pending.pop() {
        for entry in fs::read_dir(&next).expect("a directory") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                found.push((path.clone(), fs::read(&path).expect("a file")));
            }
        }
    }
    found
}

fn holds(bytes: &[u8], needle: &str) -> bool {
    bytes.windows(needle.len()).any(|w| w == needle.as_bytes())
}

fn assert_nowhere(rig: &Rig, env: &InventoryEnv, output: &str, canary: &str) {
    assert!(!output.contains(canary), "`{canary}` is printed:\n{output}");
    for (path, bytes) in stored_bytes(&env.state_dir) {
        assert!(!holds(&bytes, canary), "`{canary}` is stored in {path:?}");
    }
    let _ = rig;
}

fn last_run_settings(env: &InventoryEnv) -> Vec<usher_store::SettingRow> {
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    let last = runs.last().expect("a run was recorded");
    store.settings_of(last.id).expect("settings")
}

// ---- the printed map ---------------------------------------------------------------------

/// The layout of a setting row is its own; what is pinned is that one line carries the key,
/// the status words, the value and the file.
#[test]
fn a_setting_in_effect_prints_its_status_value_and_file_on_one_line() {
    let rig = Rig::new();
    let file = rig.write(&rig.project_file(), r#"{"model": "project-model"}"#);
    let out = rig.map(&rig.env());
    let line = line_with(&out, "project-model");
    assert!(line.contains("model"), "{line}");
    assert!(line.contains(ReadingStatus::InEffect.display()), "{line}");
    assert!(line.contains(&file.display().to_string()), "{line}");
}

/// The 1.5 sources block comes first and is the same text; the settings block follows it.
#[test]
fn the_sources_block_comes_first_and_the_settings_block_follows() {
    let rig = Rig::new();
    let user = rig.write(&rig.user_file(), r#"{"model": "user-model"}"#);
    let other = Rig::new();
    other.write(&other.user_file(), r#"{"model": "user-model"}"#);
    let inventory_text =
        inventory(&other.env(), SystemRoot::at(other.system.clone())).expect("an inventory");
    let first = inventory_text.lines().next().expect("a stamp line");
    let out = rig.map(&rig.env());
    assert_eq!(out.lines().next(), Some(first), "{out}");
    let source_line = out.find(&user.display().to_string()).expect("a source row");
    let setting_line = out.find("user-model").expect("a setting row");
    assert!(source_line < setting_line, "{out}");
}

/// A command line `usher map <dir>` runs the verb; a missing directory is a named refusal.
#[test]
fn run_dispatches_map_with_a_directory_and_refuses_it_without_one() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "user-model"}"#);
    let env = rig.env();
    let args = vec!["map".to_string(), rig.repo.display().to_string()];
    let out = run(&args, &env).expect("map runs through the dispatcher");
    assert!(out.contains("user-model"), "{out}");
    let err = run(&["map".to_string()], &env).expect_err("a directory is required");
    assert!(
        !err.status().contains("not a usher command"),
        "map is a command: {err:?}"
    );
    assert!(err.next_action().contains("usher map"), "{err:?}");
}

// ---- the footer --------------------------------------------------------------------------

/// The reader names what it does not observe as data; the verb renders each as text.
#[test]
fn the_footer_says_what_the_map_does_not_observe() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "m"}"#);
    let out = rig.map(&rig.env());
    for text in [
        "command-line settings (--settings, flags) at launch are not observable",
        "settings `env` values are not read",
        "managed values held server-side may differ from the cache",
    ] {
        assert!(out.contains(text), "`{text}` is missing from:\n{out}");
    }
    assert!(!out.contains("built-in classifier rules"), "{out}");
}

#[test]
fn the_classifier_line_is_printed_only_when_an_auto_mode_list_entry_is_in_effect() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"autoMode": {"allow": ["rule"]}}"#);
    let out = rig.map(&rig.env());
    assert!(
        out.contains(
            "built-in classifier rules are not observed; an array without \"$defaults\" may replace them"
        ),
        "{out}"
    );
}

// ---- the store record --------------------------------------------------------------------

/// One run per `usher map`, with verb `map`, the sources recorded as the inventory does,
/// the environment key as the inventory builds it, and the readings stored.
#[test]
fn one_map_records_one_run_with_the_map_verb_its_sources_and_its_readings() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "user-model"}"#);
    let env = rig.env();
    rig.map(&env);
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 1, "{runs:?}");
    assert_eq!(runs[0].verb, "map");
    assert_eq!(
        runs[0].environment_key,
        format!("host-1/alex/claude-code/{}", rig.repo.display())
    );
    assert!(!store.sources_of(runs[0].id).expect("sources").is_empty());
    let rows = store.settings_of(runs[0].id).expect("settings");
    let model = rows.iter().find(|r| r.key == "model").expect("a model row");
    assert_eq!(model.layer, "user");
    assert_eq!(model.status, ReadingStatus::InEffect.token());
    assert_eq!(model.method, "rebuilt");
    assert_eq!(
        model.value_text.as_deref(),
        Some("\"user-model\""),
        "the stored value is compact JSON, so a string stays quoted"
    );
    rig.map(&env);
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 2, "a second map is a second run");
}

/// The directory is keyed as given. A symlinked directory is not resolved for the key; its
/// target is printed as information only.
#[test]
fn a_symlinked_directory_is_keyed_as_given_and_its_target_is_printed() {
    let rig = Rig::new();
    let link = rig.scratch.home().join("work/link");
    symlink(&rig.repo, &link).expect("symlink");
    let env = rig.env();
    let out = map(&env, &link, SystemRoot::at(rig.system.clone())).expect("a map");
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(
        runs[0].environment_key,
        format!("host-1/alex/claude-code/{}", link.display())
    );
    let target = fs::canonicalize(&rig.repo).expect("a target");
    assert!(out.contains(&target.display().to_string()), "{out}");
}

#[test]
fn a_directory_that_is_missing_or_a_file_is_refused_by_name_and_nothing_is_recorded() {
    let rig = Rig::new();
    let env = rig.env();
    let file = rig.write(&rig.scratch.home().join("work/a-file"), "x");
    for bad in [rig.scratch.home().join("work/missing"), file] {
        let err = map(&env, &bad, SystemRoot::at(rig.system.clone()))
            .expect_err("the directory is refused");
        assert!(
            err.status().contains(&bad.display().to_string())
                || err.check().contains(&bad.display().to_string()),
            "{err:?}"
        );
        assert!(!err.next_action().is_empty(), "{err:?}");
    }
    assert!(
        !env.state_dir.exists()
            || Store::open(&env.state_dir)
                .expect("the store opens")
                .runs()
                .expect("runs")
                .is_empty()
    );
}

/// Every status token the reader prints is one the store's check list accepts, and the
/// check list holds no other: a change to either list fails here.
#[test]
fn the_status_tokens_are_exactly_the_stores_check_list() {
    let rig = Rig::new();
    let env = rig.env();
    rig.map(&env);
    assert!(
        env.state_dir.join("usher.db").exists(),
        "the map recorded no run"
    );
    let conn = stored_check_list(&env.state_dir);
    let mut tokens: Vec<&str> = [
        ReadingStatus::InEffect,
        ReadingStatus::Shadowed,
        ReadingStatus::Unread,
        ReadingStatus::NotDecided,
        ReadingStatus::OverriddenByEnvironment,
        ReadingStatus::ValueNotRead,
        ReadingStatus::NotChecked,
        ReadingStatus::Unknown,
    ]
    .iter()
    .map(|s| s.token())
    .collect();
    tokens.sort_unstable();
    let mut stored: Vec<&str> = conn.iter().map(String::as_str).collect();
    stored.sort_unstable();
    assert_eq!(tokens, stored);
}

/// The status list as the stored table definition writes it, read through a second
/// connection.
fn stored_check_list(state_dir: &Path) -> Vec<String> {
    let conn = rusqlite::Connection::open(state_dir.join("usher.db")).expect("a connection");
    let sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'map_setting'",
            [],
            |r| r.get(0),
        )
        .expect("the table definition");
    let start = sql
        .find("CHECK (status IN (")
        .expect("the status check is in the table definition");
    let rest = &sql[start + "CHECK (status IN (".len()..];
    let end = rest.find("))").expect("the list ends");
    rest[..end]
        .split(',')
        .map(|item| item.trim().trim_matches('\'').to_string())
        .collect()
}

// ---- values that must never reach the output or the store -------------------------------

/// A unique token in an `env` value, in an unlisted key and under a credential key appears
/// in neither the output nor any file the run stored.
#[test]
fn canaries_reach_neither_the_output_nor_the_store() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"env": {"FLEET_TOKEN": "canary-env-value"}, "theme": "canary-unlisted", "x": {"oauthAccount": {"k": "canary-credential"}}}"#,
    );
    let env = rig.env();
    let out = rig.map(&env);
    assert!(
        out.contains("FLEET_TOKEN"),
        "the name is still printed:\n{out}"
    );
    for canary in ["canary-env-value", "canary-unlisted", "canary-credential"] {
        assert_nowhere(&rig, &env, &out, canary);
    }
}

/// The store does not refuse a sensitive value itself, so the caller holds it back: an
/// `env` reading is stored as value not read, with no value.
#[test]
fn an_env_reading_is_stored_as_value_not_read_with_no_value() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"env": {"FLEET_TOKEN": "canary-stored-token"}}"#,
    );
    let env = rig.env();
    let out = rig.map(&env);
    let rows = last_run_settings(&env);
    let row = rows
        .iter()
        .find(|r| r.key == "env.FLEET_TOKEN")
        .unwrap_or_else(|| panic!("no env.FLEET_TOKEN row in {rows:?}"));
    assert_eq!(row.status, ReadingStatus::ValueNotRead.token());
    assert_eq!(row.value_text, None);
    assert!(
        rows.iter()
            .all(|r| r.value_text.is_none() || r.key != "env")
    );
    assert_nowhere(&rig, &env, &out, "canary-stored-token");
}

/// A segment that holds a dot is one segment: it prints as a JSON string, and it never
/// collides in the store with the same text as two segments.
#[test]
fn a_dotted_segment_is_one_segment_in_the_output_and_the_store() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"a": {"b.c": 1}}"#);
    rig.write(&rig.project_file(), r#"{"a": {"b": {"c": 1}}}"#);
    let env = rig.env();
    let out = rig.map(&env);
    assert!(out.contains("a.\"b.c\""), "{out}");
    let rows = last_run_settings(&env);
    let dotted = rows
        .iter()
        .find(|r| r.layer == "user" && r.key.contains("b.c"))
        .expect("the dotted row");
    let split = rows
        .iter()
        .find(|r| r.layer == "project" && r.key.contains('b'))
        .expect("the two-segment row");
    assert_ne!(dotted.key, split.key, "{rows:?}");
}

/// A credential key at the top of a file is listed by name with nothing under it.
#[test]
fn a_top_level_credential_key_is_listed_by_name_and_nothing_under_it_is_printed() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"oauthAccount": {"canary-k": "canary-v"}}"#,
    );
    let env = rig.env();
    let out = rig.map(&env);
    let line = line_with(&out, "oauthAccount");
    assert!(line.contains("credential key; not opened"), "{line}");
    assert_nowhere(&rig, &env, &out, "canary-k");
    assert_nowhere(&rig, &env, &out, "canary-v");
}

// ---- the observation invariant -----------------------------------------------------------

#[test]
fn a_map_changes_no_harness_home_and_no_managed_directory() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "m"}"#);
    rig.write(
        &rig.scratch.home().join(".claude.json"),
        r#"{"projects":{}}"#,
    );
    rig.write(
        &rig.system.join("etc/claude-code/managed-settings.json"),
        r#"{"permissions":{}}"#,
    );
    let watched = RealHomes::from_homes(vec![
        NamedHome {
            name: "claude-code",
            path: rig.scratch.home().join(".claude"),
        },
        NamedHome {
            name: "claude-code-global-state",
            path: rig.scratch.home().join(".claude.json"),
        },
        NamedHome {
            name: "claude-code-managed",
            path: rig.system.join("etc/claude-code"),
        },
        NamedHome {
            name: "claude-code-managed-mac",
            path: rig.system.join("Library/Application Support/ClaudeCode"),
        },
    ]);
    let guard = RealHomeGuard::snapshot(&watched).expect("snapshot");
    let out = rig.map(&rig.env());
    guard.assert_unchanged();
    assert!(
        out.contains("etc/claude-code/managed-settings.json"),
        "the run did look at the watched managed directory:\n{out}"
    );
}
