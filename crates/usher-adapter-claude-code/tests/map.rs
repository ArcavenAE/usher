//! Story 1.6, PR C: the Claude Code settings map reader. A pure function over the source
//! listing and the files it names: which layer decides each key, which entries a list key
//! carries from which layer, which keys a layer is not read for, and what the environment
//! overrides. Values appear only for keys knowledge marks readable and not sensitive.
//! Public API.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use usher_adapter_claude_code::ClaudeCodeAdapter;
use usher_core::{
    LaunchContext, NotObserved, ReadingMethod, ReadingStatus, SettingReading, SettingsMap,
    SettingsMapper, SourceLister, SourceListing, SourceStatus, SystemRoot, environment_key,
};
use usher_knowledge::{KnowledgeFile, Platform};
use usher_testkit::ScratchHome;

const VERSION: &str = "2.1.317";

/// A small knowledge file: the three always-read layers, a stack, one plain key and one
/// sensitive key that only the user layer may set.
const FIXTURE: &str = r#"
schema_version = 5
sequence = 30

[[item]]
id = "claude-code.fx-config-dir"
harness = "claude-code"
layer = "slow"
class = "paths"
versions = ">=2.1.317, <3.0.0"
summary = "The config directory variable."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/env" }
last_verified = 2026-11-20
variable = { name = "CLAUDE_CONFIG_DIR", binds = "${CONFIG_DIR}", default = "${HOME}/.claude" }

[[item]]
id = "claude-code.fx-user"
harness = "claude-code"
layer = "slow"
class = "sources"
versions = ">=2.1.317, <3.0.0"
summary = "The user file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/user" }
last_verified = 2026-11-20
source = { path = "${CONFIG_DIR}/settings.json", layer = "user", read = "read", rank = 1, always_present = true }

[[item]]
id = "claude-code.fx-project"
harness = "claude-code"
layer = "slow"
class = "sources"
versions = ">=2.1.317, <3.0.0"
summary = "The project file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/project" }
last_verified = 2026-11-20
source = { path = "${LAUNCH_DIR}/.claude/settings.json", layer = "project", read = "read", rank = 1, always_present = true }

[[item]]
id = "claude-code.fx-managed"
harness = "claude-code"
layer = "slow"
class = "sources"
versions = ">=2.1.317, <3.0.0"
summary = "The managed file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/managed" }
last_verified = 2026-11-20
source = { path = "/etc/claude-code/managed-settings.json", layer = "managed", read = "first-wins", rank = 1, always_present = false, platforms = ["linux"] }

[[item]]
id = "claude-code.fx-local"
harness = "claude-code"
layer = "slow"
class = "sources"
versions = ">=2.1.317, <3.0.0"
summary = "The local file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/local" }
last_verified = 2026-11-20
source = { path = "${LAUNCH_DIR}/.claude/settings.local.json", layer = "local", read = "read", rank = 1, always_present = false }

[[item]]
id = "claude-code.fx-stack"
harness = "claude-code"
layer = "slow"
class = "layers"
versions = ">=2.1.317, <3.0.0"
summary = "A stack that puts user above project."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/stack" }
last_verified = 2026-11-20
stack = ["managed", "command-line", "local", "user", "project"]

[[item]]
id = "claude-code.fx-plain"
harness = "claude-code"
layer = "fast"
class = "settings-keys"
versions = ">=2.1.317, <3.0.0"
summary = "A plain key."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/plain" }
last_verified = 2026-11-20
settings_key = { key = "fx.plain", merge = "scalar", sensitive = false, scopes = ["any"] }

[[item]]
id = "claude-code.fx-token"
harness = "claude-code"
layer = "fast"
class = "settings-keys"
versions = ">=2.1.317, <3.0.0"
summary = "A sensitive scalar that any file may set."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/token" }
last_verified = 2026-11-20
settings_key = { key = "fx.token", merge = "scalar", sensitive = true, scopes = ["any"] }

[[item]]
id = "claude-code.fx-secrets"
harness = "claude-code"
layer = "fast"
class = "settings-keys"
versions = ">=2.1.317, <3.0.0"
summary = "A sensitive list that the user and managed files may set."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/secrets" }
last_verified = 2026-11-20
settings_key = { key = "fx.secrets", merge = "list-merge", sensitive = true, scopes = ["user", "managed"] }

[[item]]
id = "claude-code.fx-scoped"
harness = "claude-code"
layer = "fast"
class = "settings-keys"
versions = ">=2.1.317, <3.0.0"
summary = "A plain scalar that the user and local files may set."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/scoped" }
last_verified = 2026-11-20
settings_key = { key = "fx.scoped", merge = "scalar", sensitive = false, scopes = ["user", "local"] }

[[item]]
id = "claude-code.fx-nested"
harness = "claude-code"
layer = "fast"
class = "settings-keys"
versions = ">=2.1.317, <3.0.0"
summary = "A nested plain scalar."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/nested" }
last_verified = 2026-11-20
settings_key = { key = "fx.nested.leaf", merge = "scalar", sensitive = false, scopes = ["any"] }

[[item]]
id = "claude-code.fx-secret"
harness = "claude-code"
layer = "fast"
class = "settings-keys"
versions = ">=2.1.317, <3.0.0"
summary = "A sensitive key that only the user file may set."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/secret" }
last_verified = 2026-11-20
settings_key = { key = "fx.secret", merge = "scalar", sensitive = true, scopes = ["user"] }
"#;

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
        fs::create_dir_all(&repo).expect("repo");
        Rig {
            scratch,
            system,
            repo,
        }
    }

    fn home(&self) -> PathBuf {
        self.scratch.home().to_path_buf()
    }

    fn user_file(&self) -> PathBuf {
        self.home().join(".claude/settings.json")
    }

    fn project_file(&self) -> PathBuf {
        self.repo.join(".claude/settings.json")
    }

    fn local_file(&self) -> PathBuf {
        self.repo.join(".claude/settings.local.json")
    }

    fn write(&self, path: &Path, text: &str) -> PathBuf {
        fs::create_dir_all(path.parent().expect("a parent")).expect("dirs");
        fs::write(path, text).expect("file");
        path.to_path_buf()
    }

    fn context(&self, set: &[&str]) -> LaunchContext {
        self.context_at(Some(VERSION), set)
    }

    fn context_at(&self, version: Option<&str>, set: &[&str]) -> LaunchContext {
        self.context_in(&self.repo, version, set)
    }

    fn context_in(&self, launch_dir: &Path, version: Option<&str>, set: &[&str]) -> LaunchContext {
        let home = self.home();
        let key = environment_key("host", "alex", "claude-code", launch_dir, &[])
            .expect("an absolute directory");
        LaunchContext {
            key,
            config_dir: home.join(".claude"),
            global_state_dir: home.clone(),
            set_variables: set
                .iter()
                .map(|s| (*s).to_string())
                .collect::<BTreeSet<_>>(),
            home,
            repo: Some(self.repo.clone()),
            user: "alex".to_string(),
            harness_version: version.map(str::to_string),
        }
    }

    fn adapter(&self, knowledge: KnowledgeFile) -> ClaudeCodeAdapter {
        ClaudeCodeAdapter::new(
            knowledge,
            SystemRoot::at(self.system.clone()),
            Platform::Linux,
        )
    }

    fn listing(&self, adapter: &ClaudeCodeAdapter, context: &LaunchContext) -> SourceListing {
        adapter.list_sources(context).expect("a listing")
    }

    /// The map over the embedded knowledge.
    fn seed_map(&self, set: &[&str]) -> SettingsMap {
        let adapter = self.adapter(KnowledgeFile::embedded().expect("embedded"));
        let context = self.context(set);
        let listing = self.listing(&adapter, &context);
        adapter.map_settings(&listing, &context).expect("a map")
    }

    /// The map over the fixture knowledge.
    fn fixture_map(&self) -> SettingsMap {
        let adapter = self.adapter(KnowledgeFile::parse(FIXTURE).expect("fixture"));
        let context = self.context(&[]);
        let listing = self.listing(&adapter, &context);
        adapter.map_settings(&listing, &context).expect("a map")
    }
}

