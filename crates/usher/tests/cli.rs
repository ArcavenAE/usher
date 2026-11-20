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

/// The verb lives in the CLI crate; the binary only hands the command over. An argument
/// the verb does not take is refused by the verb, before anything is read.
#[test]
fn inventory_with_an_argument_is_refused_by_the_verb_not_as_an_unknown_mode() {
    let out = usher(&["inventory", "extra"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("usher inventory takes no arguments"),
        "{stderr}"
    );
    assert!(stderr.contains("next: run `usher inventory`"), "{stderr}");
    assert!(!stderr.contains("unknown mode"), "{stderr}");
}
