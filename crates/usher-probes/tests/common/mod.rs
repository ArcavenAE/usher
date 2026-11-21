//! Shared helpers: a fake harness binary that prints a version line.

#![allow(dead_code)] // each test file uses a different part

use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

use usher_probes::run::{Harness, Launcher, Unmeasured};
use usher_testkit::ScratchHome;

/// Writes an executable `name` that prints `version_line` for any arguments
/// and returns the directory that holds it.
pub fn fake_harness(scratch: &ScratchHome, name: &str, version_line: &str) -> PathBuf {
    let bin = scratch.root().join("bin");
    fs::create_dir_all(&bin).expect("bin dir");
    let path = bin.join(name);
    fs::write(&path, format!("#!/bin/sh\necho '{version_line}'\n")).expect("script written");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
    bin
}

/// A claim that starts the harness through the launcher and holds.
pub fn hold(harness: &Harness, launcher: &Launcher<'_>) -> Result<bool, Unmeasured> {
    launcher
        .run(&harness.path, &["--version"])
        .map_err(|err| Unmeasured::Broken(err.to_string()))?;
    Ok(true)
}

/// A claim that starts the harness through the launcher and does not hold.
pub fn refute(harness: &Harness, launcher: &Launcher<'_>) -> Result<bool, Unmeasured> {
    hold(harness, launcher).map(|_| false)
}