fn readings<'a>(map: &'a SettingsMap, key: &str) -> Vec<&'a SettingReading> {
    map.readings.iter().filter(|r| r.key == key).collect()
}

fn reading<'a>(map: &'a SettingsMap, key: &str, layer: &str) -> &'a SettingReading {
    let found: Vec<&SettingReading> = readings(map, key)
        .into_iter()
        .filter(|r| r.layer == layer)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "expected one reading of {key} in {layer}, found {found:?} in {:?}",
        map.readings
    );
    found[0]
}

/// The typed JSON string a reading must hold.
fn text(value: &str) -> Option<serde_json::Value> {
    Some(serde_json::Value::String(value.to_string()))
}

fn note_has(reading: &SettingReading, text: &str) -> bool {
    reading.note.as_deref().is_some_and(|n| n.contains(text))
}

/// A value is typed JSON: the string "3" and the number 3 stay distinct, and so do the
/// string "true" and the boolean.
#[test]
fn a_value_is_typed_json_so_a_string_and_a_number_stay_distinct() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"fx": {"plain": 3}}"#);
    rig.write(&rig.project_file(), r#"{"fx": {"plain": "3"}}"#);
    let map = rig.fixture_map();
    assert_eq!(
        reading(&map, "fx.plain", "user").value,
        Some(serde_json::json!(3))
    );
    assert_eq!(reading(&map, "fx.plain", "project").value, text("3"));
}

// ---- scalar keys follow the stack ------------------------------------------------------

/// A key whose scopes are `any`, set in the project and user files, follows the stack the
/// knowledge file states: the project value is in effect, and the user value is shadowed
/// with the winning layer named.
#[test]
fn an_any_scope_scalar_set_in_project_and_user_follows_the_stack() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "user-model"}"#);
    rig.write(&rig.project_file(), r#"{"model": "project-model"}"#);
    let map = rig.seed_map(&[]);
    let project = reading(&map, "model", "project");
    assert_eq!(project.status, ReadingStatus::InEffect);
    assert_eq!(project.value, text("project-model"), "{project:?}");
    assert_eq!(project.source_path, rig.project_file());
    let user = reading(&map, "model", "user");
    assert_eq!(user.status, ReadingStatus::Shadowed);
    assert_eq!(
        user.note.as_deref(),
        Some(format!("shadowed by project ({})", rig.project_file().display()).as_str()),
        "the winner is named with its layer and its file: {user:?}"
    );
    assert_eq!(user.source_path, rig.user_file());
}

#[test]
fn local_outranks_project_and_project_outranks_user_in_the_seeded_stack() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "u"}"#);
    rig.write(&rig.project_file(), r#"{"model": "p"}"#);
    rig.write(&rig.local_file(), r#"{"model": "l"}"#);
    let map = rig.seed_map(&[]);
    assert_eq!(
        reading(&map, "model", "local").status,
        ReadingStatus::InEffect
    );
    let project = reading(&map, "model", "project");
    let user = reading(&map, "model", "user");
    assert_eq!(project.status, ReadingStatus::Shadowed);
    assert_eq!(user.status, ReadingStatus::Shadowed);
    let winner = format!("shadowed by local ({})", rig.local_file().display());
    assert_eq!(project.note.as_deref(), Some(winner.as_str()));
    assert_eq!(user.note.as_deref(), Some(winner.as_str()));
}

/// The order across layers is knowledge, not a constant in the reader: a knowledge file
/// whose stack puts user above project flips the winner for the same two files.
#[test]
fn the_winner_comes_from_the_stack_in_knowledge_and_not_from_the_reader() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"fx": {"plain": "from-user"}}"#);
    rig.write(&rig.project_file(), r#"{"fx": {"plain": "from-project"}}"#);
    let map = rig.fixture_map();
    assert_eq!(
        reading(&map, "fx.plain", "user").status,
        ReadingStatus::InEffect
    );
    assert_eq!(
        reading(&map, "fx.plain", "project").status,
        ReadingStatus::Shadowed
    );
}

// ---- list keys keep every entry with its layer -----------------------------------------

#[test]
fn a_list_key_keeps_every_entry_with_its_layer_file_and_position() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"permissions": {"allow": ["Bash(ls)", "Read(a)"]}}"#,
    );
    rig.write(
        &rig.project_file(),
        r#"{"permissions": {"allow": ["Edit(b)"]}}"#,
    );
    let map = rig.seed_map(&[]);
    let rows = readings(&map, "permissions.allow");
    let shape: Vec<(&str, Option<u32>, ReadingStatus)> = rows
        .iter()
        .map(|r| (r.layer.as_str(), r.entry_index, r.status))
        .collect();
    assert_eq!(
        shape,
        [
            ("project", Some(0), ReadingStatus::InEffect),
            ("user", Some(0), ReadingStatus::InEffect),
            ("user", Some(1), ReadingStatus::InEffect),
        ],
        "layers highest first, then entry position: {rows:?}"
    );
    assert_eq!(rows[0].value, text("Edit(b)"));
    assert_eq!(rows[0].source_path, rig.project_file());
    assert_eq!(rows[2].value, text("Read(a)"));
    assert_eq!(rows[2].source_path, rig.user_file());
}

/// A lower-ranked duplicate stays in effect and says "also set in" on its row, naming the
/// higher layer that carries the same entry.
#[test]
fn a_duplicate_list_entry_in_a_lower_layer_stays_in_effect_and_says_it_is_also_set() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"permissions": {"allow": ["Bash(ls)"]}}"#,
    );
    rig.write(
        &rig.local_file(),
        r#"{"permissions": {"allow": ["Bash(ls)"]}}"#,
    );
    let map = rig.seed_map(&[]);
    let local = reading(&map, "permissions.allow", "local");
    let user = reading(&map, "permissions.allow", "user");
    assert_eq!(local.status, ReadingStatus::InEffect);
    assert_eq!(user.status, ReadingStatus::InEffect);
    assert!(note_has(user, "also set in local"), "{user:?}");
}

/// Provisional: the rule text is shown verbatim in local output, because the rule is the
/// setting. The default may change when the product choice is made.
#[test]
fn a_permission_rule_is_shown_verbatim() {
    let rig = Rig::new();
    let rule = r#"Bash(curl -H "Authorization: Bearer abc123" https://example.test)"#;
    rig.write(
        &rig.user_file(),
        &format!(
            r#"{{"permissions": {{"deny": [{}]}}}}"#,
            serde_json_string(rule)
        ),
    );
    let map = rig.seed_map(&[]);
    let row = reading(&map, "permissions.deny", "user");
    assert_eq!(
        row.value,
        Some(serde_json::Value::String(rule.to_string())),
        "{row:?}"
    );
}

fn serde_json_string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

// ---- keys outside their scopes ---------------------------------------------------------

/// `autoMode.allow` may be set in the user and managed layers only. Set in a project file
/// and in the user file, the value in effect comes from the user file alone, and the
/// project row says why it did not count.
#[test]
fn a_key_found_in_a_layer_outside_its_scopes_is_unread_and_never_feeds_the_value() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"autoMode": {"allow": ["user rule"]}}"#,
    );
    rig.write(
        &rig.project_file(),
        r#"{"autoMode": {"allow": ["project rule"]}}"#,
    );
    let map = rig.seed_map(&[]);
    let user = reading(&map, "autoMode.allow", "user");
    assert_eq!(user.status, ReadingStatus::InEffect);
    assert_eq!(user.value, text("user rule"));
    let project = reading(&map, "autoMode.allow", "project");
    assert_eq!(project.status, ReadingStatus::Unread);
    assert_eq!(
        project.note.as_deref(),
        Some("not read from project for this key; knowledge item claude-code.key-automode-allow"),
        "{project:?}"
    );
    assert_eq!(
        project.value,
        text("project rule"),
        "a readable key's value is shown"
    );
    assert!(
        readings(&map, "autoMode.allow")
            .iter()
            .filter(|r| r.status == ReadingStatus::InEffect)
            .all(|r| r.layer == "user"),
        "{:?}",
        readings(&map, "autoMode.allow")
    );
}

