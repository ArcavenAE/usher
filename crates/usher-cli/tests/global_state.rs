//! Story 1.5, PR D: `usher inventory` places the global state file by CLAUDE_CONFIG_DIR,
//! and prints each layer under its own label. Scratch home, scratch system root.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use usher_cli::{InventoryEnv, LaunchEnvironment, inventory};
use usher_core::SystemRoot;
use usher_knowledge::Platform;
use usher_store::Store;
use usher_testkit::ScratchHome;

const ACCOUNT: &str = r#"{"oauthAccount":{"emailAddress":"alex@example.test"}}"#;

fn account_line(path: &Path) -> String {
    format!(
        "auth-route  {}  oauthAccount present (knowledge item claude-code.seat-auth-account)",
        path.display()
    )
}

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
        let target = scratch.home().join(".local/share/claude/versions/2.1.317");
        fs::create_dir_all(target.parent().expect("a parent")).expect("dirs");
        fs::write(&target, "binary").expect("binary");
        fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).expect("mode");
        let launcher = scratch.home().join(".local/bin/claude");
        fs::create_dir_all(launcher.parent().expect("a parent")).expect("dirs");
        symlink(&target, launcher).expect("symlink");
        Rig {
            scratch,
            system,
            repo,
        }
    }

    fn env(&self, config_dir: Option<&str>) -> InventoryEnv {
        let pairs: Vec<(&str, &str)> = config_dir
            .map(|value| ("CLAUDE_CONFIG_DIR", value))
            .into_iter()
            .collect();
        InventoryEnv {
            host: "host-1".to_string(),
            user: "alex".to_string(),
            home: self.scratch.home().to_path_buf(),
            launch_dir: self.repo.clone(),
            launch: LaunchEnvironment::from_pairs(&pairs),
            path: Some(self.scratch.home().join(".local/bin").into_os_string()),
            state_dir: self.scratch.home().join(".local/state/usher"),
            now: "2026-11-20T12:00:00Z".to_string(),
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

    fn global_in_home(&self) -> PathBuf {
        self.write(&self.scratch.home().join(".claude.json"), ACCOUNT)
    }
}

fn stored_paths(rig: &Rig) -> Vec<String> {
    let store = Store::open(&rig.env(None).state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    let run = runs.last().expect("a run");
    store
        .sources_of(run.id)
        .expect("sources")
        .into_iter()
        .map(|row| row.path)
        .collect()
}

#[test]
fn an_unset_variable_reads_the_global_state_file_in_the_home_directory() {
    let rig = Rig::new();
    let global = rig.global_in_home();
    let out = rig.run(&rig.env(None));
    assert!(out.contains(&global.display().to_string()), "{out}");
    assert!(stored_paths(&rig).contains(&global.display().to_string()));
    assert!(out.lines().any(|l| l == account_line(&global)), "{out}");
}

#[test]
fn an_absolute_variable_moves_the_global_state_file_and_the_home_one_is_not_read() {
    let rig = Rig::new();
    let at_home = rig.global_in_home();
    let dir = rig.scratch.root().join("state");
    let moved = rig.write(&dir.join(".claude.json"), ACCOUNT);
    let out = rig.run(&rig.env(Some(&dir.display().to_string())));
    assert!(out.contains(&moved.display().to_string()), "{out}");
    assert!(!out.contains(&at_home.display().to_string()), "{out}");
    assert!(out.lines().any(|l| l == account_line(&moved)), "{out}");
    let stored = stored_paths(&rig);
    assert!(stored.contains(&moved.display().to_string()), "{stored:?}");
    assert!(
        !stored.contains(&at_home.display().to_string()),
        "{stored:?}"
    );
}

#[test]
fn an_empty_variable_reads_the_global_state_file_in_the_home_directory() {
    let rig = Rig::new();
    let global = rig.global_in_home();
    let out = rig.run(&rig.env(Some("")));
    assert!(out.contains(&global.display().to_string()), "{out}");
    assert!(out.lines().any(|l| l == account_line(&global)), "{out}");
}

#[test]
fn a_relative_variable_leaves_the_global_state_not_checked_under_its_own_label() {
    let rig = Rig::new();
    rig.global_in_home();
    let out = rig.run(&rig.env(Some("relative/dir")));
    assert!(
        out.lines()
            .any(|line| line == "not checked: global state (CLAUDE_CONFIG_DIR is not absolute)"),
        "{out}"
    );
    assert!(!out.contains("global state settings"), "{out}");
    assert!(!out.contains("global-state settings"), "{out}");
    assert!(
        out.contains("not checked: user settings (CLAUDE_CONFIG_DIR is not absolute)"),
        "the user line keeps its label: {out}"
    );
}

#[test]
fn the_other_layers_print_under_their_own_labels() {
    let rig = Rig::new();
    let out = rig.run(&rig.env(None));
    for label in ["user settings", "project settings", "local settings"] {
        assert!(
            out.lines()
                .any(|line| line.starts_with(&format!("no file: {label} ("))),
            "{label}: {out}"
        );
    }
}

const REASON: &str = "CLAUDE_CONFIG_DIR is not absolute";
const ACCOUNT_ITEM: &str = "claude-code.seat-auth-account";

#[test]
fn a_relative_variable_prints_the_account_item_not_checked_beside_the_source_line() {
    let rig = Rig::new();
    rig.global_in_home();
    let out = rig.run(&rig.env(Some("relative/dir")));
    assert!(
        out.lines()
            .any(|l| l == format!("not checked: global state ({REASON})")),
        "{out}"
    );
    assert!(
        out.lines()
            .any(|l| l
                == format!("auth-route  not checked ({REASON}) (knowledge item {ACCOUNT_ITEM})")),
        "the seat item says so for itself: {out}"
    );
}

#[test]
fn a_relative_variable_stores_the_unplaceable_item_with_no_flag_and_its_reason() {
    let rig = Rig::new();
    rig.global_in_home();
    rig.run(&rig.env(Some("relative/dir")));
    let store = Store::open(&rig.env(None).state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    let rows = store.seat_of(runs.last().expect("a run").id).expect("rows");
    let row = rows
        .iter()
        .find(|row| row.item_id == ACCOUNT_ITEM)
        .unwrap_or_else(|| panic!("an unplaceable item is never dropped: {rows:?}"));
    assert_eq!(row.present, None);
    assert!(row.names.is_empty() && row.members.is_empty());
    assert_eq!(row.note.as_deref(), Some(REASON));
}

#[test]
fn the_not_checked_lines_name_the_variable_and_never_its_value() {
    let rig = Rig::new();
    rig.global_in_home();
    let out = rig.run(&rig.env(Some("relative/dir")));
    let seat = out
        .lines()
        .find(|l| l.ends_with(&format!("(knowledge item {ACCOUNT_ITEM})")))
        .unwrap_or_else(|| panic!("no seat line for the account item in:\n{out}"));
    let source = out
        .lines()
        .find(|l| l.starts_with("not checked: global state"))
        .unwrap_or_else(|| panic!("no not-checked source line in:\n{out}"));
    for line in [seat, source] {
        assert!(line.contains("CLAUDE_CONFIG_DIR"), "{line}");
        assert!(!line.contains("relative/dir"), "{line}");
    }
}

#[test]
fn no_seat_column_holds_the_relative_value() {
    let rig = Rig::new();
    rig.global_in_home();
    rig.run(&rig.env(Some("relative/dir")));
    let store = Store::open(&rig.env(None).state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    let rows = store.seat_of(runs.last().expect("a run").id).expect("rows");
    assert!(
        rows.iter().any(|row| row.item_id == ACCOUNT_ITEM),
        "the unplaceable item has a row to inspect: {rows:?}"
    );
    assert!(!format!("{rows:?}").contains("relative/dir"), "{rows:?}");
}

/// The control that pins the narrowing both ways: the environment key is identity, so it
/// keeps the value of a set relocation variable, and an unset variable is another key.
#[test]
fn the_environment_key_still_holds_the_relocation_variable_and_its_value() {
    let rig = Rig::new();
    rig.run(&rig.env(Some("relative/dir")));
    rig.run(&rig.env(None));
    let store = Store::open(&rig.env(None).state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 2);
    assert!(
        runs[0]
            .environment_key
            .ends_with("?CLAUDE_CONFIG_DIR=relative/dir"),
        "{}",
        runs[0].environment_key
    );
    assert!(!runs[1].environment_key.contains("CLAUDE_CONFIG_DIR"));
    assert_ne!(runs[0].environment_key, runs[1].environment_key);
}
