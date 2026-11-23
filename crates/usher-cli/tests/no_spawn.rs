//! The CLI and the adapter name no process-spawn API: neither crate starts another program,
//! so nothing on a user's `PATH` can answer for what usher reads. A structural check over
//! the two crates' sources, with no exemption.

use std::fs;
use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("read {dir:?}: {e}")) {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn the_cli_and_the_adapter_name_no_process_spawn_api() {
    let cli = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let adapter = Path::new(env!("CARGO_MANIFEST_DIR")).join("../usher-adapter-claude-code/src");
    let mut files = Vec::new();
    rust_files(&cli, &mut files);
    rust_files(&adapter, &mut files);
    assert!(
        files.len() >= 4,
        "the scan found too few files to mean anything: {files:?}"
    );
    let mut hits = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).expect("a source file");
        for (number, line) in text.lines().enumerate() {
            let spawns = line.contains("process::Command")
                || line.contains("Command::new")
                || (line.contains("std::process") && line.contains("Command"))
                || ["libc::", "posix_spawn", "fork(", "execv"]
                    .iter()
                    .any(|token| line.contains(token));
            if spawns {
                hits.push(format!(
                    "{}:{}: {}",
                    file.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "a process-spawn API is named:\n{}",
        hits.join("\n")
    );
}

/// The one way a spawn could arrive without a token in the source is a Cargo change that
/// turns on a process feature of `nix`, so the feature list is pinned: exactly `user`,
/// `hostname` and `fs`, set once in the workspace and inherited by the CLI without an
/// override. `fs` holds the open flags and no process API.
#[test]
fn the_nix_features_are_exactly_user_hostname_and_fs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let workspace = fs::read_to_string(root.join("Cargo.toml")).expect("the workspace manifest");
    let line = workspace
        .lines()
        .find(|l| l.trim_start().starts_with("nix = "))
        .expect("a workspace nix line");
    let start = line.find("features = [").expect("a feature list") + "features = [".len();
    let end = start + line[start..].find(']').expect("the list ends");
    let mut features: Vec<&str> = line[start..end]
        .split(',')
        .map(|f| f.trim().trim_matches('"'))
        .filter(|f| !f.is_empty())
        .collect();
    features.sort_unstable();
    assert_eq!(features, ["fs", "hostname", "user"], "{line}");
    let cli = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("the CLI manifest");
    let nix: Vec<&str> = cli
        .lines()
        .filter(|l| l.trim_start().starts_with("nix"))
        .collect();
    assert_eq!(
        nix,
        ["nix.workspace = true"],
        "the CLI overrides the nix line"
    );
}