#[test]
fn a_sensitive_key_found_outside_its_scopes_is_unread_and_shows_no_value() {
    let rig = Rig::new();
    rig.write(
        &rig.project_file(),
        r#"{"fx": {"secret": "canary-secret"}}"#,
    );
    let map = rig.fixture_map();
    let row = reading(&map, "fx.secret", "project");
    assert_eq!(row.status, ReadingStatus::Unread);
    assert_eq!(row.value, None, "{row:?}");
    assert!(!format!("{map:?}").contains("canary-secret"));
}

// ---- keys the knowledge file does not list ---------------------------------------------

/// A key under a seeded parent that knowledge does not list prints by name only.
#[test]
fn an_unseeded_child_of_a_seeded_parent_is_a_name_with_no_value() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"autoMode": {"someNewArray": ["canary-new-rule"]}}"#,
    );
    let map = rig.seed_map(&[]);
    let row = reading(&map, "autoMode.someNewArray", "user");
    assert_eq!(row.status, ReadingStatus::ValueNotRead);
    assert_eq!(row.value, None);
    assert_eq!(row.note.as_deref(), Some("key not marked readable"));
    assert!(!format!("{map:?}").contains("canary-new-rule"));
}

#[test]
fn an_unseeded_top_level_key_is_a_name_with_no_value() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"theme": "canary-theme"}"#);
    let map = rig.seed_map(&[]);
    let row = reading(&map, "theme", "user");
    assert_eq!(row.status, ReadingStatus::ValueNotRead);
    assert_eq!(row.value, None);
    assert_eq!(row.note.as_deref(), Some("key not marked readable"));
    assert!(!format!("{map:?}").contains("canary-theme"));
}

/// `env` is listed and sensitive: its children print by name only, and no value reaches
/// the map.
#[test]
fn the_children_of_a_sensitive_key_print_by_name_only() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"env": {"FLEET_TOKEN": "canary-env-value", "OTHER": "canary-other"}}"#,
    );
    let map = rig.seed_map(&[]);
    for name in ["env.FLEET_TOKEN", "env.OTHER"] {
        let row = reading(&map, name, "user");
        assert_eq!(row.status, ReadingStatus::ValueNotRead, "{row:?}");
        assert_eq!(row.value, None);
    }
    let text = format!("{map:?}");
    assert!(!text.contains("canary-env-value") && !text.contains("canary-other"));
}

// ---- own-rule keys ---------------------------------------------------------------------

/// Both rows print under their layers, and no layer is marked in effect for the key.
#[test]
fn default_mode_set_in_two_layers_is_not_decided_and_shows_both_rows() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"permissions": {"defaultMode": "acceptEdits"}}"#,
    );
    rig.write(
        &rig.project_file(),
        r#"{"permissions": {"defaultMode": "auto"}}"#,
    );
    let map = rig.seed_map(&[]);
    let rows = readings(&map, "permissions.defaultMode");
    assert_eq!(rows.len(), 2, "{rows:?}");
    for row in &rows {
        assert_eq!(row.status, ReadingStatus::NotDecided, "{row:?}");
        assert_eq!(row.note.as_deref(), Some("own merge rule"));
    }
    assert_eq!(
        reading(&map, "permissions.defaultMode", "user").value,
        text("acceptEdits")
    );
    assert_eq!(
        reading(&map, "permissions.defaultMode", "project").value,
        text("auto")
    );
    assert!(rows.iter().all(|r| r.status != ReadingStatus::InEffect));
}

// ---- the environment override ----------------------------------------------------------

/// With `ANTHROPIC_MODEL` present, no file value is shown as in effect. The note holds the
/// variable's name. The launch context holds names only, so no value of the variable can
/// reach the map through it.
#[test]
fn an_override_variable_that_is_present_overrides_the_key_and_the_note_holds_only_its_name() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "canary-file-model"}"#);
    let map = rig.seed_map(&["ANTHROPIC_MODEL"]);
    let row = reading(&map, "model", "user");
    assert_eq!(
        row.status,
        ReadingStatus::OverriddenByEnvironment,
        "{row:?}"
    );
    assert!(note_has(row, "ANTHROPIC_MODEL"), "{row:?}");
    assert_eq!(row.value, None, "no file value is shown as in effect");
    assert!(!format!("{map:?}").contains("canary-file-model"));
    assert!(
        readings(&map, "model")
            .iter()
            .all(|r| r.status != ReadingStatus::InEffect)
    );
}

#[test]
fn the_override_does_nothing_while_its_variable_is_not_set() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "file-model"}"#);
    let map = rig.seed_map(&[]);
    let row = reading(&map, "model", "user");
    assert_eq!(row.status, ReadingStatus::InEffect);
}

// ---- method and skipped layers ---------------------------------------------------------

/// No Claude Code row is harness-reported: usher does not call the harness.
#[test]
fn every_reading_of_the_claude_code_map_is_rebuilt() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"model": "m", "permissions": {"allow": ["Bash(ls)"], "defaultMode": "plan"}, "theme": "t", "autoMode": {"allow": ["a"], "other": 1}, "env": {"K": "v"}}"#,
    );
    rig.write(&rig.project_file(), r#"{"autoMode": {"allow": ["b"]}}"#);
    let map = rig.seed_map(&["ANTHROPIC_MODEL"]);
    assert!(!map.readings.is_empty(), "the map has readings to check");
    assert!(
        map.readings
            .iter()
            .all(|r| r.method == ReadingMethod::Rebuilt),
        "{:?}",
        map.readings
    );
}

/// A layer the listing says the version does not read takes no part in deciding values.
/// It is reported with the reason knowledge states, and with the harness's own reason
/// named as not given, since usher does not ask the harness.
#[test]
fn a_skipped_layer_carries_the_knowledge_reason_and_says_the_harness_gave_none() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "user-model"}"#);
    rig.write(&rig.project_file(), r#"{"model": "project-model"}"#);
    let adapter = rig.adapter(KnowledgeFile::embedded().expect("embedded"));
    let context = rig.context(&[]);
    let mut listing = rig.listing(&adapter, &context);
    let project = listing
        .sources
        .iter_mut()
        .find(|s| s.layer == "project")
        .expect("a project source");
    project.status = SourceStatus::Unread {
        reason: "knowledge item claude-code.source-project-settings says this version skips it"
            .to_string(),
    };
    let map = adapter.map_settings(&listing, &context).expect("a map");
    let skipped = map
        .skipped_layers
        .iter()
        .find(|s| s.layer == "project")
        .unwrap_or_else(|| panic!("no skipped project layer in {:?}", map.skipped_layers));
    assert!(
        skipped
            .reason
            .contains("claude-code.source-project-settings says this version skips it"),
        "{skipped:?}"
    );
    assert!(
        skipped
            .reason
            .contains("reason: not given (usher does not call /status)"),
        "{skipped:?}"
    );
    let user = reading(&map, "model", "user");
    assert_eq!(
        user.status,
        ReadingStatus::InEffect,
        "the skipped layer decides nothing"
    );
    assert!(
        readings(&map, "model")
            .iter()
            .filter(|r| r.layer == "project")
            .all(|r| r.status != ReadingStatus::InEffect)
    );
}

/// The reader reads only keys knowledge lists: a value under a key knowledge does not
/// list is dropped before it reaches any reading, at any depth.
#[test]
fn no_value_of_an_unlisted_key_reaches_the_map_at_any_depth() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"deep": {"a": {"b": "canary-deep"}}, "list": ["canary-list"], "model": "m"}"#,
    );
    let map = rig.seed_map(&[]);
    let text = format!("{map:?}");
    assert!(
        !text.contains("canary-deep") && !text.contains("canary-list"),
        "{text}"
    );
    assert_eq!(
        reading(&map, "model", "user").status,
        ReadingStatus::InEffect,
        "the listed key beside them is read"
    );
}

