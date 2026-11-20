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
                || (line.contains("std::process") && line.contains("Command"));
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
