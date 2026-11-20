//! Story 1.5, PR D: a variable only the seat reads reaches the verb from the real process
//! environment as "set", and its value appears nowhere: not in the output, not in the
//! store, and not in the environment key. The child runs `usher inventory`'s own entry
//! points over a scratch home, so the real home is never read or written.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;

use usher_cli::{InventoryEnv, inventory};
use usher_core::SystemRoot;

const SECRET: &str = "sk-child-process-secret-value";

/// Run by the test below as a child process with the secret in its environment.
#[test]
#[ignore = "run only by a_seat_only_variable_is_present_and_its_value_is_held_nowhere, as a child process"]
fn child_runs_the_inventory_over_the_process_environment() {
    let Some(root) = std::env::var_os("USHER_SEAT_CHILD_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let mut env = InventoryEnv::from_process().expect("this process's environment");
    assert!(
        env.launch.get("CLAUDE_CONFIG_DIR").is_none(),
        "the child must not inherit a real config directory"
    );
    env.home = root.join("home");
    env.state_dir = root.join("home/.local/state/usher");
    env.launch_dir = root.join("home/repo");
    env.path = Some(root.join("home/.local/bin").into_os_string());
    assert!(
        !format!("{env:?}").contains(SECRET),
        "the value is held in the environment record"
    );
    let system = root.join("system");
    let out = inventory(&env, SystemRoot::at(system)).expect("an inventory");
    let line = out
        .lines()
        .find(|l| l.ends_with("(knowledge item claude-code.seat-auth-env)"))
        .unwrap_or_else(|| panic!("no auth env line in:\n{out}"));
    assert!(line.contains("ANTHROPIC_API_KEY present"), "{line}");
    assert!(!out.contains(SECRET), "the value is printed");
    for entry in fs::read_dir(&env.state_dir).expect("the state directory") {
        let bytes = fs::read(entry.expect("an entry").path()).expect("a file");
        assert!(
            !bytes.windows(SECRET.len()).any(|w| w == SECRET.as_bytes()),
            "the value is stored"
        );
    }
    println!("CHILD_OK");
}

#[test]
fn a_seat_only_variable_is_present_and_its_value_is_held_nowhere() {
    let scratch = usher_testkit::ScratchHome::new().expect("scratch");
    let root = scratch.root().to_path_buf();
    fs::create_dir_all(root.join("home/repo/.git")).expect("repo");
    fs::create_dir_all(root.join("system")).expect("system root");
    let target = root.join("home/.local/share/claude/versions/2.1.317");
    fs::create_dir_all(target.parent().expect("a parent")).expect("dirs");
    fs::write(&target, "binary").expect("binary");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).expect("mode");
    let launcher = root.join("home/.local/bin/claude");
    fs::create_dir_all(launcher.parent().expect("a parent")).expect("dirs");
    symlink(&target, launcher).expect("symlink");
    let out = std::process::Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            "--ignored",
            "--exact",
            "child_runs_the_inventory_over_the_process_environment",
            "--nocapture",
        ])
        .env_remove("CLAUDE_CONFIG_DIR")
        .env("USHER_SEAT_CHILD_ROOT", &root)
        .env("ANTHROPIC_API_KEY", SECRET)
        .output()
        .expect("the child runs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success() && stdout.contains("CHILD_OK"),
        "{stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!stdout.contains(SECRET), "the child printed the value");
}
