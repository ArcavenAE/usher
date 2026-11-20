//! A scratch copy of a codex home: config files only, never credentials,
//! removed when dropped; and a fresh home for a probe.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use usher_testkit::ScratchHome;

static NEXT: AtomicU64 = AtomicU64::new(0);

/// The copy. Dropping it deletes the directory.
#[derive(Debug)]
pub struct CodexCopy {
    path: PathBuf,
}

/// A new, empty directory under the scratch root, named by `prefix`.
fn fresh_dir(scratch: &ScratchHome, prefix: &str) -> io::Result<PathBuf> {
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = scratch.root().join(format!("{prefix}-{n}"));
    fs::create_dir(&path)?;
    Ok(path)
}

impl CodexCopy {
    /// Copies the top-level `*.toml` files of `source` into a new directory
    /// under `scratch`. `auth.json` and every other file stay behind. A
    /// symlink is not followed and a directory is not entered.
    pub fn create(source: &Path, scratch: &ScratchHome) -> io::Result<CodexCopy> {
        // Read the source first, so a missing source leaves no directory.
        let entries = fs::read_dir(source)?;
        let copy = CodexCopy {
            path: fresh_dir(scratch, "codex-copy")?,
        };
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let is_toml = name.to_str().is_some_and(|n| n.ends_with(".toml"));
            // `DirEntry::file_type` does not follow a symlink.
            if is_toml && entry.file_type()?.is_file() {
                fs::copy(entry.path(), copy.path.join(&name))?;
            }
        }
        Ok(copy)
    }

    /// The copy's directory, for `CODEX_HOME`.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for CodexCopy {
    fn drop(&mut self) {
        // Best effort: a failed removal must not turn a passing test into a panic.
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn plain_file_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name != "auth.json"
        && !name.contains(['/', '\\'])
}

/// The codex home for a probe: the scratch home's own codex home, which must be
/// empty, written with exactly the files the probe gives, as `(name, text)`
/// pairs. A name with a path separator, or `auth.json`, is refused, a codex
/// home that already holds anything is refused, and a refused call writes
/// nothing. Returns the directory, for `CODEX_HOME`.
pub fn probe_home(scratch: &ScratchHome, files: &[(&str, &str)]) -> io::Result<PathBuf> {
    if let Some((name, _)) = files.iter().find(|(name, _)| !plain_file_name(name)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("`{name}` is not a file name a probe home may hold"),
        ));
    }
    let home = scratch.codex_home();
    if fs::read_dir(home)?.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "the scratch codex home already holds files, so it is not a fresh probe home",
        ));
    }
    for (name, text) in files {
        fs::write(home.join(name), text)?;
    }
    Ok(home.to_path_buf())
}
