//! Pinned harness binaries: find one, run it under a scratch home, record the
//! version, or report why it is missing.
//!
//! cargo test has no skipped state: a test that returns early reports `ok`.
//! So a missing binary is never silent. [`skip`] prints `SKIPPED: <test>:
//! <reason>` on stderr and prints the list of every skip so far, and
//! [`skipped`] returns the same list. The standard test harness has no
//! end-of-run hook, so the last list printed in a run is the complete one.

use std::ffi::{OsStr, OsString};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::{env, fs};

use crate::scratch::ScratchHome;

/// The result of looking for a harness binary and asking it for its version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    /// The binary was found and ran.
    Found {
        /// Where it was found.
        path: PathBuf,
        /// The first line it printed for `--version`.
        version: String,
    },
    /// The binary could not be used; the reason names why.
    Missing {
        /// A plain statement of what was missing or what failed.
        reason: String,
    },
}

/// One recorded skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skip {
    /// The test that skipped.
    pub test: String,
    /// Why it skipped.
    pub reason: String,
}

static SKIPS: Mutex<Vec<Skip>> = Mutex::new(Vec::new());

/// Looks for `name` on `search_path` (or on `PATH` when `None`) and runs
/// `<binary> --version` under `scratch`, so the run cannot touch the real home.
pub fn probe(name: &str, search_path: Option<&OsStr>, scratch: &ScratchHome) -> Probe {
    let search: OsString = match search_path {
        Some(path) => path.to_os_string(),
        None => env::var_os("PATH").unwrap_or_default(),
    };
    let Some(path) = env::split_paths(&search)
        .map(|dir| dir.join(name))
        .find(|candidate| is_executable(candidate))
    else {
        return Probe::Missing {
            reason: format!("`{name}` is not on the search path"),
        };
    };
    let output = match scratch.command(&path).arg("--version").output() {
        Ok(output) => output,
        Err(err) => {
            return Probe::Missing {
                reason: format!("`{} --version` did not run: {err}", path.display()),
            };
        }
    };
    if !output.status.success() {
        return Probe::Missing {
            reason: format!(
                "`{} --version` exited with {}",
                path.display(),
                output.status
            ),
        };
    }
    // Some tools print their version on stderr; keep lossy text either way.
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    if text.trim().is_empty() {
        text = String::from_utf8_lossy(&output.stderr).into_owned();
    }
    let version = text.lines().next().unwrap_or("").trim().to_string();
    Probe::Found { path, version }
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

impl Probe {
    /// Returns the binary path and version when found. When missing, records
    /// the skip for `test`, prints it, and returns `None`; the test should
    /// then return.
    pub fn found_or_skip(self, test: &str) -> Option<(PathBuf, String)> {
        match self {
            Probe::Found { path, version } => Some((path, version)),
            Probe::Missing { reason } => {
                skip(test, &reason);
                None
            }
        }
    }
}

/// Records that `test` skipped for `reason`, then prints the skip and the list
/// of every skip recorded so far.
pub fn skip(test: &str, reason: &str) {
    let mut skips = SKIPS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    skips.push(Skip {
        test: test.to_string(),
        reason: reason.to_string(),
    });
    eprintln!("SKIPPED: {test}: {reason}");
    eprintln!("skip list ({} so far):", skips.len());
    for entry in skips.iter() {
        eprintln!("  {}: {}", entry.test, entry.reason);
    }
}

/// Every skip recorded in this process so far.
pub fn skipped() -> Vec<Skip> {
    SKIPS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_fake_harness(scratch: &ScratchHome, name: &str, body: &str) -> PathBuf {
        let bin = scratch.root().join("bin");
        fs::create_dir_all(&bin).expect("bin dir");
        let path = bin.join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("script written");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
        bin
    }

    #[test]
    fn a_found_binary_records_its_path_and_version() {
        let scratch = ScratchHome::new().expect("scratch home");
        let bin = write_fake_harness(&scratch, "fakeharness", "echo 'fakeharness 1.2.3'");
        let result = probe("fakeharness", Some(bin.as_os_str()), &scratch);
        assert_eq!(
            result,
            Probe::Found {
                path: bin.join("fakeharness"),
                version: "fakeharness 1.2.3".to_string(),
            }
        );
    }

    #[test]
    fn the_binary_runs_under_the_scratch_home() {
        let scratch = ScratchHome::new().expect("scratch home");
        let bin = write_fake_harness(&scratch, "homeprinter", "echo \"$HOME\"");
        let Probe::Found { version, .. } = probe("homeprinter", Some(bin.as_os_str()), &scratch)
        else {
            panic!("the fake harness should be found");
        };
        assert_eq!(version, scratch.home().to_string_lossy());
    }

    #[test]
    fn a_version_printed_on_stderr_is_kept() {
        let scratch = ScratchHome::new().expect("scratch home");
        let bin = write_fake_harness(&scratch, "errversion", "echo 'errversion 9' >&2");
        let result = probe("errversion", Some(bin.as_os_str()), &scratch);
        assert!(matches!(result, Probe::Found { ref version, .. } if version == "errversion 9"));
    }

    #[test]
    fn a_missing_binary_names_why() {
        let scratch = ScratchHome::new().expect("scratch home");
        let empty = scratch.root().join("empty");
        fs::create_dir_all(&empty).expect("empty dir");
        let result = probe("usher-no-such-harness", Some(empty.as_os_str()), &scratch);
        let Probe::Missing { reason } = result else {
            panic!("a binary that does not exist must be Missing");
        };
        assert!(reason.contains("usher-no-such-harness"));
    }

    #[test]
    fn a_failing_version_command_is_missing_with_its_status() {
        let scratch = ScratchHome::new().expect("scratch home");
        let bin = write_fake_harness(&scratch, "badversion", "exit 3");
        let Probe::Missing { reason } = probe("badversion", Some(bin.as_os_str()), &scratch) else {
            panic!("a failing --version must be Missing");
        };
        assert!(reason.contains("exited with"));
    }

    #[test]
    fn a_missing_binary_is_recorded_as_a_named_skip() {
        let scratch = ScratchHome::new().expect("scratch home");
        let empty = scratch.root().join("empty");
        fs::create_dir_all(&empty).expect("empty dir");
        let probe_result = probe("usher-skip-me", Some(empty.as_os_str()), &scratch);
        assert_eq!(probe_result.clone().found_or_skip("skip_demo_test"), None);
        let recorded = skipped();
        assert!(
            recorded
                .iter()
                .any(|s| s.test == "skip_demo_test" && s.reason.contains("usher-skip-me")),
            "the skip was not recorded: {recorded:?}"
        );
    }
}
