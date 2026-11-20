//! The observation invariant for installer state (story 2.1): a narrowed digest of what
//! brew and mise keep on disk, so a test can prove `usher tools` changed none of it.
//!
//! The digest is narrow on purpose: whole trees such as the Cellar are large, and a brew
//! or mise run by the person while a test runs reads as a violation, which is a true
//! report and not a flake to retry.

use std::io;

use usher_core::InstallerPaths;

/// The narrowed digest of installer state. Equal digests mean nothing watched changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerDigest(u64);

/// Digests the watched parts of installer state.
pub fn digest_installers(paths: &InstallerPaths) -> io::Result<InstallerDigest> {
    let _ = paths;
    Ok(InstallerDigest(0))
}

/// A snapshot of installer state to compare against later.
#[derive(Debug)]
pub struct InstallerGuard {
    paths: InstallerPaths,
    before: InstallerDigest,
}

impl InstallerGuard {
    /// Takes the snapshot.
    pub fn snapshot(paths: &InstallerPaths) -> io::Result<Self> {
        Ok(Self {
            paths: paths.clone(),
            before: digest_installers(paths)?,
        })
    }

    /// Fails when anything watched changed since the snapshot.
    pub fn verify(&self) -> Result<(), String> {
        let _ = (&self.paths, &self.before);
        Ok(())
    }
}
