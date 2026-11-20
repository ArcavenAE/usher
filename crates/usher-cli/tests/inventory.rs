//! Story 1.5, PR C: what `usher inventory` prints and records. Runs against a scratch
//! home and a scratch system root; the real harness homes are guarded.

use std::cell::RefCell;
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use usher_cli::{
    InventoryEnv, LaunchEnvironment, inventory, inventory_platform, inventory_system_root,
    inventory_with, run,
};
use usher_core::SystemRoot;
use usher_knowledge::{Knowledge, KnowledgeFile, Platform, TrustedKeys};
use usher_store::Store;
use usher_testkit::{NamedHome, RealHomeGuard, RealHomes, ScratchHome, Signer};

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
        Rig {
            scratch,
            system,
            repo,
        }
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

    fn run(&self, env: &InventoryEnv) -> String {
        inventory(env, SystemRoot::at(self.system.clone())).expect("an inventory")
    }

    fn write(&self, path: &Path, text: &str) -> PathBuf {
        fs::create_dir_all(path.parent().expect("a parent")).expect("dirs");
        fs::write(path, text).expect("file");
        path.to_path_buf()
    }

    /// Installs a native launcher link for `version`.
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

    fn user(&self, text: &str) -> PathBuf {
        self.write(&self.scratch.home().join(".claude/settings.json"), text)
    }
}

/// The seed's launcher item. An update replaces the whole file, so a fixture that wants
/// the version read from the launcher link carries the item that says how.
const LAUNCHER: &str = r#"
[[item]]
id = "claude-code.native-launcher"
harness = "claude-code"
layer = "slow"
class = "paths"
versions = ">=2.1.317, <3.0.0"
summary = "The native installer's launcher is a symlink to a file named by its version."
reference = { kind = "vendor-docs", locator = "https://code.claude.com/docs/en/setup" }
last_verified = 2026-11-20
version_from = { launcher = "${HOME}/.local/bin/claude", rule = "symlink-target-last-component", layout = "${HOME}/.local/share/claude/versions/${VERSION}" }
"#;

/// Fixture knowledge goes through the signed update path, never in unsigned.
fn signed(text: &str) -> Knowledge {
    let signer = Signer::new();
    let trusted = TrustedKeys::for_tests(&signer.public_key_base64(), None).expect("keys");
    let mut knowledge = Knowledge::embedded().expect("embedded");
    knowledge
        .update(
            text.as_bytes(),
            &signer.sign(text.as_bytes(), "cli fixture"),
            &trusted,
        )
        .expect("a signed update is taken");
    knowledge
}

fn line_with<'a>(output: &'a str, needle: &str) -> &'a str {
    output
        .lines()
        .find(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("no line holds `{needle}` in:\n{output}"))
}

#[test]
fn the_first_line_stamps_the_harness_version_and_the_knowledge_version() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let out = rig.run(&rig.env());
    let first = out.lines().next().expect("a first line");
    let knowledge = KnowledgeFile::embedded()
        .expect("embedded")
        .version()
        .to_string();
    assert!(first.starts_with("Claude Code 2.1.317"), "{first}");
    assert!(first.contains("knowledge v"), "{first}");
    assert_eq!(
        first,
        format!("Claude Code 2.1.317, {knowledge}: every status below assumes these")
    );
}

