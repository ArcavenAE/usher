//! The observation invariant: running usher or a harness under a scratch home
//! must leave the real harness homes exactly as they were.
//!
//! The check hashes metadata, never file contents: name, file type, size,
//! mtime, ctime and inode of every entry, symlinks not followed. The digests
//! are compared within one process run and are never written to disk, so
//! `std`'s `DefaultHasher` is enough. If a digest is ever stored or compared
//! across runs or toolchains, replace it with a stable hash first.
//!
//! This module reads the real home for metadata only. It is Unix-only for now
//! because ctime and inode come from `MetadataExt` (macOS first, Windows open).

use std::collections::hash_map::DefaultHasher;
use std::ffi::OsString;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::{error::Error, fs, io};

use crate::scratch::ScratchHome;

/// A metadata digest of one directory tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeDigest {
    /// Nothing exists at the path.
    Absent,
    /// The tree exists; the value hashes every entry's metadata.
    Present(u64),
}

/// Digests the metadata of `path` and everything under it.
///
/// The root itself is followed when it is a symlink, because dotfile managers
/// commonly link a harness home and the guard must watch the directory behind
/// it. Entries below the root are never followed.
pub fn digest_tree(path: &Path) -> io::Result<TreeDigest> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(TreeDigest::Absent),
        Err(err) => return Err(err),
    };
    let mut hasher = DefaultHasher::new();
    hash_entry(&mut hasher, Path::new(""), &meta);
    if meta.is_dir() {
        walk(&mut hasher, path, Path::new(""))?;
    }
    Ok(TreeDigest::Present(hasher.finish()))
}

fn walk(hasher: &mut DefaultHasher, dir: &Path, relative: &Path) -> io::Result<()> {
    let mut names: Vec<OsString> = fs::read_dir(dir)?
        .map(|entry| entry.map(|e| e.file_name()))
        .collect::<io::Result<_>>()?;
    names.sort();
    for name in names {
        let path = dir.join(&name);
        let rel = relative.join(&name);
        // An entry removed between the listing and the stat (a live harness
        // session) is skipped; the parent's changed mtime still shows it.
        let meta = match fs::symlink_metadata(&path) {
            Ok(meta) => meta,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err),
        };
        hash_entry(hasher, &rel, &meta);
        if meta.is_dir() {
            walk(hasher, &path, &rel)?;
        }
    }
    Ok(())
}

fn hash_entry(hasher: &mut DefaultHasher, relative: &Path, meta: &fs::Metadata) {
    relative.hash(hasher);
    let kind: u8 = if meta.is_symlink() {
        1
    } else if meta.is_dir() {
        2
    } else if meta.is_file() {
        3
    } else {
        4
    };
    kind.hash(hasher);
    meta.len().hash(hasher);
    (meta.mtime(), meta.mtime_nsec()).hash(hasher);
    (meta.ctime(), meta.ctime_nsec()).hash(hasher);
    meta.ino().hash(hasher);
}

/// One harness home to watch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedHome {
    /// The harness name used in reports.
    pub name: &'static str,
    /// The directory to digest.
    pub path: PathBuf,
}

/// The real harness homes a test must leave untouched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealHomes {
    homes: Vec<NamedHome>,
}

impl RealHomes {
    /// Resolves the homes from the real environment of this process.
    pub fn ambient() -> Self {
        Self::resolve(|key| std::env::var_os(key))
    }

    /// Resolves the homes from `get`, so the rules are testable without
    /// touching the process environment.
    ///
    /// Claude Code uses `CLAUDE_CONFIG_DIR` or `~/.claude`, codex uses
    /// `CODEX_HOME` or `~/.codex`, and goose uses `GOOSE_PATH_ROOT` (absolute
    /// values only) or `~/.config/goose`. A home that cannot be resolved
    /// because `HOME` is unset is left out.
    pub fn resolve(get: impl Fn(&str) -> Option<OsString>) -> Self {
        let home = get("HOME").map(PathBuf::from);
        let pick = |var: &str, default: &[&str], absolute_only: bool| -> Option<PathBuf> {
            if let Some(value) = get(var).map(PathBuf::from)
                && (!absolute_only || value.is_absolute())
            {
                return Some(value);
            }
            home.as_ref()
                .map(|h| default.iter().fold(h.clone(), |acc, part| acc.join(part)))
        };
        let mut homes = Vec::new();
        for (name, var, default, absolute_only) in [
            ("claude-code", "CLAUDE_CONFIG_DIR", &[".claude"][..], false),
            ("codex", "CODEX_HOME", &[".codex"][..], false),
            ("goose", "GOOSE_PATH_ROOT", &[".config", "goose"][..], true),
        ] {
            if let Some(path) = pick(var, default, absolute_only) {
                homes.push(NamedHome { name, path });
            }
        }
        Self { homes }
    }

