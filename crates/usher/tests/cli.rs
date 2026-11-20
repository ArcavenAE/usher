//! Runs the built `usher` binary, as the I/O matrix of Story 1.1 describes.

use std::process::{Command, Output};

fn usher(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_usher"))
        .args(args)
        .output()
        .expect("the usher binary runs")
}

#[test]
fn version_prints_the_version_and_exits_zero() {
    let out = usher(&["--version"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.trim(),
        format!("usher {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn unknown_mode_exits_nonzero_and_names_a_next_action() {
    let out = usher(&["frobnicate"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    for field in ["step:", "check:", "status:", "next:"] {
        assert!(stderr.contains(field), "stderr lacks `{field}`: {stderr}");
    }
    assert!(stderr.contains("frobnicate"));
}

#[test]
fn no_mode_exits_nonzero_with_the_same_shape() {
    let out = usher(&[]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("next:"),
        "stderr lacks a next action: {stderr}"
    );
}

mod inventory {
    use std::fs;
    use std::path::Path;
    use std::process::{Command, Output};

    use usher_testkit::{RealHomes, ScratchHome, observe};

    /// A fixture value that is plainly not a credential.
    const FAKE_VALUE: &str = "FIXTURE-VALUE-NOT-A-SECRET";

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        fs::write(path, text).expect("write");
    }

    /// Runs `usher inventory` with nothing inherited but PATH-free scratch variables.
    fn run(scratch: &ScratchHome, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_usher"))
            .env_clear()
            .env("HOME", scratch.home())
            .env("TMPDIR", scratch.tmp())
            .env("CLAUDE_CONFIG_DIR", scratch.claude_config_dir())
            .env("USHER_SYSTEM_ROOT", scratch.root().join("system"))
            .arg("inventory")
            .args(args)
            .output()
            .expect("the usher binary runs")
    }

    #[test]
    fn lists_every_source_with_location_layer_and_status_and_changes_nothing() {
        let homes = RealHomes::ambient();
        observe(&homes, |scratch| {
            let config = scratch.claude_config_dir();
            let project = scratch.root().join("project");
            write(
                &config.join("settings.json"),
                r#"{"model":"m","brandNewKey":1}"#,
            );
            write(&config.join("remote-settings.json"), "{}");
            write(
                &config.join(".claude.json"),
                &format!(r#"{{"oauthAccount":"{FAKE_VALUE}"}}"#),
            );
            write(&project.join(".claude/settings.json"), "{}");
            write(&project.join(".claude/settings.local.json"), "{}");
            let out = run(scratch, &["--root", &project.to_string_lossy()]);
            let stdout = String::from_utf8_lossy(&out.stdout);
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            for needle in [
                "layer: user",
                "layer: project",
                "layer: local",
                "layer: managed",
                "layer: global-config",
                "harness: claude-code",
                "status: unknown (no environment is mapped)",
                "unrecognized keys: brandNewKey",
                "the list of recognized keys is partial",
            ] {
                assert!(stdout.contains(needle), "missing `{needle}`:\n{stdout}");
            }
            assert!(
                !stdout.contains(FAKE_VALUE),
                "a value reached the output:\n{stdout}"
            );
        });
    }

    #[test]
    fn without_a_root_it_says_project_files_were_not_looked_for() {
        let scratch = ScratchHome::new().expect("scratch");
        let out = run(&scratch, &[]);
        assert!(out.status.success());
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("no --root was given"), "{stdout}");
        assert!(stdout.contains("No settings source was found."), "{stdout}");
        assert!(stdout.contains("Looked for, not present:"), "{stdout}");
    }

    #[test]
    fn a_bad_argument_exits_nonzero_with_the_four_fields() {
        let scratch = ScratchHome::new().expect("scratch");
        let out = run(&scratch, &["--root"]);
        assert!(!out.status.success());
        let stderr = String::from_utf8_lossy(&out.stderr);
        for field in ["step:", "check:", "status:", "next:"] {
            assert!(stderr.contains(field), "stderr lacks `{field}`: {stderr}");
        }
    }
}