#[test]
fn an_unknown_version_is_stamped_unknown_with_its_reason_and_statuses_read_unknown() {
    let rig = Rig::new();
    let user = rig.user(r#"{"model":"m"}"#);
    let out = rig.run(&rig.env());
    let first = out.lines().next().expect("a first line");
    let knowledge = KnowledgeFile::embedded()
        .expect("embedded")
        .version()
        .to_string();
    assert!(
        first.starts_with("Claude Code version unknown ("),
        "{first}"
    );
    assert!(first.contains("knowledge for it not known"), "{first}");
    assert!(first.contains(&format!("({knowledge} loaded)")), "{first}");
    assert!(
        first.ends_with("every status below reads unknown"),
        "{first}"
    );
    let row = line_with(&out, &user.display().to_string());
    assert!(row.contains("unknown"), "{row}");
}

#[test]
fn a_present_source_prints_its_layer_path_status_and_key_names_never_a_value() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let user = rig.user(r#"{"model":"super-secret-model","permissions":{}}"#);
    let out = rig.run(&rig.env());
    let row = line_with(&out, &user.display().to_string());
    assert!(row.contains("user") && row.contains("in effect"), "{row}");
    let keys = line_with(&out, "keys:");
    assert!(
        keys.contains("model") && keys.contains("permissions"),
        "{keys}"
    );
    let knowledge = KnowledgeFile::embedded()
        .expect("embedded")
        .version()
        .to_string();
    assert_eq!(
        keys.trim(),
        format!(
            "keys: model, permissions (names only; not classified: {knowledge} lists no keys for claude-code 2.1.317)"
        )
    );
    assert!(!out.contains("super-secret-model"), "no value is printed");
}

#[test]
fn a_source_the_version_does_not_read_prints_unread_with_the_knowledge_item() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let old = rig.write(&rig.scratch.home().join(".claude-old.json"), "{}");
    let knowledge = signed(&format!(
        "{}{LAUNCHER}",
        r#"
schema_version = 5
sequence = 9

[[item]]
id = "claude-code.old-settings"
harness = "claude-code"
layer = "slow"
class = "sources"
versions = ">=2.1.317, <3.0.0"
summary = "A place this version no longer reads."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/old" }
last_verified = 2026-11-20
source = { path = "${HOME}/.claude-old.json", layer = "user", read = "not-read", rank = 1, always_present = false }
"#
    ));
    let out = inventory_with(&rig.env(), SystemRoot::at(rig.system.clone()), &knowledge)
        .expect("an inventory");
    let row = line_with(&out, &old.display().to_string());
    assert!(
        row.trim_end()
            .ends_with("unread (knowledge item claude-code.old-settings)"),
        "{row}"
    );
}

#[test]
fn an_invalid_file_prints_unknown_with_its_position_and_category_and_no_content() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let user = rig.user(r#"{"token": "sk-live-secret-123" oops}"#);
    let out = rig.run(&rig.env());
    let row = line_with(&out, &user.display().to_string());
    assert!(
        row.contains("unknown (not valid JSON at line 1, column ") && row.contains("syntax)"),
        "{row}"
    );
    assert!(!out.contains("sk-live-secret-123"), "{out}");
    assert!(!out.contains("oops"), "{out}");
}

#[test]
fn a_shadowed_managed_source_names_the_winner() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let managed = |id: &str, path: &str, rank: u32| {
        format!(
            r#"
[[item]]
id = "{id}"
harness = "claude-code"
layer = "slow"
class = "sources"
versions = ">=2.1.317, <3.0.0"
summary = "A managed settings file."
reference = {{ kind = "vendor-docs", locator = "https://example.test/docs/managed" }}
last_verified = 2026-11-20
source = {{ path = "{path}", layer = "managed", read = "first-wins", rank = {rank}, always_present = false, platforms = ["linux"] }}
"#
        )
    };
    let knowledge = signed(&format!(
        "schema_version = 5\nsequence = 9\n{}{}{LAUNCHER}",
        managed("demo.high", "/etc/claude-code/high.json", 2),
        managed("demo.low", "/etc/claude-code/low.json", 3),
    ));
    let winner = rig.write(
        &rig.system.join("etc/claude-code/high.json"),
        r#"{"permissions":{}}"#,
    );
    let loser = rig.write(
        &rig.system.join("etc/claude-code/low.json"),
        r#"{"model":"m"}"#,
    );
    let out = inventory_with(&rig.env(), SystemRoot::at(rig.system.clone()), &knowledge)
        .expect("an inventory");
    let row = line_with(&out, &loser.display().to_string());
    assert!(
        row.contains(&format!("shadowed by {}", winner.display())),
        "{row}"
    );
}

#[test]
fn an_absent_always_present_layer_prints_one_no_file_line_with_its_path() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let out = rig.run(&rig.env());
    let user = rig.scratch.home().join(".claude/settings.json");
    assert!(
        out.contains(&format!("no file: user settings ({})", user.display())),
        "{out}"
    );
    assert!(!out.contains("no file: managed"), "{out}");
}

