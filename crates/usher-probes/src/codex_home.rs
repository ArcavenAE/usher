//! A scratch copy of a codex home: config files only, never credentials,
//! removed when dropped.

use std::{
    io,
    path::{Path, PathBuf},
};

use usher_testkit::ScratchHome;

/// The copy. Dropping it deletes the directory.
#[derive(Debug)]
pub struct CodexCopy {
    path: PathBuf,
}

impl CodexCopy {
    /// Copies the top-level `*.toml` files of `source` into a new directory
    /// under `scratch`. `auth.json` and every other file stay behind.
    pub fn create(_source: &Path, _scratch: &ScratchHome) -> io::Result<CodexCopy> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    /// The copy's directory, for `CODEX_HOME`.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// A fresh codex home for a probe: a new directory under the scratch home that
/// holds exactly the files the probe writes, as `(name, text)` pairs. A name
/// with a path separator, or `auth.json`, is refused, and nothing already in
/// the scratch codex home is carried over.
pub fn probe_home(_scratch: &ScratchHome, _files: &[(&str, &str)]) -> io::Result<PathBuf> {
    Err(io::Error::from(io::ErrorKind::Unsupported))
}
