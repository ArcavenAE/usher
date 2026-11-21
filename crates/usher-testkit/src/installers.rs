//! The observation invariant for installer state (story 2.1): a narrowed digest of what
//! brew and mise keep on disk, so a test can prove `usher tools` changed none of it.
//!
//! The digest is narrow on purpose: whole trees such as the Cellar are large, and a brew
//! or mise run by the person while a test runs reads as a violation, which is a true
//! report and not a flake to retry.

use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    io,
    path::Path,
    time::UNIX_EPOCH,
};

use usher_core::InstallerPaths;

/// The narrowed digest of installer state. Equal digests mean nothing watched changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerDigest(u64);

/// Digests the watched parts of installer state.
pub fn digest_installers(paths: &InstallerPaths) -> io::Result<InstallerDigest> {
    let mut hasher = DefaultHasher::new();
    match &paths.brew {
        Some(brew) => {
            "brew".hash(&mut hasher);
            file_text(&brew.repository.join(".git/HEAD")).hash(&mut hasher);
            file_text(&brew.repository.join(".git/FETCH_HEAD")).hash(&mut hasher);
            names(&brew.repository.join("Library/Taps"), 2).hash(&mut hasher);
            names(&brew.prefix.join("Cellar"), 2).hash(&mut hasher);
            names(&brew.prefix.join("Caskroom"), 2).hash(&mut hasher);
            names(&brew.prefix.join("var/homebrew/linked"), 1).hash(&mut hasher);
            timed(&brew.cache.join("api")).hash(&mut hasher);
        }
        None => {
            "no brew".hash(&mut hasher);
            for candidate in &paths.brew_candidates {
                (candidate, candidate.exists()).hash(&mut hasher);
            }
        }
    }
    names(&paths.mise_data.join("installs"), 2).hash(&mut hasher);
    names(&paths.mise_data.join("shims"), 1).hash(&mut hasher);
    timed(&paths.mise_cache).hash(&mut hasher);
    timed(&paths.mise_state).hash(&mut hasher);
    Ok(InstallerDigest(hasher.finish()))
}

/// The text of a file, or nothing when it is absent or unreadable.
fn file_text(path: &Path) -> Option<Vec<u8>> {
    fs::read(path).ok()
}

/// The entry names under `root` down to `depth` levels, sorted; empty when `root` is
/// absent.
fn names(root: &Path, depth: usize) -> Vec<String> {
    let mut found = Vec::new();
    collect(root, "", depth, &mut found);
    found.sort();
    found
}

fn collect(dir: &Path, prefix: &str, depth: usize, found: &mut Vec<String>) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
        found.push(name.clone());
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            collect(&entry.path(), &format!("{name}/"), depth - 1, found);
        }
    }
}