#[test]
fn a_relative_claude_config_dir_is_not_checked_and_says_so() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let mut env = rig.env();
    env.launch = LaunchEnvironment::from_pairs(&[("CLAUDE_CONFIG_DIR", "relative/claude")]);
    let out = rig.run(&env);
    assert!(
        out.contains("not checked: user settings (CLAUDE_CONFIG_DIR is not absolute)"),
        "{out}"
    );
}

#[test]
fn an_absolute_claude_config_dir_moves_the_user_layer() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let moved = rig.write(&rig.scratch.root().join("cfg/settings.json"), "{}");
    let mut env = rig.env();
    env.launch = LaunchEnvironment::from_pairs(&[(
        "CLAUDE_CONFIG_DIR",
        &rig.scratch.root().join("cfg").display().to_string(),
    )]);
    let out = rig.run(&env);
    assert!(out.contains(&moved.display().to_string()), "{out}");
    let elsewhere = rig.scratch.home().join(".claude/settings.json");
    assert!(!out.contains(&elsewhere.display().to_string()), "{out}");
}

#[test]
fn an_empty_claude_config_dir_means_the_default() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let user = rig.user("{}");
    let mut env = rig.env();
    env.launch = LaunchEnvironment::from_pairs(&[("CLAUDE_CONFIG_DIR", "")]);
    let out = rig.run(&env);
    assert!(out.contains(&user.display().to_string()), "{out}");
}

#[test]
fn every_run_is_recorded_as_a_new_row_with_derived_state_only() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let user = rig.user(r#"{"model":"super-secret-model"}"#);
    let env = rig.env();
    rig.run(&env);
    let first_run = Store::open(&env.state_dir)
        .expect("the store opens")
        .runs()
        .expect("runs");
    assert_eq!(first_run.len(), 1);
    let first_sources = Store::open(&env.state_dir)
        .expect("the store opens")
        .sources_of(first_run[0].id)
        .expect("sources");
    rig.run(&env);
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 2, "a second run inserts, never rewrites");
    assert_eq!(runs[0], first_run[0], "the first run's row is unchanged");
    assert_eq!(
        store.sources_of(runs[0].id).expect("sources"),
        first_sources,
        "the first run's sources are unchanged"
    );
    assert_eq!(runs[0].recorded_at, NOW);
    assert_eq!(runs[0].harness, "claude-code");
    assert_eq!(runs[0].harness_version.as_deref(), Some("2.1.317"));
    let knowledge = KnowledgeFile::embedded().expect("embedded");
    assert_eq!(
        runs[0].knowledge_schema,
        knowledge.version().schema_version()
    );
    assert_eq!(runs[0].knowledge_sequence, knowledge.version().sequence());
    assert_eq!(
        runs[0].environment_key,
        format!("host-1/alex/claude-code/{}", rig.repo.display()),
        "the launch directory, and no entry for a variable that is unset"
    );
    let sources = store.sources_of(runs[0].id).expect("sources");
    let row = sources
        .iter()
        .find(|s| s.path == user.display().to_string())
        .expect("the user source is recorded");
    assert!(row.present);
    assert_eq!(row.layer, "user");
    assert_eq!(row.key_names, ["model"]);
    assert!(!format!("{sources:?}").contains("super-secret-model"));
}

#[test]
fn an_absent_layer_is_recorded_as_checked_and_absent() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let env = rig.env();
    rig.run(&env);
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 1);
    let sources = store.sources_of(runs[0].id).expect("sources");
    let user = sources
        .iter()
        .find(|s| s.layer == "user")
        .expect("the absent user layer is recorded");
    assert!(!user.present);
    assert!(
        user.path.ends_with(".claude/settings.json"),
        "{}",
        user.path
    );
}

#[test]
fn a_run_changes_no_harness_home_and_no_managed_directory() {
    let rig = Rig::new();
    rig.install("2.1.317");
    rig.user("{}");
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
    let out = rig.run(&rig.env());
    guard.assert_unchanged();
    assert!(
        out.contains("etc/claude-code/managed-settings.json"),
        "the run did look at the watched managed directory:\n{out}"
    );
}

