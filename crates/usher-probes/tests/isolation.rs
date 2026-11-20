//! The probe crate stays out of every shipping build: no other member depends
//! on it, and the signing-crate guard passes with it in the workspace.

use std::{fs, path::PathBuf, process::Command};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

#[test]
fn the_signing_crate_guard_passes_and_no_other_member_depends_on_the_probes() {
    let root = workspace_root();

    for entry in fs::read_dir(root.join("crates")).expect("crates dir") {
        let dir = entry.expect("entry").path();
        if dir.file_name().is_some_and(|n| n == "usher-probes") {
            continue;
        }
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).expect("a member manifest");
        assert!(
            !manifest.contains("usher-probes"),
            "{} depends on usher-probes",
            dir.display()
        );
    }

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new("scripts/guard-signer-crates.sh")
        .arg(".")
        .current_dir(&root)
        .env("GUARD_CARGO", cargo)
        .output()
        .expect("the guard script runs");
    assert!(
        output.status.success(),
        "guard failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