// ---- keys set nowhere ------------------------------------------------------------------

/// A readable key set in no file gets no row: what applies then is the harness's built-in
/// default, which usher does not observe. The unlisted key beside it shows the file was
/// read.
#[test]
fn a_readable_key_set_in_no_file_gets_no_row() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"theme": "dark"}"#);
    let map = rig.seed_map(&[]);
    assert_eq!(
        reading(&map, "theme", "user").status,
        ReadingStatus::ValueNotRead,
        "the file was read"
    );
    for key in ["model", "permissions.allow", "permissions.defaultMode"] {
        assert!(readings(&map, key).is_empty(), "{key}: {:?}", map.readings);
    }
}

// ---- the children of a sensitive object ------------------------------------------------

/// `env` children are one row per name per layer. No value is read, so no child is in
/// effect and none is shadowed: the map claims no winner it cannot show.
#[test]
fn no_env_child_is_in_effect_or_shadowed_in_any_layer() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"env": {"FLEET_TOKEN": "canary-user"}}"#,
    );
    rig.write(
        &rig.project_file(),
        r#"{"env": {"FLEET_TOKEN": "canary-project"}}"#,
    );
    let map = rig.seed_map(&[]);
    let rows = readings(&map, "env.FLEET_TOKEN");
    let layers: Vec<&str> = rows.iter().map(|r| r.layer.as_str()).collect();
    assert_eq!(layers, ["project", "user"], "one row per layer: {rows:?}");
    assert!(
        rows.iter().all(|r| r.status == ReadingStatus::ValueNotRead),
        "{rows:?}"
    );
    assert!(
        map.readings.iter().all(|r| r.key != "env"),
        "{:?}",
        map.readings
    );
    assert!(
        rows.iter().all(|r| r.value.is_none()),
        "no env child holds a value: {rows:?}"
    );
    assert!(!format!("{map:?}").contains("canary"), "{map:?}");
}

// ---- a whole source the version does not read ------------------------------------------

/// A row that is unread because its whole source is unread names the source item, not the
/// key item.
#[test]
fn a_row_of_an_unread_source_names_the_source_item() {
    let rig = Rig::new();
    rig.write(&rig.project_file(), r#"{"model": "project-model"}"#);
    let adapter = rig.adapter(KnowledgeFile::embedded().expect("embedded"));
    let context = rig.context(&[]);
    let mut listing = rig.listing(&adapter, &context);
    let project = listing
        .sources
        .iter_mut()
        .find(|s| s.layer == "project")
        .expect("a project source");
    let item = project.item_id.clone();
    project.status = SourceStatus::Unread {
        reason: "this version skips it".to_string(),
    };
    let map = adapter.map_settings(&listing, &context).expect("a map");
    let row = reading(&map, "model", "project");
    assert_eq!(row.status, ReadingStatus::Unread);
    assert_eq!(
        row.note.as_deref(),
        Some(format!("not read from project; knowledge item {item}").as_str()),
        "{row:?}"
    );
    assert!(!note_has(row, "claude-code.key-model"), "{row:?}");
}

// ---- a layer that cannot be read -------------------------------------------------------

/// A file that does not parse yields no key rows of its own. Every key whose value that
/// layer could outrank reads not decided, with the layer named; nothing guesses past it.
#[test]
fn a_project_file_that_does_not_parse_leaves_the_user_keys_not_decided() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "user-model"}"#);
    rig.write(&rig.project_file(), "{ not json");
    let map = rig.seed_map(&[]);
    let user = reading(&map, "model", "user");
    assert_eq!(user.status, ReadingStatus::NotDecided, "{user:?}");
    assert_eq!(user.note.as_deref(), Some("project file could not be read"));
    assert!(
        map.readings.iter().all(|r| r.layer != "project"),
        "the broken file has no rows of its own: {:?}",
        map.readings
    );
}

/// A file that cannot be opened is the same case as one that does not parse.
#[test]
fn a_project_file_that_cannot_be_opened_leaves_the_user_keys_not_decided() {
    use std::os::unix::fs::PermissionsExt;
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "user-model"}"#);
    let file = rig.write(&rig.project_file(), r#"{"model": "project-model"}"#);
    fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).expect("mode");
    // Probe the effect, not the uid: root and ACLs can read a mode 000 file.
    if fs::File::open(&file).is_ok() {
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).expect("mode");
        usher_testkit::skip(
            "a_project_file_that_cannot_be_opened_leaves_the_user_keys_not_decided",
            "this process can read a mode 000 file, so there is nothing to test",
        );
        return;
    }
    let map = rig.seed_map(&[]);
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).expect("mode");
    let user = reading(&map, "model", "user");
    assert_eq!(user.status, ReadingStatus::NotDecided, "{user:?}");
    assert_eq!(user.note.as_deref(), Some("project file could not be read"));
    assert!(!format!("{map:?}").contains("project-model"));
}

/// A broken layer below the layer that wins changes nothing.
#[test]
fn a_broken_file_below_a_winning_layer_leaves_the_winner_in_effect() {
    let rig = Rig::new();
    rig.write(&rig.local_file(), r#"{"model": "local-model"}"#);
    rig.write(&rig.project_file(), "{ not json");
    let map = rig.seed_map(&[]);
    let local = reading(&map, "model", "local");
    assert_eq!(local.status, ReadingStatus::InEffect, "{local:?}");
    assert_eq!(local.value, text("local-model"));
}

/// With the version unknown there is one row per key name per present file, each unknown
/// and holding no value.
#[test]
fn an_unknown_version_gives_one_unknown_row_per_key_name_per_file() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"model": "canary-user-model", "theme": "canary-theme"}"#,
    );
    rig.write(&rig.project_file(), r#"{"model": "canary-project-model"}"#);
    let adapter = rig.adapter(KnowledgeFile::embedded().expect("embedded"));
    let context = rig.context_at(None, &[]);
    let listing = rig.listing(&adapter, &context);
    let map = adapter.map_settings(&listing, &context).expect("a map");
    let shape: Vec<(String, String, ReadingStatus)> = map
        .readings
        .iter()
        .map(|r| (r.key.clone(), r.layer.clone(), r.status))
        .collect();
    assert_eq!(
        shape,
        [
            (
                "model".to_string(),
                "project".to_string(),
                ReadingStatus::Unknown
            ),
            (
                "model".to_string(),
                "user".to_string(),
                ReadingStatus::Unknown
            ),
            (
                "theme".to_string(),
                "user".to_string(),
                ReadingStatus::Unknown
            ),
        ]
    );
    assert!(map.readings.iter().all(|r| r.value.is_none()));
    assert!(!format!("{map:?}").contains("canary"));
    for reading in &map.readings {
        let source = listing
            .sources
            .iter()
            .find(|s| s.layer == reading.layer)
            .expect("a source for the layer");
        let SourceStatus::Unknown { reason } = &source.status else {
            panic!("expected unknown: {source:?}");
        };
        assert_eq!(reading.note.as_ref(), Some(reason), "{reading:?}");
    }
    assert!(
        map.skipped_layers.is_empty(),
        "unknown sources are not skipped: {:?}",
        map.skipped_layers
    );
}

/// The server-managed cache and the global state file are not settings layers in the stack,
/// so they yield no rows.
#[test]
fn the_server_managed_cache_and_the_global_state_file_yield_no_rows() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"theme": "dark"}"#);
    rig.write(
        &rig.home().join(".claude/remote-settings.json"),
        r#"{"model": "canary-cache"}"#,
    );
    rig.write(
        &rig.home().join(".claude.json"),
        r#"{"model": "canary-state"}"#,
    );
    let adapter = rig.adapter(KnowledgeFile::embedded().expect("embedded"));
    let context = rig.context(&[]);
    let listing = rig.listing(&adapter, &context);
    let layers: Vec<&str> = listing.sources.iter().map(|s| s.layer.as_str()).collect();
    assert!(
        layers.contains(&"server-managed-cache") && layers.contains(&"global-state"),
        "the listing found both files: {layers:?}"
    );
    let map = adapter.map_settings(&listing, &context).expect("a map");
    assert_eq!(
        reading(&map, "theme", "user").status,
        ReadingStatus::ValueNotRead
    );
    assert!(
        map.readings
            .iter()
            .all(|r| r.layer != "server-managed-cache" && r.layer != "global-state"),
        "{:?}",
        map.readings
    );
    assert!(!format!("{map:?}").contains("canary"));
}