#[test]
fn the_verb_reads_sources_for_the_hosts_platform() {
    #[cfg(target_os = "macos")]
    assert_eq!(inventory_platform(), Platform::Macos);
    #[cfg(target_os = "linux")]
    assert_eq!(inventory_platform(), Platform::Linux);
    #[cfg(target_os = "windows")]
    assert_eq!(inventory_platform(), Platform::Windows);
}

#[test]
fn an_unknown_version_is_recorded_with_no_harness_version() {
    let rig = Rig::new();
    let env = rig.env();
    rig.run(&env);
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].harness_version, None);
}

#[test]
fn the_verb_builds_its_adapter_with_the_host_root() {
    assert_eq!(inventory_system_root(), SystemRoot::host());
}

#[test]
fn run_refuses_an_argument_to_inventory_and_an_unknown_command() {
    let rig = Rig::new();
    let env = rig.env();
    let err = run(&["inventory".to_string(), "extra".to_string()], &env)
        .expect_err("inventory takes no argument");
    assert_eq!(err.step(), "run inventory");
    assert!(err.status().contains("extra"), "{}", err.status());
    assert!(
        err.status().contains("usher inventory takes no arguments"),
        "{}",
        err.status()
    );
    assert_eq!(err.next_action(), "run `usher inventory`");
    let err = run(&["whatever".to_string()], &env).expect_err("an unknown command");
    assert_eq!(err.step(), "select command");
    assert!(
        err.status().contains("`whatever` is not a usher command"),
        "{}",
        err.status()
    );
    assert_eq!(
        err.next_action(),
        "run `usher inventory` or `usher --version`"
    );
}

#[test]
fn a_set_claude_config_dir_keys_the_environment_with_its_value_and_an_empty_one_is_dropped() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let cfg = rig.scratch.root().join("cfg");
    let mut env = rig.env();
    env.launch =
        LaunchEnvironment::from_pairs(&[("CLAUDE_CONFIG_DIR", &cfg.display().to_string())]);
    rig.run(&env);
    let mut empty_env = rig.env();
    empty_env.launch = LaunchEnvironment::from_pairs(&[("CLAUDE_CONFIG_DIR", "")]);
    rig.run(&empty_env);
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 2);
    let dir = rig.repo.display();
    assert_eq!(
        runs[0].environment_key,
        format!(
            "host-1/alex/claude-code/{dir}?CLAUDE_CONFIG_DIR={}",
            cfg.display()
        )
    );
    assert_eq!(
        runs[1].environment_key,
        format!("host-1/alex/claude-code/{dir}"),
        "an empty variable is dropped like an unset one"
    );
    let default_user = rig.scratch.home().join(".claude/settings.json");
    let sources = store.sources_of(runs[1].id).expect("sources");
    assert!(
        sources
            .iter()
            .any(|s| s.layer == "user" && s.path == default_user.display().to_string()),
        "an empty variable still resolves the user layer to the default"
    );
}

#[test]
fn every_frozen_source_in_the_all_sources_row_is_listed_with_one_status() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let paths = [
        rig.write(
            &rig.system.join("etc/claude-code/managed-settings.json"),
            r#"{"permissions":{}}"#,
        ),
        rig.user("{}"),
        rig.write(&rig.repo.join(".claude/settings.json"), "{}"),
        rig.write(&rig.repo.join(".claude/settings.local.json"), "{}"),
        rig.write(
            &rig.scratch.home().join(".claude.json"),
            r#"{"projects":{}}"#,
        ),
        rig.write(
            &rig.scratch.home().join(".claude/remote-settings.json"),
            "{}",
        ),
    ];
    let out = rig.run(&rig.env());
    for path in &paths {
        let row = line_with(&out, &path.display().to_string());
        let statuses = ["in effect", "shadowed by", "unread (", "unknown ("];
        assert_eq!(
            statuses.iter().filter(|s| row.contains(*s)).count(),
            1,
            "one status in {row}"
        );
    }
}