    /// Watches exactly these homes. Tests use this to stand in a fake "real"
    /// home so they never depend on the state of the host's own home.
    pub fn from_homes(homes: Vec<NamedHome>) -> Self {
        Self { homes }
    }

    /// The homes being watched.
    pub fn homes(&self) -> &[NamedHome] {
        &self.homes
    }
}

/// A harness home changed between snapshot and verify.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantViolation {
    changed: Vec<String>,
}

impl InvariantViolation {
    /// The harness homes whose digest differs, or that could not be read.
    pub fn changed(&self) -> &[String] {
        &self.changed
    }
}

impl fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "observation changed a real harness home: {}",
            self.changed.join(", ")
        )
    }
}

impl Error for InvariantViolation {}

/// Digests of the real homes taken before a run, to compare after it.
#[derive(Debug)]
pub struct RealHomeGuard {
    before: Vec<(NamedHome, TreeDigest)>,
}

impl RealHomeGuard {
    /// Digests every home now.
    pub fn snapshot(homes: &RealHomes) -> io::Result<Self> {
        let before = homes
            .homes()
            .iter()
            .map(|home| Ok((home.clone(), digest_tree(&home.path)?)))
            .collect::<io::Result<_>>()?;
        Ok(Self { before })
    }

    /// Digests every home again and names the ones that differ. A home that
    /// can no longer be read counts as changed.
    pub fn verify(&self) -> Result<(), InvariantViolation> {
        let changed: Vec<String> = self
            .before
            .iter()
            .filter(|(home, digest)| digest_tree(&home.path).ok() != Some(*digest))
            .map(|(home, _)| format!("{} ({})", home.name, home.path.display()))
            .collect();
        if changed.is_empty() {
            Ok(())
        } else {
            Err(InvariantViolation { changed })
        }
    }

    /// Fails the calling test when any home changed.
    pub fn assert_unchanged(&self) {
        if let Err(violation) = self.verify() {
            panic!("{violation}");
        }
    }
}

