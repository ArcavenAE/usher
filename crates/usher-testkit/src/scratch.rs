//! Scratch homes: a throwaway home directory a test runs a harness in.
//!
//! The environment is applied to spawned [`Command`]s only. Setting variables
//! in the test process itself needs `std::env::set_var`, which is `unsafe`, and
//! every usher crate forbids unsafe code. Library code under test therefore
//! takes its launch context explicitly (architecture decision AD-17) instead of
//! reading `HOME` in process.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, io, process};

/// Where scratch roots are made. A short fixed base keeps socket paths under
/// the 104-byte limit that `TMPDIR` values under the harness can pass.
const SCRATCH_BASE: &str = "/tmp";

/// How many names to try before giving up on a collision.
const NAME_ATTEMPTS: u32 = 64;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A fresh temporary directory laid out like a person's home.
///
/// `HOME`, `TMPDIR` and each harness home variable point inside it. The
/// directory is removed when the value is dropped.
#[derive(Debug)]
pub struct ScratchHome {
    root: PathBuf,
    home: PathBuf,
    tmp: PathBuf,
    claude_config_dir: PathBuf,
    codex_home: PathBuf,
    goose_path_root: PathBuf,
}

impl ScratchHome {
    /// Makes a scratch home under `/tmp`.
    pub fn new() -> io::Result<Self> {
        Self::new_in(Path::new(SCRATCH_BASE))
    }

    /// Makes a scratch home under `base`, which must already exist.
    pub fn new_in(base: &Path) -> io::Result<Self> {
        let mut last = None;
        for _ in 0..NAME_ATTEMPTS {
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = base.join(format!("us-{}-{n}", process::id()));
            match fs::create_dir(&root) {
                Ok(()) => return Self::lay_out(root),
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => last = Some(err),
                Err(err) => return Err(err),
            }
        }
        Err(last.unwrap_or_else(|| io::Error::other("no scratch name was free")))
    }

    fn lay_out(root: PathBuf) -> io::Result<Self> {
        let home = root.join("home");
        let scratch = Self {
            tmp: root.join("tmp"),
            claude_config_dir: home.join(".claude"),
            codex_home: home.join(".codex"),
            goose_path_root: home.join(".config").join("goose"),
            home,
            root,
        };
        // On a failure here `scratch` drops and removes the root.
        for dir in [
            &scratch.tmp,
            &scratch.claude_config_dir,
            &scratch.codex_home,
            &scratch.goose_path_root,
        ] {
            fs::create_dir_all(dir)?;
        }
        Ok(scratch)
    }

    /// The scratch root that holds everything and is removed on drop.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The value of `HOME` for processes run in this scratch home.
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// The value of `TMPDIR`.
    pub fn tmp(&self) -> &Path {
        &self.tmp
    }

    /// The value of `CLAUDE_CONFIG_DIR`.
    pub fn claude_config_dir(&self) -> &Path {
        &self.claude_config_dir
    }

    /// The value of `CODEX_HOME`.
    pub fn codex_home(&self) -> &Path {
        &self.codex_home
    }

    /// The value of `GOOSE_PATH_ROOT`. goose reads it only when it is absolute.
    pub fn goose_path_root(&self) -> &Path {
        &self.goose_path_root
    }

    /// The five variables a process needs to stay inside this scratch home.
    pub fn env(&self) -> [(&'static str, &Path); 5] {
        [
            ("HOME", self.home()),
            ("TMPDIR", self.tmp()),
            ("CLAUDE_CONFIG_DIR", self.claude_config_dir()),
            ("CODEX_HOME", self.codex_home()),
            ("GOOSE_PATH_ROOT", self.goose_path_root()),
        ]
    }

    /// A command that runs with the scratch variables set and the rest of the
    /// environment inherited (so `PATH` still resolves the program).
    pub fn command(&self, program: impl AsRef<OsStr>) -> Command {
        let mut command = Command::new(program);
        for (key, value) in self.env() {
            command.env(key, value);
        }
        command
    }
}

impl Drop for ScratchHome {
    fn drop(&mut self) {
        // Best effort: a failed removal must not turn a passing test into a panic.
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variable_points_inside_one_short_fresh_root() {
        let scratch = ScratchHome::new().expect("a scratch home is made");
        assert!(scratch.root().to_string_lossy().len() < 60);
        assert!(scratch.root().starts_with(SCRATCH_BASE));
        let names: Vec<_> = scratch.env().iter().map(|(name, _)| *name).collect();
        assert_eq!(
            names,
            [
                "HOME",
                "TMPDIR",
                "CLAUDE_CONFIG_DIR",
                "CODEX_HOME",
                "GOOSE_PATH_ROOT"
            ]
        );
        for (name, value) in scratch.env() {
            assert!(value.starts_with(scratch.root()), "{name} escapes the root");
            assert!(value.is_dir(), "{name} directory was not made");
            assert!(value.is_absolute(), "{name} is not absolute");
        }
    }

    #[test]
    fn two_scratch_homes_do_not_share_a_root() {
        let a = ScratchHome::new().expect("first scratch home");
        let b = ScratchHome::new().expect("second scratch home");
        assert_ne!(a.root(), b.root());
    }

    #[test]
    fn the_root_is_removed_on_drop() {
        let scratch = ScratchHome::new().expect("a scratch home is made");
        let root = scratch.root().to_path_buf();
        fs::write(scratch.home().join("file"), "x").expect("a file is written");
        assert!(root.is_dir());
        drop(scratch);
        assert!(!root.exists());
    }

    #[test]
    fn a_missing_base_is_an_error_not_a_panic() {
        let err = ScratchHome::new_in(Path::new("/nonexistent-usher-scratch-base"))
            .expect_err("the base does not exist");
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn a_spawned_process_sees_the_scratch_variables() {
        let scratch = ScratchHome::new().expect("a scratch home is made");
        let out = scratch
            .command("sh")
            .args([
                "-c",
                "printf '%s|%s|%s|%s|%s' \"$HOME\" \"$TMPDIR\" \"$CLAUDE_CONFIG_DIR\" \"$CODEX_HOME\" \"$GOOSE_PATH_ROOT\"",
            ])
            .output()
            .expect("sh runs");
        let seen = String::from_utf8_lossy(&out.stdout).into_owned();
        let wanted: Vec<String> = scratch
            .env()
            .iter()
            .map(|(_, value)| value.to_string_lossy().into_owned())
            .collect();
        assert_eq!(seen, wanted.join("|"));
    }
}