/// The entries directly under `dir` with each one's size and modification time, sorted
/// by name; empty when `dir` is absent.
fn timed(dir: &Path) -> Vec<(String, u64, u128)> {
    let mut found = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let (size, nanos) = entry.metadata().map_or((0, 0), |meta| {
                let nanos = meta
                    .modified()
                    .ok()
                    .and_then(|when| when.duration_since(UNIX_EPOCH).ok())
                    .map_or(0, |elapsed| elapsed.as_nanos());
                (meta.len(), nanos)
            });
            found.push((
                entry.file_name().to_string_lossy().into_owned(),
                size,
                nanos,
            ));
        }
    }
    found.sort();
    found
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
        let after = digest_installers(&self.paths)
            .map_err(|err| format!("installer state could not be read again: {err}"))?;
        if after == self.before {
            Ok(())
        } else {
            Err(
                "installer state changed since the snapshot: a watched brew or mise path differs"
                    .to_string(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::symlink, path::Path, time::Duration};

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
            brew_candidates: vec![],
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
            b(&paths).prefix.join("Caskroom/app/1.0"),
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
        fs::write(paths.mise_state.join("s"), "1").expect("mise state file");
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
            brew_candidates: vec![],
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
    fn a_new_tap_under_an_existing_org_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        fs::create_dir_all(
            b(&rig.paths)
                .repository
                .join("Library/Taps/acme/homebrew-more"),
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
    fn a_file_directly_inside_a_cellar_version_is_not_watched() {
        let rig = rig();
        let before = digest(&rig);
        write(&b(&rig.paths).prefix.join("Cellar/jq/1.7.1/README"), "x");
        assert_eq!(
            before,
            digest(&rig),
            "the digest reads names to depth 2 only"
        );
    }

    #[test]
    fn a_new_cask_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        fs::create_dir_all(b(&rig.paths).prefix.join("Caskroom/other/1.0")).expect("cask");
        assert_ne!(before, digest(&rig));
    }

    #[test]
    fn a_new_version_of_an_existing_cask_is_a_change() {
        let rig = rig();
        let before = digest(&rig);
        fs::create_dir_all(b(&rig.paths).prefix.join("Caskroom/app/2.0")).expect("version");
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

    /// The files whose size and modification time are both read.
    fn timed_files(rig: &Rig) -> Vec<std::path::PathBuf> {
        vec![
            b(&rig.paths).cache.join("api/formula.json"),
            rig.paths.mise_cache.join("c"),
            rig.paths.mise_state.join("s"),
        ]
    }

    fn mtime(path: &Path) -> std::time::SystemTime {
        fs::metadata(path)
            .expect("metadata")
            .modified()
            .expect("mtime")
    }

    fn set_mtime(path: &Path, when: std::time::SystemTime) {
        fs::File::options()
            .write(true)
            .open(path)
            .expect("open")
            .set_modified(when)
            .expect("set mtime");
    }

    #[test]
    fn a_rewrite_of_the_same_size_with_a_later_mtime_is_a_change() {
        let rig = rig();
        for path in timed_files(&rig) {
            let before = digest(&rig);
            let old = mtime(&path);
            let text = fs::read_to_string(&path).expect("text");
            fs::write(&path, text.replace(|_| true, "z")).expect("same size");
            set_mtime(&path, old + Duration::from_secs(60));
            assert_ne!(before, digest(&rig), "{}", path.display());
        }
    }

    #[test]
    fn a_rewrite_to_a_new_size_that_keeps_the_mtime_is_a_change() {
        let rig = rig();
        for path in timed_files(&rig) {
            let before = digest(&rig);
            let old = mtime(&path);
            fs::write(&path, "a longer text than before").expect("new size");
            set_mtime(&path, old);
            assert_ne!(before, digest(&rig), "{}", path.display());
        }
    }

    #[test]
    fn an_absent_brew_digests_and_a_mise_install_is_still_a_change() {
        let rig = rig();
        let no_brew = InstallerPaths {
            brew: None,
            brew_candidates: vec![],
            mise_data: rig.paths.mise_data.clone(),
            mise_cache: rig.paths.mise_cache.clone(),
            mise_state: rig.paths.mise_state.clone(),
        };
        let before = digest_installers(&no_brew).expect("a digest without brew");
        fs::create_dir_all(rig.paths.mise_data.join("installs/go/1.25.0")).expect("install");
        assert_ne!(before, digest_installers(&no_brew).expect("a digest"));
    }

    #[test]
    fn with_no_brew_a_candidate_location_that_appears_is_a_change_and_fails_the_guard() {
        let rig = rig();
        let candidate = rig.paths.mise_data.join("candidate-a/Homebrew");
        let other = rig.paths.mise_data.join("candidate-b/Homebrew");
        let no_brew = InstallerPaths {
            brew: None,
            brew_candidates: vec![candidate.clone(), other],
            mise_data: rig.paths.mise_data.clone(),
            mise_cache: rig.paths.mise_cache.clone(),
            mise_state: rig.paths.mise_state.clone(),
        };
        let before = digest_installers(&no_brew).expect("absent candidates are not an error");
        let guard = InstallerGuard::snapshot(&no_brew).expect("a snapshot");
        fs::create_dir_all(&candidate).expect("a brew appears");
        assert_ne!(before, digest_installers(&no_brew).expect("a digest"));
        let err = guard.verify().expect_err("a created candidate must fail");
        assert!(err.contains("installer state changed"), "{err}");
    }
}