#[test]
fn the_oauth_account_key_in_the_global_state_file_is_never_opened() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let global = rig.write(
        &rig.scratch.home().join(".claude.json"),
        r#"{"oauthAccount":{"emailAddress":"alex@example.test"},"projects":{}}"#,
    );
    let out = rig.run(&rig.env());
    assert!(out.contains(&global.display().to_string()), "{out}");
    assert!(!out.contains("alex@example.test"), "{out}");
    assert!(
        !out.contains("emailAddress"),
        "a nested key name is not read either: {out}"
    );
    let store = Store::open(&rig.env().state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 1);
    let sources = store.sources_of(runs[0].id).expect("sources");
    assert!(!format!("{sources:?}").contains("alex@example.test"));
}

#[test]
fn a_version_outside_every_range_stamps_unknown_and_every_status_reads_unknown() {
    let rig = Rig::new();
    rig.install("3.0.0");
    let managed = rig.write(
        &rig.system.join("etc/claude-code/managed-settings.json"),
        r#"{"permissions":{}}"#,
    );
    let user = rig.user("{}");
    let project = rig.write(&rig.repo.join(".claude/settings.json"), "{}");
    let out = rig.run(&rig.env());
    let first = out.lines().next().expect("a first line");
    assert!(first.contains("knowledge for it not known"), "{first}");
    for path in [&managed, &user, &project] {
        let row = line_with(&out, &path.display().to_string());
        assert!(row.contains("unknown ("), "{row}");
    }
}

/// AD-17: one environment is never keyed two ways, so the harness version, which can be
/// unknown or can change, is not part of the key.
#[test]
fn a_known_and_an_unknown_version_run_in_one_environment_record_the_same_key() {
    let rig = Rig::new();
    let cfg = rig.scratch.root().join("cfg");
    let mut env = rig.env();
    env.launch =
        LaunchEnvironment::from_pairs(&[("CLAUDE_CONFIG_DIR", &cfg.display().to_string())]);
    rig.install("2.1.317");
    rig.run(&env);
    fs::remove_file(rig.scratch.home().join(".local/bin/claude")).expect("remove the launcher");
    rig.run(&env);
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].harness_version.as_deref(), Some("2.1.317"));
    assert_eq!(runs[1].harness_version, None);
    assert_eq!(runs[0].environment_key, runs[1].environment_key);
    assert_eq!(
        runs[1].environment_key,
        format!(
            "host-1/alex/claude-code/{}?CLAUDE_CONFIG_DIR={}",
            rig.repo.display(),
            cfg.display()
        ),
        "the set variable is in the key whether or not the version is known"
    );
}

/// A signed knowledge update that names another variable changes no key where that
/// variable is unset.
#[test]
fn a_knowledge_update_naming_an_unset_variable_changes_no_key() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let env = rig.env();
    rig.run(&env);
    let knowledge = signed(
        r#"
schema_version = 5
sequence = 99

[[item]]
id = "claude-code.config-dir-variable"
harness = "claude-code"
layer = "slow"
class = "paths"
versions = ">=2.1.317, <3.0.0"
summary = "CLAUDE_CONFIG_DIR relocates the user config home."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/env" }
last_verified = 2026-11-20
variable = { name = "CLAUDE_CONFIG_DIR", binds = "${CONFIG_DIR}", default = "${HOME}/.claude" }

[[item]]
id = "claude-code.other-variable"
harness = "claude-code"
layer = "slow"
class = "paths"
versions = ">=3.0.0, <4.0.0"
summary = "A variable a later range reads."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/env" }
last_verified = 2026-11-20
variable = { name = "CLAUDE_OTHER_DIR", binds = "${CONFIG_DIR}", default = "${HOME}/.other" }
"#,
    );
    inventory_with(&env, SystemRoot::at(rig.system.clone()), &knowledge).expect("an inventory");
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].environment_key, runs[1].environment_key);
    assert_ne!(runs[0].knowledge_sequence, runs[1].knowledge_sequence);
}