// ---- what the map does not observe -----------------------------------------------------

fn not_observed(map: &SettingsMap) -> Vec<NotObserved> {
    map.not_observed.clone()
}

/// The static items, the organization default model because `model` is listed, the
/// built-in classifier rules because an auto mode array is set, and the layers knowledge
/// reads that are not in the stack.
#[test]
fn the_not_observed_set_names_what_applies_to_this_environment() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"model": "m", "autoMode": {"allow": ["rule"]}}"#,
    );
    rig.write(&rig.home().join(".claude/remote-settings.json"), "{}");
    rig.write(&rig.home().join(".claude.json"), "{}");
    let map = rig.seed_map(&[]);
    let seen = not_observed(&map);
    for item in [
        NotObserved::CommandLine,
        NotObserved::EnvValues,
        NotObserved::ServerManagedValues,
        NotObserved::OrganizationDefaultModel,
        NotObserved::BuiltInClassifierRules,
        NotObserved::LayerMissingFromStack("server-managed-cache".to_string()),
        NotObserved::LayerMissingFromStack("global-state".to_string()),
    ] {
        assert!(seen.contains(&item), "{item:?} is missing from {seen:?}");
    }
}

/// With no auto mode array set in any file, the built-in classifier item is absent.
#[test]
fn no_classifier_item_is_named_when_no_auto_mode_array_is_set() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"model": "m"}"#);
    let map = rig.seed_map(&[]);
    let seen = not_observed(&map);
    assert!(
        seen.contains(&NotObserved::CommandLine),
        "the map names its blind spots: {seen:?}"
    );
    assert!(
        !seen.contains(&NotObserved::BuiltInClassifierRules),
        "{seen:?}"
    );
}

/// The organization default model is named only where `model` is a listed key.
#[test]
fn the_organization_default_model_item_follows_the_model_key_in_knowledge() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"fx": {"plain": "v"}}"#);
    let map = rig.fixture_map();
    let seen = not_observed(&map);
    assert!(seen.contains(&NotObserved::CommandLine), "{seen:?}");
    assert!(
        !seen.contains(&NotObserved::OrganizationDefaultModel),
        "the fixture lists no model key: {seen:?}"
    );
}

/// A managed first-wins source that a higher-ranked managed source hides is skipped, with
/// the knowledge reason and the harness's own reason named as not given.
#[test]
fn a_managed_source_shadowed_by_rank_is_a_skipped_layer() {
    let rig = Rig::new();
    let managed = rig.write(
        &rig.system.join("etc/claude-code/managed-settings.json"),
        r#"{"permissions": {"allow": ["Bash(managed)"]}}"#,
    );
    rig.write(&rig.user_file(), r#"{"model": "user-model"}"#);
    let adapter = rig.adapter(KnowledgeFile::embedded().expect("embedded"));
    let context = rig.context(&[]);
    let mut listing = rig.listing(&adapter, &context);
    let source = listing
        .sources
        .iter_mut()
        .find(|s| s.layer == "managed")
        .expect("a managed source");
    source.status = SourceStatus::Shadowed {
        by: PathBuf::from("/higher/ranked/managed-settings.json"),
    };
    let map = adapter.map_settings(&listing, &context).expect("a map");
    let skipped = map
        .skipped_layers
        .iter()
        .find(|s| s.layer == "managed")
        .unwrap_or_else(|| panic!("no skipped managed layer in {:?}", map.skipped_layers));
    assert!(
        skipped
            .reason
            .contains("/higher/ranked/managed-settings.json"),
        "{skipped:?}"
    );
    assert!(
        skipped
            .reason
            .contains("reason: not given (usher does not call /status)"),
        "{skipped:?}"
    );
    assert!(
        map.readings
            .iter()
            .filter(|r| r.source_path == managed)
            .all(|r| r.status != ReadingStatus::InEffect),
        "{:?}",
        map.readings
    );
    assert_eq!(
        reading(&map, "model", "user").status,
        ReadingStatus::InEffect
    );
}

// ---- order -----------------------------------------------------------------------------

/// Keys come back sorted by key path, segment by segment, whatever order the files list
/// them in. "a-b" sorts before "a.z" as a string and after it by segment, so a plain
/// string sort fails this.
#[test]
fn keys_come_back_sorted_by_path_segment_by_segment() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"b": 1, "a-b": 2, "a": {"z": 3}, "theme": "t"}"#,
    );
    let map = rig.seed_map(&[]);
    let order: Vec<&str> = map
        .readings
        .iter()
        .map(|r| r.key.as_str())
        .filter(|k| ["a-b", "a.z", "b"].contains(k))
        .collect();
    assert_eq!(order, ["a.z", "a-b", "b"], "{:?}", map.readings);
}

// ---- the managed layer -----------------------------------------------------------------

/// Managed is the top of the stack: a plain scalar set in managed and in local is decided
/// by managed, and local is shadowed with managed named.
#[test]
fn a_managed_scalar_wins_over_local_and_the_local_row_names_it() {
    let rig = Rig::new();
    let managed = rig.write(
        &rig.system.join("etc/claude-code/managed-settings.json"),
        r#"{"fx": {"plain": "managed-value"}}"#,
    );
    rig.write(&rig.local_file(), r#"{"fx": {"plain": "local-value"}}"#);
    let map = rig.fixture_map();
    let top = reading(&map, "fx.plain", "managed");
    assert_eq!(top.status, ReadingStatus::InEffect, "{top:?}");
    assert_eq!(top.value, text("managed-value"));
    assert_eq!(top.source_path, managed);
    let local = reading(&map, "fx.plain", "local");
    assert_eq!(local.status, ReadingStatus::Shadowed, "{local:?}");
    assert_eq!(
        local.note.as_deref(),
        Some(format!("shadowed by managed ({})", managed.display()).as_str())
    );
}

/// A list key set in managed and in user keeps both entries, each in effect.
#[test]
fn a_list_key_set_in_managed_and_user_keeps_both_entries() {
    let rig = Rig::new();
    rig.write(
        &rig.system.join("etc/claude-code/managed-settings.json"),
        r#"{"permissions": {"allow": ["Bash(managed)"]}}"#,
    );
    rig.write(
        &rig.user_file(),
        r#"{"permissions": {"allow": ["Bash(user)"]}}"#,
    );
    let map = rig.seed_map(&[]);
    let rows = readings(&map, "permissions.allow");
    let shape: Vec<(&str, ReadingStatus)> =
        rows.iter().map(|r| (r.layer.as_str(), r.status)).collect();
    assert_eq!(
        shape,
        [
            ("managed", ReadingStatus::InEffect),
            ("user", ReadingStatus::InEffect)
        ],
        "{rows:?}"
    );
}

// ---- a sensitive scalar ----------------------------------------------------------------

/// A sensitive scalar set in two layers: the map names the winner and never shows the value.
/// The highest layer is in effect with no value and "value not read (sensitive)", and the
/// lower layer is shadowed with no value.
#[test]
fn a_sensitive_scalar_names_its_winner_and_shows_no_value() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"fx": {"token": "canary-user-token"}}"#,
    );
    rig.write(
        &rig.project_file(),
        r#"{"fx": {"token": "canary-project-token"}}"#,
    );
    let map = rig.fixture_map();
    // The fixture's stack puts the user layer above the project layer.
    let user = reading(&map, "fx.token", "user");
    assert_eq!(user.status, ReadingStatus::InEffect, "{user:?}");
    assert_eq!(user.value, None);
    assert_eq!(user.note.as_deref(), Some("value not read (sensitive)"));
    let project = reading(&map, "fx.token", "project");
    assert_eq!(project.status, ReadingStatus::Shadowed, "{project:?}");
    assert_eq!(project.value, None);
    assert_eq!(
        project.note.as_deref(),
        Some(format!("shadowed by user ({})", rig.user_file().display()).as_str())
    );
    assert!(!format!("{map:?}").contains("canary"));
}

