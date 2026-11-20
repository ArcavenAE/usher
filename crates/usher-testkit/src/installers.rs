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

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::symlink, path::Path};

    use usher_core::BrewPaths;

    use super::*;
    use crate::ScratchHome;

    struct Rig {
        _scratch: ScratchHome,
        paths: InstallerPaths,
    }

    fn rig() -> Rig {
        let scratch = ScratchHome::new().expect("scratch");
        let root = scratch.root().to_path_buf();
        let paths = InstallerPaths {
            brew: Some(BrewPaths {
                repository: root.join("brew"),
                prefix: root.join("brew"),
                cache: root.join("brew-cache"),
            }),
            mise_data: root.join("mise/data"),
            mise_cache: root.join("mise/cache"),
            mise_state: root.join("mise/state"),
        };
        for dir in [
            b(&paths).repository.join(".git"),
            b(&paths)
                .repository
                .join("Library/Taps/acme/homebrew-tools"),
            b(&paths).prefix.join("Cellar/jq/1.7.1/bin"),
            b(&paths).prefix.join("Caskroom"),
            b(&paths).prefix.join("var/homebrew/linked"),
            b(&paths).cache.join("api"),
            paths.mise_data.join("installs/go/1.23.0/bin"),
            paths.mise_data.join("shims"),
            paths.mise_cache.clone(),
            paths.mise_state.clone(),
        ] {
            fs::create_dir_all(dir).expect("dir");
        }
        fs::write(
            b(&paths).repository.join(".git/HEAD"),
            "ref: refs/heads/aaa",
        )
        .expect("head");
        fs::write(b(&paths).repository.join(".git/FETCH_HEAD"), "one").expect("fetch head");
        fs::write(b(&paths).cache.join("api/formula.json"), "12345").expect("api");
        fs::write(paths.mise_cache.join("c"), "1").expect("mise cache file");
        Rig {
            _scratch: scratch,
            paths,
        }
    }

    fn b(paths: &InstallerPaths) -> &BrewPaths {
        paths.brew.as_ref().expect("brew paths")
    }

    fn digest(rig: &Rig) -> InstallerDigest {
        digest_installers(&rig.paths).expect("a digest")
    }

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().expect("a parent")).expect("dirs");
        fs::write(path, text).expect("write");
    }

    #[test]
    fn an_untouched_installer_state_digests_the_same_twice() {
        let rig = rig();
        assert_eq!(digest(&rig), digest(&rig));
    }

    #[test]
    fn a_state_that_is_absent_digests_without_error_and_creating_it_is_a_change() {
        let rig = rig();
        let missing = InstallerPaths {
            brew: Some(BrewPaths {
                repository: b(&rig.paths).repository.join("none"),
                prefix: b(&rig.paths).prefix.join("none"),
                cache: b(&rig.paths).cache.join("none"),
            }),
            mise_data: rig.paths.mise_data.join("none"),
            mise_cache: rig.paths.mise_cache.join("none"),
            mise_state: rig.paths.mise_state.join("none"),
        };
        let before = digest_installers(&missing).expect("absent state is not an error");
        fs::create_dir_all(missing.mise_data.join("installs/go/1.0.0")).expect("create");
        assert_ne!(before, digest_installers(&missing).expect("a digest"));
    }

    #[test]
    fn the_contents_of_the_repository_head_are_digested() {
        let rig = rig();
        let before = digest(&rig);
        write(
            &b(&rig.paths).repository.join(".git/HEAD"),
            "ref: refs/heads/bbb",
        );
        assert_ne!(before, digest(&rig), "same size, different content");
    }

    #[test]
    fn the_contents_of_the_fetch_head_are_digested() {
        let rig = rig();
        let before = digest(&rig);
        write(&b(&rig.paths).repository.join(".git/FETCH_HEAD"), "two");
        assert_ne!(before, digest(&rig));
    }

    #[test]
    fn a_new_tap_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        fs::create_dir_all(
            b(&rig.paths)
                .repository
                .join("Library/Taps/other/homebrew-extras"),
        )
        .expect("tap");
        assert_ne!(before, digest(&rig));
    }

    #[test]
    fn a_new_cellar_version_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        fs::create_dir_all(b(&rig.paths).prefix.join("Cellar/jq/1.7.2")).expect("version");
        assert_ne!(before, digest(&rig));
    }

    #[test]
    fn a_file_deep_inside_a_cellar_version_is_not_watched() {
        let rig = rig();
        let before = digest(&rig);
        write(&b(&rig.paths).prefix.join("Cellar/jq/1.7.1/bin/jq"), "x");
        assert_eq!(
            before,
            digest(&rig),
            "the digest reads names to depth 2 only"
        );
    }

    #[test]
    fn a_new_cask_version_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        fs::create_dir_all(b(&rig.paths).prefix.join("Caskroom/app/3.0")).expect("cask");
        assert_ne!(before, digest(&rig));
    }

    #[test]
    fn a_new_link_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        symlink(
            "../../../Cellar/jq/1.7.1",
            b(&rig.paths).prefix.join("var/homebrew/linked/jq"),
        )
        .expect("link");
        assert_ne!(before, digest(&rig));
    }

    #[test]
    fn a_rewritten_formula_index_in_the_brew_cache_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        write(&b(&rig.paths).cache.join("api/formula.json"), "123456789");
        assert_ne!(before, digest(&rig), "size or mtime differs");
    }

    #[test]
    fn a_new_mise_install_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        fs::create_dir_all(rig.paths.mise_data.join("installs/go/1.24.0")).expect("install");
        assert_ne!(before, digest(&rig));
    }

    #[test]
    fn a_new_shim_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        write(&rig.paths.mise_data.join("shims/go"), "#!/bin/sh");
        assert_ne!(before, digest(&rig));
    }

    #[test]
    fn a_changed_mise_cache_file_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        write(&rig.paths.mise_cache.join("c"), "22222");
        assert_ne!(before, digest(&rig));
    }

    #[test]
    fn a_changed_mise_state_directory_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        write(&rig.paths.mise_state.join("trusted"), "x");
        assert_ne!(before, digest(&rig));
    }

    #[test]
    fn the_guard_passes_when_nothing_changed() {
        let rig = rig();
        let guard = InstallerGuard::snapshot(&rig.paths).expect("a snapshot");
        guard.verify().expect("nothing changed");
    }

    #[test]
    fn the_guard_fails_naming_the_change_when_a_watched_path_changes() {
        let rig = rig();
        let guard = InstallerGuard::snapshot(&rig.paths).expect("a snapshot");
        fs::create_dir_all(b(&rig.paths).prefix.join("Cellar/jq/1.7.2")).expect("version");
        let err = guard.verify().expect_err("a positive control must fire");
        assert!(err.contains("installer state changed"), "{err}");
    }
}
