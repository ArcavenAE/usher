//! Story 1.5, PR C: what `usher inventory` prints and records. Runs against a scratch
//! home and a scratch system root; the real harness homes are guarded.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use usher_cli::{InventoryEnv, inventory, inventory_system_root, run};
use usher_core::SystemRoot;
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
            claude_config_dir: None,
            path: Some(self.scratch.home().join(".local/bin").into_os_string()),
            state_dir: self.scratch.home().join(".local/state/usher"),
            now: NOW.to_string(),
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
    assert!(first.starts_with("Claude Code 2.1.317"), "{first}");
    assert!(first.contains("knowledge v"), "{first}");
}

#[test]
fn an_unknown_version_is_stamped_unknown_with_its_reason_and_statuses_read_unknown() {
    let rig = Rig::new();
    let user = rig.user(r#"{"model":"m"}"#);
    let out = rig.run(&rig.env());
    let first = out.lines().next().expect("a first line");
    assert!(
        first.starts_with("Claude Code version unknown ("),
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
    assert!(keys.contains("not classified"), "{keys}");
    assert!(!out.contains("super-secret-model"), "no value is printed");
}

#[test]
fn a_shadowed_managed_source_names_the_winner() {
    let rig = Rig::new();
    rig.install("2.1.317");
    let winner = rig.write(
        &rig.system
            .join("Library/Application Support/ClaudeCode/managed-settings.json"),
        r#"{"permissions":{}}"#,
    );
    let loser = rig.write(
        &rig.system.join("etc/claude-code/managed-settings.json"),
        r#"{"model":"m"}"#,
    );
    let out = rig.run(&rig.env());
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
    env.claude_config_dir = Some("relative/claude".into());
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
    env.claude_config_dir = Some(rig.scratch.root().join("cfg").into_os_string());
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
    env.claude_config_dir = Some("".into());
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
    rig.run(&env);
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 2, "a second run inserts, never rewrites");
    assert_eq!(runs[0].recorded_at, NOW);
    assert_eq!(runs[0].harness, "claude-code");
    assert_eq!(runs[0].harness_version.as_deref(), Some("2.1.317"));
    assert!(
        runs[0]
            .environment_key
            .starts_with("host-1/alex/claude-code/"),
        "{}",
        runs[0].environment_key
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
    let run = &store.runs().expect("runs")[0];
    let sources = store.sources_of(run.id).expect("sources");
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
        &rig.system.join("etc/claude-code/managed-settings.json"),
        r#"{"permissions":{}}"#,
    );
    let watched = RealHomes::from_homes(vec![
        NamedHome {
            name: "claude-code",
            path: rig.scratch.home().join(".claude"),
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
fn the_verb_builds_its_adapter_with_the_host_root() {
    assert_eq!(inventory_system_root(), SystemRoot::host());
}

#[test]
fn run_hands_inventory_to_its_verb_and_refuses_any_other_command_or_argument() {
    let rig = Rig::new();
    let env = rig.env();
    let err = run(&["inventory".to_string(), "extra".to_string()], &env)
        .expect_err("inventory takes no argument");
    assert_eq!(err.step(), "run inventory");
    assert!(err.status().contains("extra"), "{}", err.status());
    let err = run(&["whatever".to_string()], &env).expect_err("an unknown command");
    assert_eq!(err.step(), "select command");
    assert!(!err.next_action().is_empty());
}