// ---- keys knowledge does not list ------------------------------------------------------

/// An unlisted key reads value not read in every layer that sets it, whether it is a
/// top-level key or a child of a listed parent: one row per layer, never not decided.
#[test]
fn an_unlisted_key_reads_value_not_read_in_every_layer_that_sets_it() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"theme": "canary-user", "autoMode": {"someNewArray": ["canary-new"]}}"#,
    );
    rig.write(
        &rig.project_file(),
        r#"{"theme": "canary-project", "autoMode": {"someNewArray": ["canary-new-2"]}}"#,
    );
    let map = rig.seed_map(&[]);
    for key in ["theme", "autoMode.someNewArray"] {
        let rows = readings(&map, key);
        let layers: Vec<&str> = rows.iter().map(|r| r.layer.as_str()).collect();
        assert_eq!(layers, ["project", "user"], "{key}: {rows:?}");
        for row in rows {
            assert_eq!(row.status, ReadingStatus::ValueNotRead, "{row:?}");
            assert_eq!(row.note.as_deref(), Some("key not marked readable"));
            assert_eq!(row.value, None);
        }
    }
    assert!(!format!("{map:?}").contains("canary"));
}

// ---- how an unlisted object is walked --------------------------------------------------

/// An unlisted object is walked to its leaf paths, names only.
#[test]
fn an_unlisted_object_is_walked_to_its_leaf_paths_and_names_only() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"a": {"z": "canary-leaf"}}"#);
    let map = rig.seed_map(&[]);
    let row = reading(&map, "a.z", "user");
    assert_eq!(row.status, ReadingStatus::ValueNotRead);
    assert_eq!(row.value, None);
    assert!(readings(&map, "a").is_empty(), "{:?}", map.readings);
    assert!(!format!("{map:?}").contains("canary"));
}

/// An array is a leaf: its elements are values, not keys. So is an empty object.
#[test]
fn an_array_and_an_empty_object_are_leaves() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"a": [1, 2], "e": {}, "n": {"list": ["canary-element"]}}"#,
    );
    let map = rig.seed_map(&[]);
    for key in ["a", "e", "n.list"] {
        let row = reading(&map, key, "user");
        assert_eq!(row.status, ReadingStatus::ValueNotRead, "{row:?}");
        assert_eq!(row.value, None);
    }
    assert!(
        map.readings.iter().all(|r| !r.key.starts_with("a.")),
        "an array is not walked: {:?}",
        map.readings
    );
    assert!(!format!("{map:?}").contains("canary"));
}

/// A segment that holds a dot, whitespace or a quote is printed as a JSON string, so it
/// stays one segment and never reads as two.
#[test]
fn a_segment_with_a_dot_whitespace_or_quote_is_printed_as_a_json_string() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"a": {"b.c": 1, "d e": 2, "f\"g": 3, "plain": 4}}"#,
    );
    let map = rig.seed_map(&[]);
    for key in ["a.\"b.c\"", "a.\"d e\"", "a.\"f\\\"g\"", "a.plain"] {
        reading(&map, key, "user");
    }
    assert!(readings(&map, "a.b.c").is_empty(), "{:?}", map.readings);
}

/// The walk stops at eight segments: an object still open there gives one row at that
/// path, with a note, and nothing deeper is listed.
#[test]
fn the_walk_stops_at_eight_segments_with_a_note() {
    let rig = Rig::new();
    let mut text = String::from("\"canary-deep\"");
    for depth in (1..=10).rev() {
        text = format!("{{\"s{depth}\": {text}}}");
    }
    rig.write(&rig.user_file(), &text);
    let map = rig.seed_map(&[]);
    let path = (1..=8)
        .map(|n| format!("s{n}"))
        .collect::<Vec<_>>()
        .join(".");
    let row = reading(&map, &path, "user");
    assert_eq!(row.status, ReadingStatus::ValueNotRead, "{row:?}");
    assert_eq!(
        row.note.as_deref(),
        Some("deeper keys not listed (depth limit 8)")
    );
    assert!(
        map.readings.iter().all(|r| r.key.split('.').count() <= 8),
        "{:?}",
        map.readings
    );
    assert!(!format!("{map:?}").contains("canary"));
}

/// A credential key stops the walk at any depth: its own name is one row, and nothing
/// under it is listed or read.
#[test]
fn a_credential_key_is_listed_by_name_at_any_depth_with_nothing_under_it() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"x": {"oauthAccount": {"canary-k": "canary-v"}}}"#,
    );
    let map = rig.seed_map(&[]);
    let row = reading(&map, "x.oauthAccount", "user");
    assert_eq!(row.status, ReadingStatus::ValueNotRead, "{row:?}");
    assert_eq!(row.note.as_deref(), Some("credential key; not opened"));
    assert_eq!(row.value, None);
    assert!(
        map.readings
            .iter()
            .all(|r| !r.key.starts_with("x.oauthAccount.")),
        "{:?}",
        map.readings
    );
    assert!(!format!("{map:?}").contains("canary"));
}

// ---- a layer the listing cannot place ---------------------------------------------------

