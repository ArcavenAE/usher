//! Story 1.6, PR D: an override variable reaches the `map` verb from the real process
//! environment as present, and its value appears nowhere: not in the output, not in the
//! store, and not in the environment record. Only its name is printed. The child runs the
//! verb's own entry points over a scratch home, so the real home is never read or written.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;

use usher_cli::{InventoryEnv, map};
use usher_core::SystemRoot;

const OVERRIDE_VALUE: &str = "canary-override-value-in-the-process";
const FILE_VALUE: &str = "canary-model-in-the-file";

/// Run by the test below as a child process with the variable in its environment.
///
/// Provisional: the row text rests on the row layout, which is not final.
#[test]
#[ignore = "run only by an_override_variable_is_present_and_its_value_is_held_nowhere, as a child process"]
fn child_runs_the_map_over_the_process_environment() {
    let Some(root) = std::env::var_os("USHER_MAP_CHILD_ROOT") else {
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
        !format!("{env:?}").contains(OVERRIDE_VALUE),
        "the value is held in the environment record"
    );
    let launch_dir = env.launch_dir.clone();
    let out = map(&env, &launch_dir, SystemRoot::at(root.join("system"))).expect("a map");
    let line = out
        .lines()
        .find(|l| l.contains("overridden by environment"))
        .unwrap_or_else(|| panic!("no overridden line in:\n{out}"));
    let status = line
        .find("overridden by environment")
        .expect("the status words");
    let layer = line.find("(user: ").expect("the layer and file");
    let note = line
        .find("; ANTHROPIC_MODEL present")
        .expect("the note names the variable");
    assert!(
        status < layer && layer < note,
        "fields are out of order: {line}"
    );
    assert!(
        line.trim_start()
            .starts_with("model  overridden by environment  (user: "),
        "the row has no value field: {line}"
    );
    assert!(!line.contains("value not read"), "{line}");
    assert!(!out.contains(OVERRIDE_VALUE), "the value is printed");
    assert!(!out.contains(FILE_VALUE), "the file value is printed");
    let mut pending = vec![env.state_dir.clone()];
    let mut files = 0;
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("a state directory") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            files += 1;
            let bytes = fs::read(&path).expect("a file");
            for canary in [OVERRIDE_VALUE, FILE_VALUE] {
                assert!(
                    !bytes.windows(canary.len()).any(|w| w == canary.as_bytes()),
                    "`{canary}` is stored in {path:?}"
                );
            }
        }
    }
    assert!(files > 0, "the walk found no stored file");
    println!("CHILD_OK");
}

#[test]
fn an_override_variable_is_present_and_its_value_is_held_nowhere() {
    let scratch = usher_testkit::ScratchHome::new().expect("scratch");
    let root = scratch.root().to_path_buf();
    fs::create_dir_all(root.join("home/repo/.git")).expect("repo");
    fs::create_dir_all(root.join("system")).expect("system root");
    fs::create_dir_all(root.join("home/.claude")).expect("config dir");
    fs::write(
        root.join("home/.claude/settings.json"),
        format!(r#"{{"model": "{FILE_VALUE}"}}"#),
    )
    .expect("settings");
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
            "child_runs_the_map_over_the_process_environment",
            "--nocapture",
        ])
        .env_remove("CLAUDE_CONFIG_DIR")
        .env("USHER_MAP_CHILD_ROOT", &root)
        .env("ANTHROPIC_MODEL", OVERRIDE_VALUE)
        .output()
        .expect("the child runs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success() && stdout.contains("CHILD_OK"),
        "the child failed:\n{stdout}\n{stderr}"
    );
    for canary in [OVERRIDE_VALUE, FILE_VALUE] {
        assert!(
            !stdout.contains(canary),
            "`{canary}` is on the child's stdout"
        );
        assert!(
            !stderr.contains(canary),
            "`{canary}` is on the child's stderr"
        );
    }
}
