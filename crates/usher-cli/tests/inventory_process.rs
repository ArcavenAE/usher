//! An override variable in the real process environment changes nothing `usher inventory`
//! prints or stores: the verb treats its name as set, never reads its value, and the value
//! appears nowhere. A child process runs the verb with and without the variable over one
//! scratch home, and the parent compares what each printed and stored.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;

use usher_cli::{InventoryEnv, inventory};
use usher_core::SystemRoot;
use usher_store::Store;

const OVERRIDE_VALUE: &str = "canary-inventory-override-value";

/// Run by the test below as a child process, with or without the variable.
#[test]
#[ignore = "run only by the_override_variable_changes_nothing_the_inventory_prints_or_stores, as a child process"]
fn child_runs_the_inventory_over_the_process_environment() {
    let Some(root) = std::env::var_os("USHER_INV_CHILD_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let mut env = InventoryEnv::from_process().expect("this process's environment");
    env.home = root.join("home");
    env.state_dir = root.join("home/.local/state/usher");
    env.launch_dir = root.join("home/repo");
    env.path = Some(root.join("home/.local/bin").into_os_string());
    env.now = "2026-11-20T12:00:00Z".to_string();
    env.host = "host-1".to_string();
    env.user = "alex".to_string();
    assert!(
        !format!("{env:?}").contains(OVERRIDE_VALUE),
        "the value is held in the environment record"
    );
    let out = inventory(&env, SystemRoot::at(root.join("system"))).expect("an inventory");
    let store = Store::open(&env.state_dir).expect("the store opens");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 1, "{runs:?}");
    let sources: Vec<_> = store
        .sources_of(runs[0].id)
        .expect("sources")
        .into_iter()
        .map(|mut row| {
            row.run_id = 0;
            row
        })
        .collect();
    println!("OUT_BEGIN\n{out}\nOUT_END");
    println!("KEY={}", runs[0].environment_key);
    println!("SOURCES={sources:?}");
    println!("SEAT={:?}", store.seat_of(runs[0].id).expect("seat rows"));
}

fn run_child(root: &Path, with_variable: bool) -> String {
    let mut command = Command::new(std::env::current_exe().expect("this test binary"));
    command
        .args([
            "--ignored",
            "--exact",
            "child_runs_the_inventory_over_the_process_environment",
            "--nocapture",
        ])
        .env_remove("CLAUDE_CONFIG_DIR")
        .env_remove("ANTHROPIC_MODEL")
        .env("USHER_INV_CHILD_ROOT", root);
    if with_variable {
        command.env("ANTHROPIC_MODEL", OVERRIDE_VALUE);
    }
    let out = command.output().expect("the child runs");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "the child failed:\n{stdout}\n{stderr}"
    );
    assert!(!stdout.contains(OVERRIDE_VALUE), "the value is on stdout");
    assert!(!stderr.contains(OVERRIDE_VALUE), "the value is on stderr");
    assert!(stdout.contains("OUT_BEGIN"), "{stdout}");
    // The runner's own lines (test names, timings) are not part of the comparison.
    stdout
        .lines()
        .skip_while(|l| !l.contains("OUT_BEGIN"))
        .take_while(|l| !l.starts_with("test result"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn stored_files(dir: &Path) -> Vec<Vec<u8>> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in fs::read_dir(&next).expect("a directory") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                found.push(fs::read(&path).expect("a file"));
            }
        }
    }
    found
}

#[test]
fn the_override_variable_changes_nothing_the_inventory_prints_or_stores() {
    let scratch = usher_testkit::ScratchHome::new().expect("scratch");
    let root = scratch.root().to_path_buf();
    fs::create_dir_all(root.join("home/repo/.git")).expect("repo");
    fs::create_dir_all(root.join("system")).expect("system root");
    fs::create_dir_all(root.join("home/.claude")).expect("config dir");
    fs::write(
        root.join("home/.claude/settings.json"),
        r#"{"model": "file-model"}"#,
    )
    .expect("settings");
    let target = root.join("home/.local/share/claude/versions/2.1.317");
    fs::create_dir_all(target.parent().expect("a parent")).expect("dirs");
    fs::write(&target, "binary").expect("binary");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).expect("mode");
    let launcher = root.join("home/.local/bin/claude");
    fs::create_dir_all(launcher.parent().expect("a parent")).expect("dirs");
    symlink(&target, launcher).expect("symlink");

    let without = run_child(&root, false);
    let state = root.join("home/.local/state/usher");
    for bytes in stored_files(&state) {
        assert!(
            !bytes
                .windows(OVERRIDE_VALUE.len())
                .any(|w| w == OVERRIDE_VALUE.as_bytes())
        );
    }
    fs::remove_dir_all(&state).expect("a fresh store for the second run");
    let with = run_child(&root, true);
    for bytes in stored_files(&state) {
        assert!(
            !bytes
                .windows(OVERRIDE_VALUE.len())
                .any(|w| w == OVERRIDE_VALUE.as_bytes()),
            "the value is stored"
        );
    }
    assert!(without.contains("KEY="), "{without}");
    assert_eq!(
        without, with,
        "an override variable changed what the inventory prints or stores"
    );
}
