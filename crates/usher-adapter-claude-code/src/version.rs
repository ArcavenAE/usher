//! Reading the installed Claude Code version from disk, never by running it.
//!
//! Knowledge names the launcher, the rule and the layout (`version_from`). The reader
//! reads the launcher link once, with no canonicalize, resolves a relative target
//! against the launcher's directory, and matches the result to the layout component by
//! component; the last component is the version.

use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{env, fs};

use usher_core::{LaunchContext, SystemRoot};
use usher_knowledge::{Version, VersionFrom};

use crate::paths::{normalize, resolve};

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
    match read(from, context, path_var) {
        Ok(version) => VersionReading::Known(version),
        Err(reason) => VersionReading::Unknown { reason },
    }
}

fn read(
    from: &VersionFrom,
    context: &LaunchContext,
    path_var: Option<&OsStr>,
) -> Result<String, String> {
    let host = SystemRoot::host();
    let launcher = resolve(
        from.launcher(),
        context,
        &host,
        "the config directory variable",
    )
    .map_err(|why| format!("the launcher path cannot be placed: {why}"))?;
    let meta = fs::symlink_metadata(&launcher)
        .map_err(|why| format!("the launcher {} cannot be read: {why}", launcher.display()))?;
    if !meta.file_type().is_symlink() {
        return Err(format!(
            "the launcher {} is not a symlink",
            launcher.display()
        ));
    }
    let raw = fs::read_link(&launcher)
        .map_err(|why| format!("the launcher {} cannot be read: {why}", launcher.display()))?;
    let target = if raw.is_absolute() {
        raw
    } else {
        launcher.parent().unwrap_or(Path::new("/")).join(raw)
    };
    let target = normalize(&target);

    let (layout_dir, _) = from
        .layout()
        .rsplit_once('/')
        .ok_or_else(|| "the knowledge layout has no directory".to_string())?;
    let layout_dir = resolve(layout_dir, context, &host, "the config directory variable")
        .map_err(|why| format!("the layout cannot be placed: {why}"))?;
    let layout_dir = normalize(&layout_dir);
    let mismatch = || {
        format!(
            "the launcher target {} does not match the layout {}",
            target.display(),
            from.layout()
        )
    };
    if target.parent() != Some(layout_dir.as_path()) {
        return Err(mismatch());
    }
    let last = target
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(mismatch)?;
    let version = Version::parse(last).map_err(|_| {
        format!(
            "the last component `{last}` of {} is not a version",
            target.display()
        )
    })?;

    match fs::metadata(&target) {
        Ok(meta) if meta.is_file() => {}
        Ok(_) => {
            return Err(format!(
                "the launcher target {} is not a file",
                target.display()
            ));
        }
        Err(why) => {
            return Err(format!(
                "the launcher link is dangling: {} cannot be read: {why}",
                target.display()
            ));
        }
    }
    check_path_claude(&launcher, context, path_var)?;
    Ok(version.to_string())
}