/// A managed source the listing cannot decide (two managed files share a rank) whose file
/// does parse still blocks the layers below it: it might hold the key, so nothing below it
/// reads in effect.
#[test]
fn an_undecided_managed_source_above_a_user_value_leaves_the_user_value_not_decided() {
    let rig = Rig::new();
    rig.write(
        &rig.system.join("etc/claude-code/managed-settings.json"),
        r#"{"model": "managed-model"}"#,
    );
    rig.write(&rig.user_file(), r#"{"model": "user-model"}"#);
    let adapter = rig.adapter(KnowledgeFile::embedded().expect("embedded"));
    let context = rig.context(&[]);
    let mut listing = rig.listing(&adapter, &context);
    let managed = listing
        .sources
        .iter_mut()
        .find(|s| s.layer == "managed")
        .expect("a managed source");
    managed.status = SourceStatus::Unknown {
        reason: "first-wins not decided: two managed sources share rank 3".to_string(),
    };
    let map = adapter.map_settings(&listing, &context).expect("a map");
    let user = reading(&map, "model", "user");
    assert_eq!(user.status, ReadingStatus::NotDecided, "{user:?}");
    assert_eq!(
        user.note.as_deref(),
        Some("managed not placed: first-wins not decided: two managed sources share rank 3"),
        "{user:?}"
    );
    assert!(
        map.readings
            .iter()
            .all(|r| r.status != ReadingStatus::InEffect),
        "{:?}",
        map.readings
    );
}

// ---- the classifier item ---------------------------------------------------------------

/// The built-in classifier item applies only when an auto mode list key has an entry in
/// effect: not for the boolean own-rule key, and not for an array found where it is not read.
#[test]
fn the_classifier_item_needs_an_auto_mode_list_entry_in_effect() {
    let classifier = |files: &[(&str, &str)]| {
        let rig = Rig::new();
        for (layer, text) in files {
            let path = match *layer {
                "user" => rig.user_file(),
                _ => rig.project_file(),
            };
            rig.write(&path, text);
        }
        let map = rig.seed_map(&[]);
        assert!(
            map.not_observed.contains(&NotObserved::CommandLine),
            "the map names its blind spots: {:?}",
            map.not_observed
        );
        map.not_observed
            .contains(&NotObserved::BuiltInClassifierRules)
    };
    assert!(
        !classifier(&[("user", r#"{"autoMode": {"classifyAllShell": true}}"#)]),
        "the boolean own-rule key is not an array"
    );
    assert!(
        !classifier(&[("project", r#"{"autoMode": {"allow": ["rule"]}}"#)]),
        "an array found where it is not read is not in effect"
    );
    assert!(
        classifier(&[("user", r#"{"autoMode": {"allow": ["rule"]}}"#)]),
        "an array in effect names the item"
    );
}

// ---- a sensitive list ------------------------------------------------------------------

/// Lists concatenate, so each in-scope entry of a sensitive list is in effect, with its
/// position, no value, and the sensitive note. No "also set in" note goes on an entry,
/// because that note would compare secret values.
#[test]
fn each_entry_of_a_sensitive_list_is_in_effect_with_no_value() {
    let rig = Rig::new();
    rig.write(
        &rig.system.join("etc/claude-code/managed-settings.json"),
        r#"{"fx": {"secrets": ["canary-same"]}}"#,
    );
    rig.write(
        &rig.user_file(),
        r#"{"fx": {"secrets": ["canary-same", "canary-other"]}}"#,
    );
    let map = rig.fixture_map();
    let rows = readings(&map, "fx.secrets");
    let shape: Vec<(&str, Option<u32>, ReadingStatus)> = rows
        .iter()
        .map(|r| (r.layer.as_str(), r.entry_index, r.status))
        .collect();
    assert_eq!(
        shape,
        [
            ("managed", Some(0), ReadingStatus::InEffect),
            ("user", Some(0), ReadingStatus::InEffect),
            ("user", Some(1), ReadingStatus::InEffect),
        ],
        "{rows:?}"
    );
    for row in rows {
        assert_eq!(row.value, None, "{row:?}");
        assert_eq!(row.note.as_deref(), Some("value not read (sensitive)"));
    }
    assert!(!format!("{map:?}").contains("canary"));
}

// ---- lists under a layer that cannot be read -------------------------------------------

/// Lists concatenate, so an entry that was read applies whatever a broken layer holds: a
/// user list key under a project file that does not parse stays in effect.
#[test]
fn a_list_key_under_a_broken_file_keeps_its_entries_in_effect() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"permissions": {"allow": ["Bash(ls)"]}}"#,
    );
    rig.write(&rig.project_file(), "{ not json");
    let map = rig.seed_map(&[]);
    let row = reading(&map, "permissions.allow", "user");
    assert_eq!(row.status, ReadingStatus::InEffect, "{row:?}");
    assert_eq!(row.value, text("Bash(ls)"));
    assert_eq!(
        row.note.as_deref(),
        Some("project file could not be read; more entries may apply")
    );
}

// ---- two local files -------------------------------------------------------------------

/// Started in a subdirectory, the repository root's local file and the starting
/// directory's both belong to the local layer. Where both set a scalar the root's value
/// applies and the other reads shadowed with the root's file named; list entries from both
/// apply.
#[test]
fn two_local_files_decide_a_scalar_by_the_root_and_keep_both_lists() {
    let rig = Rig::new();
    let start = rig.repo.join("pkg");
    fs::create_dir_all(&start).expect("start dir");
    let root_file = rig.write(
        &rig.local_file(),
        r#"{"model": "root-model", "permissions": {"allow": ["Bash(root)"]}}"#,
    );
    let start_file = rig.write(
        &start.join(".claude/settings.local.json"),
        r#"{"model": "start-model", "permissions": {"allow": ["Bash(start)"]}}"#,
    );
    let adapter = rig.adapter(KnowledgeFile::embedded().expect("embedded"));
    let context = rig.context_in(&start, Some(VERSION), &[]);
    let listing = rig.listing(&adapter, &context);
    let map = adapter.map_settings(&listing, &context).expect("a map");
    let models = readings(&map, "model");
    assert_eq!(models.len(), 2, "{models:?}");
    let root = models
        .iter()
        .find(|r| r.source_path == root_file)
        .expect("a row for the root file");
    let started = models
        .iter()
        .find(|r| r.source_path == start_file)
        .expect("a row for the starting directory file");
    assert_eq!(root.status, ReadingStatus::InEffect, "{root:?}");
    assert_eq!(root.value, text("root-model"));
    assert_eq!(started.status, ReadingStatus::Shadowed, "{started:?}");
    assert_eq!(
        started.note.as_deref(),
        Some(format!("shadowed by local ({})", root_file.display()).as_str())
    );
    let allow = readings(&map, "permissions.allow");
    assert_eq!(allow.len(), 2, "{allow:?}");
    assert!(allow.iter().all(|r| r.status == ReadingStatus::InEffect));
}

// ---- scopes of a broken layer, and the texts the map pins ------------------------------

/// A layer outside a key's scopes cannot feed the key, so it cannot outrank it: a managed
/// file that does not parse leaves a user value in effect when the key is scoped to user
/// and local.
#[test]
fn a_broken_layer_outside_the_key_scopes_does_not_block_it() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"fx": {"scoped": "user-value"}}"#);
    rig.write(
        &rig.system.join("etc/claude-code/managed-settings.json"),
        "{ not json",
    );
    let map = rig.fixture_map();
    let row = reading(&map, "fx.scoped", "user");
    assert_eq!(row.status, ReadingStatus::InEffect, "{row:?}");
    assert_eq!(row.value, text("user-value"));
}

/// A top-level credential key is listed by name with nothing under it, as at any depth.
#[test]
fn a_top_level_credential_key_is_listed_by_name_with_nothing_under_it() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"oauthAccount": {"canary-k": "canary-v"}}"#,
    );
    let map = rig.seed_map(&[]);
    let row = reading(&map, "oauthAccount", "user");
    assert_eq!(row.status, ReadingStatus::ValueNotRead, "{row:?}");
    assert_eq!(row.note.as_deref(), Some("credential key; not opened"));
    assert_eq!(row.value, None);
    assert_eq!(map.readings.len(), 1, "{:?}", map.readings);
    assert!(!format!("{map:?}").contains("canary"));
}

/// A list key whose value is not an array reads not decided, with no value.
#[test]
fn a_list_key_that_is_not_an_array_is_not_decided_with_no_value() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"permissions": {"allow": "canary-not-a-list"}}"#,
    );
    let map = rig.seed_map(&[]);
    let row = reading(&map, "permissions.allow", "user");
    assert_eq!(row.status, ReadingStatus::NotDecided, "{row:?}");
    assert_eq!(row.note.as_deref(), Some("value is not an array"));
    assert_eq!(row.value, None);
    assert!(!format!("{map:?}").contains("canary"));
}

/// The child of a sensitive object reads value not read with its own note.
#[test]
fn a_sensitive_child_note_says_the_key_is_sensitive() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"env": {"FLEET_TOKEN": "canary-token"}}"#,
    );
    let map = rig.seed_map(&[]);
    let row = reading(&map, "env.FLEET_TOKEN", "user");
    assert_eq!(row.note.as_deref(), Some("key is sensitive"), "{row:?}");
}

// ---- a layer the listing cannot place, by the keys it sets ------------------------------

const TIE: &str = "first-wins not decided: two managed sources share rank 3";

/// The fixture map with the managed source marked unplaced, as a rank tie leaves it.
fn fixture_map_with_unplaced_managed(rig: &Rig) -> SettingsMap {
    let adapter = rig.adapter(KnowledgeFile::parse(FIXTURE).expect("fixture"));
    let context = rig.context(&[]);
    let mut listing = rig.listing(&adapter, &context);
    let managed = listing
        .sources
        .iter_mut()
        .find(|s| s.layer == "managed")
        .expect("a managed source");
    managed.status = SourceStatus::Unknown {
        reason: TIE.to_string(),
    };
    adapter.map_settings(&listing, &context).expect("a map")
}

