//! Reading the installed Claude Code version from disk, never by running it.
//!
//! Knowledge names the launcher, the rule and the layout (`version_from`). The reader
//! reads the launcher link once, with no canonicalize, resolves a relative target
//! against the launcher's directory, and matches the result to the layout component by
//! component; the last component is the version.

use std::ffi::OsStr;

use usher_core::LaunchContext;
use usher_knowledge::VersionFrom;

/// What the reader found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionReading {
    /// The version, as `major.minor.patch`.
    Known(String),
    /// The version is not known, and why.
    Unknown {
        /// Why the version reads as unknown.
        reason: String,
    },
}

/// Reads the version for one launch context. `path_var` is the `PATH` of the
/// environment being inspected.
pub fn read_version(
    from: &VersionFrom,
    context: &LaunchContext,
    path_var: Option<&OsStr>,
) -> VersionReading {
    let _ = (from, context, path_var);
    VersionReading::Unknown {
        reason: String::new(),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};

    use usher_core::{LaunchContext, environment_key};
    use usher_knowledge::KnowledgeFile;
    use usher_testkit::ScratchHome;

    use super::*;

    fn from() -> VersionFrom {
        let file = KnowledgeFile::embedded().expect("the embedded file parses");
        file.items()
            .iter()
            .find(|item| item.id() == "claude-code.native-launcher")
            .and_then(|item| item.version_from())
            .expect("the seed says how to read the version")
            .clone()
    }

    fn context(scratch: &ScratchHome) -> LaunchContext {
        let home = scratch.home().to_path_buf();
        let key = environment_key("host", "alex", "claude-code", &home, &[])
            .expect("an absolute directory");
        LaunchContext {
            key,
            config_dir: home.join(".claude"),
            home,
            repo: None,
            user: "alex".to_string(),
            harness_version: None,
        }
    }

    fn launcher(scratch: &ScratchHome) -> PathBuf {
        scratch.home().join(".local/bin/claude")
    }

    fn versions(scratch: &ScratchHome) -> PathBuf {
        scratch.home().join(".local/share/claude/versions")
    }

    /// Makes `versions/<name>` a file and returns it.
    fn install(scratch: &ScratchHome, name: &str) -> PathBuf {
        let target = versions(scratch).join(name);
        fs::create_dir_all(target.parent().expect("a parent")).expect("dirs");
        fs::write(&target, b"binary").expect("file");
        target
    }

    fn link(scratch: &ScratchHome, target: &Path) {
        let launcher = launcher(scratch);
        fs::create_dir_all(launcher.parent().expect("a parent")).expect("dirs");
        symlink(target, launcher).expect("symlink");
    }

    fn path_of(scratch: &ScratchHome) -> std::ffi::OsString {
        scratch.home().join(".local/bin").into_os_string()
    }

    fn unknown(reading: &VersionReading) -> &str {
        match reading {
            VersionReading::Unknown { reason } => reason,
            VersionReading::Known(version) => panic!("expected unknown, read {version}"),
        }
    }

    #[test]
    fn a_link_whose_target_matches_the_layout_gives_its_last_component() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let reading = read_version(&from(), &context(&scratch), Some(&path_of(&scratch)));
        assert_eq!(reading, VersionReading::Known("2.1.317".to_string()));
    }

    #[test]
    fn a_relative_target_resolves_against_the_launcher_directory() {
        let scratch = ScratchHome::new().expect("scratch");
        install(&scratch, "2.1.317");
        link(&scratch, Path::new("../share/claude/versions/2.1.317"));
        let reading = read_version(&from(), &context(&scratch), Some(&path_of(&scratch)));
        assert_eq!(reading, VersionReading::Known("2.1.317".to_string()));
    }

    #[test]
    fn a_target_outside_the_layout_reads_as_unknown_naming_the_layout() {
        let scratch = ScratchHome::new().expect("scratch");
        let elsewhere = scratch.home().join("elsewhere/2.1.317");
        fs::create_dir_all(elsewhere.parent().expect("a parent")).expect("dirs");
        fs::write(&elsewhere, b"binary").expect("file");
        link(&scratch, &elsewhere);
        let reading = read_version(&from(), &context(&scratch), Some(&path_of(&scratch)));
        assert!(unknown(&reading).contains("layout"), "{reading:?}");
    }

    #[test]
    fn a_last_component_that_is_not_a_version_reads_as_unknown_naming_it() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "latest"));
        let reading = read_version(&from(), &context(&scratch), Some(&path_of(&scratch)));
        assert!(unknown(&reading).contains("latest"), "{reading:?}");
    }

    #[test]
    fn a_missing_launcher_reads_as_unknown_naming_the_launcher() {
        let scratch = ScratchHome::new().expect("scratch");
        let reading = read_version(&from(), &context(&scratch), Some(&path_of(&scratch)));
        assert!(
            unknown(&reading).contains(".local/bin/claude"),
            "{reading:?}"
        );
    }

    #[test]
    fn a_launcher_that_is_not_a_link_reads_as_unknown_saying_so() {
        let scratch = ScratchHome::new().expect("scratch");
        let launcher = launcher(&scratch);
        fs::create_dir_all(launcher.parent().expect("a parent")).expect("dirs");
        fs::write(&launcher, b"#!/bin/sh\n").expect("file");
        let reading = read_version(&from(), &context(&scratch), Some(&path_of(&scratch)));
        assert!(unknown(&reading).contains("not a symlink"), "{reading:?}");
    }

    /// A link whose target matches the layout but whose file is gone is a broken
    /// install, so the version it names is not known to be there.
    #[test]
    fn a_dangling_link_that_matches_the_layout_reads_as_unknown_saying_so() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &versions(&scratch).join("2.1.317"));
        let reading = read_version(&from(), &context(&scratch), Some(&path_of(&scratch)));
        assert!(unknown(&reading).contains("dangling"), "{reading:?}");
    }

    #[test]
    fn a_claude_on_path_that_is_not_the_launcher_reads_as_unknown_naming_it() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let other = scratch.home().join("other-bin");
        fs::create_dir_all(&other).expect("dirs");
        fs::write(other.join("claude"), b"#!/bin/sh\n").expect("file");
        let path = std::env::join_paths([other.clone(), scratch.home().join(".local/bin")])
            .expect("a path");
        let reading = read_version(&from(), &context(&scratch), Some(&path));
        let reason = unknown(&reading);
        assert!(
            reason.contains(&format!(
                "claude on PATH is {}",
                other.join("claude").display()
            )),
            "{reason}"
        );
        assert!(reason.contains("not the native launcher"), "{reason}");
    }

    #[test]
    fn no_claude_on_path_reads_as_unknown_saying_so() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let empty = scratch.home().join("empty-bin");
        fs::create_dir_all(&empty).expect("dirs");
        let reading = read_version(&from(), &context(&scratch), Some(empty.as_os_str()));
        assert!(unknown(&reading).contains("not on PATH"), "{reading:?}");
        let reading = read_version(&from(), &context(&scratch), None);
        assert!(unknown(&reading).contains("not on PATH"), "{reading:?}");
    }
}