/// The first `claude` on `PATH` that `sh` would run: a regular file with an execute bit,
/// found by the same walk. An empty entry is the launch directory and a relative one is
/// joined to it. It must be the launcher, either by path or as a link straight to it.
fn check_path_claude(
    launcher: &Path,
    context: &LaunchContext,
    path_var: Option<&OsStr>,
) -> Result<(), String> {
    let name = launcher.file_name().unwrap_or(OsStr::new("claude"));
    let launch_dir = context.key.directory();
    let hit = path_var
        .into_iter()
        .flat_map(env::split_paths)
        .map(|entry| {
            if entry.as_os_str().is_empty() {
                launch_dir.to_path_buf()
            } else if entry.is_relative() {
                launch_dir.join(entry)
            } else {
                entry
            }
        })
        .map(|dir| dir.join(name))
        .find(|candidate| {
            fs::metadata(candidate)
                .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        });
    let Some(hit) = hit else {
        return Err(format!(
            "{} is not on PATH, so it is not known to be the claude that runs",
            name.to_string_lossy()
        ));
    };
    let is_launcher = hit == launcher
        || fs::read_link(&hit)
            .map(|raw| {
                let absolute = if raw.is_absolute() {
                    raw
                } else {
                    hit.parent().unwrap_or(Path::new("/")).join(raw)
                };
                normalize(&absolute) == launcher
            })
            .unwrap_or(false);
    if is_launcher {
        Ok(())
    } else {
        Err(format!(
            "{} on PATH is {}, which is not the native launcher {}",
            name.to_string_lossy(),
            hit.display(),
            launcher.display()
        ))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
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

    /// A real install is executable, and `sh` skips a `PATH` entry that is not.
    fn write_exec(path: &Path, contents: &[u8]) {
        fs::write(path, contents).expect("file");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("mode");
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
        write_exec(&target, b"binary");
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
        write_exec(&elsewhere, b"binary");
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
        write_exec(&launcher, b"#!/bin/sh\n");
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
        let reason = unknown(&reading);
        assert!(reason.contains("dangling"), "{reason}");
        assert!(
            reason.contains(&versions(&scratch).join("2.1.317").display().to_string()),
            "the reason names the target: {reason}"
        );
    }

    #[test]
    fn a_claude_on_path_that_is_not_the_launcher_reads_as_unknown_naming_it() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let other = scratch.home().join("other-bin");
        fs::create_dir_all(&other).expect("dirs");
        write_exec(&other.join("claude"), b"#!/bin/sh\n");
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
    /// The version comes only from a target that is a regular file.
    #[test]
    fn a_target_that_is_a_directory_reads_as_unknown_saying_it_is_not_a_file() {
        let scratch = ScratchHome::new().expect("scratch");
        let target = versions(&scratch).join("2.1.317");
        fs::create_dir_all(&target).expect("dirs");
        link(&scratch, &target);
        let reading = read_version(&from(), &context(&scratch), Some(&path_of(&scratch)));
        assert!(unknown(&reading).contains("not a file"), "{reading:?}");
    }

    #[test]
    fn a_path_entry_that_links_to_the_launcher_counts_as_the_launcher() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let other = scratch.home().join("other-bin");
        fs::create_dir_all(&other).expect("dirs");
        symlink(launcher(&scratch), other.join("claude")).expect("symlink");
        let reading = read_version(&from(), &context(&scratch), Some(other.as_os_str()));
        assert_eq!(reading, VersionReading::Known("2.1.317".to_string()));
    }

    #[test]
    fn a_path_entry_that_links_to_another_file_reads_as_unknown_naming_the_hit() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let other = scratch.home().join("other-bin");
        fs::create_dir_all(&other).expect("dirs");
        let elsewhere = scratch.home().join("elsewhere");
        write_exec(&elsewhere, b"#!/bin/sh\n");
        symlink(&elsewhere, other.join("claude")).expect("symlink");
        let reading = read_version(&from(), &context(&scratch), Some(other.as_os_str()));
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

    /// `sh` passes over a dangling `claude`, so the lookup does too.
    #[test]
    fn a_dangling_claude_earlier_on_path_is_passed_over() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let broken = scratch.home().join("broken-bin");
        fs::create_dir_all(&broken).expect("dirs");
        symlink(scratch.home().join("gone"), broken.join("claude")).expect("symlink");
        let path =
            std::env::join_paths([broken, scratch.home().join(".local/bin")]).expect("a path");
        let reading = read_version(&from(), &context(&scratch), Some(&path));
        assert_eq!(reading, VersionReading::Known("2.1.317".to_string()));
    }

    #[test]
    fn a_path_whose_only_claude_is_dangling_reads_as_not_on_path() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let broken = scratch.home().join("broken-bin");
        fs::create_dir_all(&broken).expect("dirs");
        symlink(scratch.home().join("gone"), broken.join("claude")).expect("symlink");
        let reading = read_version(&from(), &context(&scratch), Some(broken.as_os_str()));
        assert!(unknown(&reading).contains("not on PATH"), "{reading:?}");
    }

    /// `sh` skips a `claude` without an execute bit, so the lookup does too.
    #[test]
    fn a_claude_without_an_execute_bit_earlier_on_path_is_passed_over() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let decoy = scratch.home().join("decoy-bin");
        fs::create_dir_all(&decoy).expect("dirs");
        fs::write(decoy.join("claude"), b"#!/bin/sh\n").expect("file");
        fs::set_permissions(decoy.join("claude"), fs::Permissions::from_mode(0o644)).expect("mode");
        let path =
            std::env::join_paths([decoy, scratch.home().join(".local/bin")]).expect("a path");
        let reading = read_version(&from(), &context(&scratch), Some(&path));
        assert_eq!(reading, VersionReading::Known("2.1.317".to_string()));
    }

    /// `sh` only runs a regular file, so a directory named `claude` is passed over.
    #[test]
    fn a_directory_named_claude_earlier_on_path_is_passed_over() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let decoy = scratch.home().join("dir-bin");
        fs::create_dir_all(decoy.join("claude")).expect("dirs");
        let path =
            std::env::join_paths([decoy, scratch.home().join(".local/bin")]).expect("a path");
        let reading = read_version(&from(), &context(&scratch), Some(&path));
        assert_eq!(reading, VersionReading::Known("2.1.317".to_string()));
    }

    /// An empty entry is the launch directory, never usher's own working directory.
    #[test]
    fn an_empty_path_entry_means_the_launch_directory() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let in_launch_dir = scratch.home().join("claude");
        write_exec(&in_launch_dir, b"#!/bin/sh\n");
        let mut path = std::ffi::OsString::from(":");
        path.push(scratch.home().join(".local/bin"));
        let reading = read_version(&from(), &context(&scratch), Some(&path));
        let reason = unknown(&reading);
        assert!(
            reason.contains(&format!("claude on PATH is {}", in_launch_dir.display())),
            "{reason}"
        );
        assert!(reason.contains("PATH"), "{reason}");
    }

    #[test]
    fn a_relative_path_entry_is_joined_to_the_launch_directory() {
        let scratch = ScratchHome::new().expect("scratch");
        link(&scratch, &install(&scratch, "2.1.317"));
        let bin = scratch.home().join("bin");
        fs::create_dir_all(&bin).expect("dirs");
        write_exec(&bin.join("claude"), b"#!/bin/sh\n");
        let mut path = std::ffi::OsString::from("bin:");
        path.push(scratch.home().join(".local/bin"));
        let reading = read_version(&from(), &context(&scratch), Some(&path));
        let reason = unknown(&reading);
        assert!(
            reason.contains(&format!(
                "claude on PATH is {}",
                bin.join("claude").display()
            )),
            "{reason}"
        );
    }
}