/// An unplaced layer whose file was read blocks a key below it only when its object holds
/// that key's path. A member that sets only another key leaves the user value in effect.
#[test]
fn an_unplaced_managed_file_that_does_not_set_the_key_leaves_the_user_value_in_effect() {
    let rig = Rig::new();
    rig.write(
        &rig.system.join("etc/claude-code/managed-settings.json"),
        r#"{"theme": "x"}"#,
    );
    rig.write(&rig.user_file(), r#"{"model": "user-model"}"#);
    let adapter = rig.adapter(KnowledgeFile::embedded().expect("embedded"));
    let context = rig.context(&[]);
    let mut listing = rig.listing(&adapter, &context);
    let managed = listing
        .sources
        .iter_mut()
        .find(|s| s.layer == "managed")
        .expect("a managed source");
    managed.status = SourceStatus::Unknown {
        reason: TIE.to_string(),
    };
    let map = adapter.map_settings(&listing, &context).expect("a map");
    let user = reading(&map, "model", "user");
    assert_eq!(user.status, ReadingStatus::InEffect, "{user:?}");
    assert_eq!(user.value, text("user-model"));
}

/// The block follows the full key path: a member that sets the nested key blocks it, and a
/// member that sets only a sibling path does not.
#[test]
fn an_unplaced_file_blocks_a_nested_key_only_when_it_sets_that_path() {
    let sets = Rig::new();
    sets.write(
        &sets.system.join("etc/claude-code/managed-settings.json"),
        r#"{"fx": {"nested": {"leaf": "managed-leaf"}}}"#,
    );
    sets.write(
        &sets.user_file(),
        r#"{"fx": {"nested": {"leaf": "user-leaf"}}}"#,
    );
    let map = fixture_map_with_unplaced_managed(&sets);
    let user = reading(&map, "fx.nested.leaf", "user");
    assert_eq!(user.status, ReadingStatus::NotDecided, "{user:?}");
    assert_eq!(
        user.note.as_deref(),
        Some(format!("managed not placed: {TIE}").as_str())
    );
    let sibling = Rig::new();
    sibling.write(
        &sibling.system.join("etc/claude-code/managed-settings.json"),
        r#"{"fx": {"other": 1}}"#,
    );
    sibling.write(
        &sibling.user_file(),
        r#"{"fx": {"nested": {"leaf": "user-leaf"}}}"#,
    );
    let map = fixture_map_with_unplaced_managed(&sibling);
    let user = reading(&map, "fx.nested.leaf", "user");
    assert_eq!(user.status, ReadingStatus::InEffect, "{user:?}");
}

// ---- a file that could not be read leaves a trace -------------------------------------

/// The files the map names as not read: layer, path and reason.
fn files_not_read(map: &SettingsMap) -> Vec<(String, PathBuf, String)> {
    map.not_observed
        .iter()
        .filter_map(|item| match item {
            NotObserved::FileNotRead {
                layer,
                path,
                reason,
            } => Some((layer.clone(), path.clone(), reason.clone())),
            _ => None,
        })
        .collect()
}

/// A list key set only in a readable layer keeps its entries, and each says that more
/// entries may apply, since an unreadable layer in its scopes may add some. The broken
/// file is named as not read.
#[test]
fn a_list_under_a_broken_file_says_more_entries_may_apply_and_the_file_is_named() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"permissions": {"deny": ["Read(x)"]}}"#,
    );
    let broken = rig.write(&rig.project_file(), "{ not json");
    let map = rig.seed_map(&[]);
    let row = reading(&map, "permissions.deny", "user");
    assert_eq!(row.status, ReadingStatus::InEffect, "{row:?}");
    assert_eq!(
        row.note.as_deref(),
        Some("project file could not be read; more entries may apply")
    );
    let files = files_not_read(&map);
    assert_eq!(files.len(), 1, "{files:?}");
    assert_eq!(files[0].0, "project");
    assert_eq!(files[0].1, broken);
    assert!(
        files[0].2.starts_with("is not valid JSON at line"),
        "{files:?}"
    );
}

/// A repeated entry keeps its "also set in" note (on the lower layer's copy) and gains the
/// unreadable-layer note, joined with a semicolon.
#[test]
fn a_repeated_list_entry_joins_the_also_set_note_and_the_unreadable_note() {
    let rig = Rig::new();
    rig.write(
        &rig.user_file(),
        r#"{"permissions": {"deny": ["Read(x)"]}}"#,
    );
    rig.write(
        &rig.local_file(),
        r#"{"permissions": {"deny": ["Read(x)"]}}"#,
    );
    rig.write(&rig.project_file(), "{ not json");
    let map = rig.seed_map(&[]);
    let rows = readings(&map, "permissions.deny");
    let user = rows
        .iter()
        .find(|r| r.layer == "user")
        .expect("a user entry");
    assert_eq!(
        user.note.as_deref(),
        Some("also set in local; project file could not be read; more entries may apply"),
        "{rows:?}"
    );
}

/// A key set only in a file that cannot be read has no row, and the file is named.
#[test]
fn a_scalar_set_only_in_a_broken_file_has_no_row_and_the_file_is_named() {
    let rig = Rig::new();
    let broken = rig.write(&rig.project_file(), r#"{"model": "canary-scalar""#);
    let map = rig.seed_map(&[]);
    assert!(
        map.readings.iter().all(|r| r.key != "model"),
        "{:?}",
        map.readings
    );
    let files = files_not_read(&map);
    assert_eq!(files.len(), 1, "{files:?}");
    assert_eq!((files[0].0.as_str(), &files[0].1), ("project", &broken));
    assert!(!format!("{map:?}").contains("canary-scalar"));
}

/// A key the vendor gives its own merge rule stays not decided, and the note says an
/// unreadable layer was in its scopes.
#[test]
fn an_own_rule_key_under_a_broken_file_joins_the_note() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"fallbackModel": "user-fallback"}"#);
    rig.write(&rig.project_file(), "{ not json");
    let map = rig.seed_map(&[]);
    let row = reading(&map, "fallbackModel", "user");
    assert_eq!(row.status, ReadingStatus::NotDecided, "{row:?}");
    assert_eq!(
        row.note.as_deref(),
        Some("own merge rule; project file could not be read")
    );
}

/// A broken layer outside a list key's scopes adds no note to it, and the file is still
/// named.
#[test]
fn a_broken_layer_outside_a_lists_scopes_adds_no_note_but_is_still_named() {
    let rig = Rig::new();
    rig.write(&rig.user_file(), r#"{"autoMode": {"allow": ["rule"]}}"#);
    rig.write(&rig.project_file(), "{ not json");
    let map = rig.seed_map(&[]);
    let row = reading(&map, "autoMode.allow", "user");
    assert_eq!(row.status, ReadingStatus::InEffect, "{row:?}");
    assert_eq!(row.note, None, "{row:?}");
    assert_eq!(files_not_read(&map).len(), 1, "{:?}", map.not_observed);
}

/// Each cause has its own reason: a file that cannot be opened, one that is not valid
/// JSON, and one that is not an object. None quotes a byte of the file.
#[test]
fn the_three_causes_give_the_three_reasons_and_quote_nothing() {
    use std::os::unix::fs::PermissionsExt;
    let rig = Rig::new();
    let user = rig.write(&rig.user_file(), r#"{"model": "canary-unopened"}"#);
    rig.write(&rig.project_file(), "{ canary-not-json");
    rig.write(&rig.local_file(), r#"["canary-array"]"#);
    fs::set_permissions(&user, fs::Permissions::from_mode(0o000)).expect("mode");
    let can_open = fs::File::open(&user).is_ok();
    let map = rig.seed_map(&[]);
    fs::set_permissions(&user, fs::Permissions::from_mode(0o644)).expect("mode");
    let files = files_not_read(&map);
    let reason = |layer: &str| {
        files
            .iter()
            .find(|f| f.0 == layer)
            .map(|f| f.2.clone())
            .unwrap_or_else(|| panic!("no {layer} file named in {files:?}"))
    };
    assert!(
        reason("project").starts_with("is not valid JSON at line 1 column"),
        "{files:?}"
    );
    assert_eq!(reason("local"), "is not a JSON object");
    if can_open {
        usher_testkit::skip(
            "the_three_causes_give_the_three_reasons_and_quote_nothing",
            "this process can read a mode 000 file, so the open case is not tested",
        );
    } else {
        assert!(
            reason("user").starts_with("cannot be opened: "),
            "{files:?}"
        );
    }
    assert!(!format!("{map:?}").contains("canary"), "{map:?}");
}