/// The verb asks the launch environment for every name knowledge lists, across all
/// ranges, so a variable that only a later range names still enters the key of a run at a
/// known version. A CLI that kept its own list of one name could not tell.
#[test]
fn a_variable_named_only_by_another_range_enters_the_key_of_a_known_version_run() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let mut env = rig.env();
    env.launch = LaunchEnvironment::from_pairs(&[("CLAUDE_OTHER_DIR", "/elsewhere")]);
    let knowledge = signed(&format!(
        "{}{LAUNCHER}",
        r#"
schema_version = 5
sequence = 99

[[item]]
id = "claude-code.config-dir-variable"
harness = "claude-code"
layer = "slow"
class = "paths"
versions = ">=2.1.317, <3.0.0"
summary = "CLAUDE_CONFIG_DIR relocates the user config home."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/env" }
last_verified = 2026-11-20
variable = { name = "CLAUDE_CONFIG_DIR", binds = "${CONFIG_DIR}", default = "${HOME}/.claude" }

[[item]]
id = "claude-code.other-variable"
harness = "claude-code"
layer = "slow"
class = "paths"
versions = ">=3.0.0, <4.0.0"
summary = "A variable a later range reads."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/env" }
last_verified = 2026-11-20
variable = { name = "CLAUDE_OTHER_DIR", binds = "${CONFIG_DIR}", default = "${HOME}/.other" }
"#
    ));
    inventory_with(&env, SystemRoot::at(rig.system.clone()), &knowledge).expect("an inventory");
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].harness_version.as_deref(), Some("2.1.317"));
    assert_eq!(
        runs[0].environment_key,
        format!(
            "host-1/alex/claude-code/{}?CLAUDE_OTHER_DIR=/elsewhere",
            rig.repo.display()
        )
    );
}

#[test]
fn from_process_keeps_only_the_named_variables_that_are_set() {
    // CARGO_PKG_NAME is set by cargo for every test process; PATH is set but not asked for.
    let launch = LaunchEnvironment::from_process(&["CARGO_PKG_NAME", "USHER_SURELY_NOT_SET_1"])
        .expect("every named variable that is set holds valid text");
    assert_eq!(launch.get("CARGO_PKG_NAME"), Some("usher-cli"));
    assert_eq!(launch.get("USHER_SURELY_NOT_SET_1"), None);
    assert_eq!(
        launch.get("PATH"),
        None,
        "a variable nobody asked for is not held"
    );
}

#[test]
fn from_pairs_holds_exactly_the_pairs_it_is_given() {
    let launch = LaunchEnvironment::from_pairs(&[("A", "1"), ("B", "")]);
    assert_eq!(launch.get("A"), Some("1"));
    assert_eq!(launch.get("B"), Some(""));
    assert_eq!(launch.get("C"), None);
}

#[test]
fn from_lookup_asks_for_each_named_variable_and_nothing_else() {
    let asked = RefCell::new(Vec::new());
    let launch = LaunchEnvironment::from_lookup(&["A", "B", "C"], |name| {
        asked.borrow_mut().push(name.to_string());
        match name {
            "A" => Some(OsString::from("1")),
            "B" => Some(OsString::new()),
            _ => None,
        }
    })
    .expect("valid text and unset variables are not an error");
    assert_eq!(*asked.borrow(), ["A", "B", "C"]);
    assert_eq!(launch.get("A"), Some("1"));
    assert_eq!(launch.get("B"), Some(""));
    assert_eq!(launch.get("C"), None);
    assert_eq!(launch.get("D"), None);
}

#[test]
fn a_named_variable_holding_invalid_utf8_is_refused_by_name_and_never_by_value() {
    let bytes = OsString::from_vec(b"secret-\xff-tail".to_vec());
    let err = LaunchEnvironment::from_lookup(&["CLAUDE_CONFIG_DIR"], |name| {
        (name == "CLAUDE_CONFIG_DIR").then(|| bytes.clone())
    })
    .expect_err("a value that is not valid UTF-8 cannot key an environment");
    let shown = format!("{err} {err:?}");
    assert!(shown.contains("CLAUDE_CONFIG_DIR"), "{shown}");
    for field in [err.step(), err.check(), err.status(), err.next_action()] {
        assert!(!field.is_empty(), "all four fields are filled: {shown}");
        assert!(
            !field.contains("secret") && !field.contains("tail") && !field.contains('\u{FFFD}'),
            "no part of the value is shown: {field}"
        );
    }
    assert!(
        !shown.contains("secret") && !shown.contains("tail"),
        "{shown}"
    );
}