/// Runs `body` in a fresh scratch home and fails the test if any real harness
/// home changed while it ran. Returns what `body` returns.
pub fn observe<T>(homes: &RealHomes, body: impl FnOnce(&ScratchHome) -> T) -> T {
    assert!(
        !homes.homes().is_empty(),
        "observe was given no harness homes to watch, so it would pass without checking anything"
    );
    let scratch = ScratchHome::new().expect("a scratch home is made");
    let guard = RealHomeGuard::snapshot(homes).expect("the real homes can be read");
    let result = body(&scratch);
    guard.assert_unchanged();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    /// A fake "real" home inside its own scratch root, so no test reads the
    /// host's actual harness homes.
    fn fake_real(scratch: &ScratchHome) -> RealHomes {
        RealHomes::from_homes(vec![NamedHome {
            name: "fake-harness",
            path: scratch.home().join("real-harness-home"),
        }])
    }

    #[test]
    fn an_untouched_home_verifies() {
        let fake = ScratchHome::new().expect("fake real home root");
        let homes = fake_real(&fake);
        fs::create_dir_all(&homes.homes()[0].path).expect("the fake home is made");
        let guard = RealHomeGuard::snapshot(&homes).expect("snapshot");
        assert_eq!(guard.verify(), Ok(()));
    }

    #[test]
    fn positive_control_a_write_to_the_real_home_is_caught() {
        let fake = ScratchHome::new().expect("fake real home root");
        let homes = fake_real(&fake);
        let path = homes.homes()[0].path.clone();
        fs::create_dir_all(&path).expect("the fake home is made");
        let guard = RealHomeGuard::snapshot(&homes).expect("snapshot");
        fs::write(path.join("settings.json"), "{}").expect("a deliberate write");
        let violation = guard.verify().expect_err("the check must fire");
        assert_eq!(violation.changed().len(), 1);
        assert!(violation.changed()[0].starts_with("fake-harness"));
    }

    #[test]
    fn changing_an_existing_file_is_caught() {
        let fake = ScratchHome::new().expect("fake real home root");
        let homes = fake_real(&fake);
        let path = homes.homes()[0].path.clone();
        fs::create_dir_all(&path).expect("the fake home is made");
        fs::write(path.join("a"), "one").expect("seed file");
        let guard = RealHomeGuard::snapshot(&homes).expect("snapshot");
        fs::write(path.join("a"), "two").expect("same-size rewrite");
        assert!(guard.verify().is_err());
    }

    #[test]
    fn symlinks_below_the_root_are_not_followed() {
        use std::os::unix::fs::symlink;
        let fake = ScratchHome::new().expect("fake real home root");
        let homes = fake_real(&fake);
        let path = homes.homes()[0].path.clone();
        let outside = fake.home().join("outside");
        fs::create_dir_all(&outside).expect("outside dir");
        fs::write(outside.join("f"), "one").expect("outside file");
        fs::create_dir_all(&path).expect("the fake home is made");
        symlink(&outside, path.join("to-outside")).expect("link out");
        symlink(&path, path.join("loop")).expect("self link");
        let before = digest_tree(&path).expect("digest terminates on a loop");
        fs::write(outside.join("f"), "two").expect("change behind the link");
        assert_eq!(digest_tree(&path).expect("digest"), before);
        fs::remove_file(path.join("to-outside")).expect("drop the link");
        assert_ne!(digest_tree(&path).expect("digest"), before);
    }

    #[test]
    fn a_symlinked_home_root_is_followed() {
        use std::os::unix::fs::symlink;
        let fake = ScratchHome::new().expect("fake real home root");
        let target = fake.home().join("target");
        fs::create_dir_all(&target).expect("target dir");
        let link = fake.home().join("linked-home");
        symlink(&target, &link).expect("link the home");
        let before = digest_tree(&link).expect("digest");
        fs::write(target.join("new"), "x").expect("write through the link");
        assert_ne!(digest_tree(&link).expect("digest"), before);
    }

    #[test]
    #[should_panic(expected = "no harness homes")]
    fn observe_refuses_an_empty_home_list() {
        observe(&RealHomes::from_homes(vec![]), |_| ());
    }

    #[test]
    fn a_home_that_appears_later_is_a_change() {
        let fake = ScratchHome::new().expect("fake real home root");
        let homes = fake_real(&fake);
        let path = homes.homes()[0].path.clone();
        let guard = RealHomeGuard::snapshot(&homes).expect("an absent home snapshots");
        assert_eq!(digest_tree(&path).expect("digest"), TreeDigest::Absent);
        assert_eq!(guard.verify(), Ok(()));
        fs::create_dir_all(&path).expect("the home appears");
        assert!(guard.verify().is_err());
    }

    #[test]
    fn observe_passes_when_the_body_stays_in_the_scratch_home() {
        let fake = ScratchHome::new().expect("fake real home root");
        let homes = fake_real(&fake);
        fs::create_dir_all(&homes.homes()[0].path).expect("the fake home is made");
        let status = observe(&homes, |scratch| {
            scratch
                .command("sh")
                .args(["-c", "echo observed > \"$CLAUDE_CONFIG_DIR/state\""])
                .status()
                .expect("sh runs")
        });
        assert!(status.success());
    }

    #[test]
    fn observe_fails_the_test_when_the_body_writes_to_the_real_home() {
        let fake = ScratchHome::new().expect("fake real home root");
        let homes = fake_real(&fake);
        let path = homes.homes()[0].path.clone();
        fs::create_dir_all(&path).expect("the fake home is made");
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            observe(&homes, |_| {
                fs::write(path.join("leak"), "x").expect("a deliberate write");
            });
        }));
        assert!(outcome.is_err(), "observe must panic on a changed home");
    }

    #[test]
    fn homes_resolve_from_the_environment_with_defaults_under_home() {
        let env = |key: &str| -> Option<OsString> {
            match key {
                "HOME" => Some("/h".into()),
                "CODEX_HOME" => Some("/elsewhere/codex".into()),
                "GOOSE_PATH_ROOT" => Some("relative/goose".into()),
                _ => None,
            }
        };
        let homes = RealHomes::resolve(env);
        let by_name = |name: &str| {
            homes
                .homes()
                .iter()
                .find(|h| h.name == name)
                .map(|h| h.path.clone())
        };
        assert_eq!(by_name("claude-code"), Some(PathBuf::from("/h/.claude")));
        assert_eq!(by_name("codex"), Some(PathBuf::from("/elsewhere/codex")));
        // A relative GOOSE_PATH_ROOT is ignored by goose, so the default applies.
        assert_eq!(by_name("goose"), Some(PathBuf::from("/h/.config/goose")));
    }

    #[test]
    fn homes_that_cannot_be_resolved_are_left_out() {
        let homes = RealHomes::resolve(|_| None);
        assert!(homes.homes().is_empty());
    }
}