#[test]
fn a_variable_nobody_asked_for_may_hold_invalid_utf8() {
    let launch = LaunchEnvironment::from_lookup(&["A"], |name| match name {
        "A" => Some(OsString::from("1")),
        _ => Some(OsString::from_vec(vec![0xff])),
    })
    .expect("only the named variables are read");
    assert_eq!(launch.get("A"), Some("1"));
}

/// The directory comes from the binding in range for the installed version, not from the
/// first binder in the file: two ranges with different defaults give different directories.
#[test]
fn the_config_directory_default_follows_the_binding_in_range_for_the_version() {
    let rig = Rig::new();
    rig.install("2.2.5");
    let range = |id: &str, versions: &str, dir: &str| {
        format!(
            r#"
[[item]]
id = "{id}-variable"
harness = "claude-code"
layer = "slow"
class = "paths"
versions = "{versions}"
summary = "CLAUDE_CONFIG_DIR relocates the user config home."
reference = {{ kind = "vendor-docs", locator = "https://example.test/docs/env" }}
last_verified = 2026-11-20
variable = {{ name = "CLAUDE_CONFIG_DIR", binds = "${{CONFIG_DIR}}", default = "${{HOME}}/{dir}" }}

[[item]]
id = "{id}-user"
harness = "claude-code"
layer = "slow"
class = "sources"
versions = "{versions}"
summary = "The user settings file."
reference = {{ kind = "vendor-docs", locator = "https://example.test/docs/settings" }}
last_verified = 2026-11-20
source = {{ path = "${{CONFIG_DIR}}/settings.json", layer = "user", read = "read", rank = 1, always_present = true }}
"#
        )
    };
    let knowledge = signed(&format!(
        "schema_version = 5\nsequence = 9\n{}{}{}{LAUNCHER}",
        range("claude-code.early", ">=2.1.317, <2.2.0", ".cfg-early"),
        range("claude-code.late", ">=2.2.0, <2.3.0", ".cfg-late"),
        range("claude-code.last", ">=2.3.0, <3.0.0", ".cfg-last"),
    ));
    let out = inventory_with(&rig.env(), SystemRoot::at(rig.system.clone()), &knowledge)
        .expect("an inventory");
    let late = rig.scratch.home().join(".cfg-late/settings.json");
    let early = rig.scratch.home().join(".cfg-early/settings.json");
    assert!(
        out.contains(&format!("no file: user settings ({})", late.display())),
        "{out}"
    );
    assert!(!out.contains(&early.display().to_string()), "{out}");
}

/// A stored row carries the item, the size and the modified time of what was found, and
/// an absent row carries the item that states it.
#[test]
fn a_recorded_row_holds_the_item_the_size_and_the_modified_time() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let content = r#"{"model":"m"}"#;
    let user = rig.user(content);
    let file = fs::File::options().write(true).open(&user).expect("open");
    file.set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_795_089_600))
        .expect("set the modified time");
    drop(file);
    let env = rig.env();
    rig.run(&env);
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    let sources = store.sources_of(runs[0].id).expect("sources");
    let present = sources
        .iter()
        .find(|s| s.path == user.display().to_string())
        .expect("the user source is recorded");
    assert_eq!(present.item_id, "claude-code.source-user-settings");
    assert_eq!(present.size, Some(content.len() as u64));
    assert_eq!(present.modified.as_deref(), Some("2026-11-19T12:00:00Z"));
    let absent = sources
        .iter()
        .find(|s| s.layer == "project")
        .expect("the absent project layer is recorded");
    assert!(!absent.present);
    assert_eq!(absent.item_id, "claude-code.source-project-settings");
    assert_eq!((absent.size, absent.modified.clone()), (None, None));
}
