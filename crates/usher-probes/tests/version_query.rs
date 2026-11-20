//! The version query is a start of the harness binary, so it goes through the
//! launcher's one spawn site: the profile, the cleared environment, the harness
//! home and the timeout apply to it, and it does not count as a claim start.

mod common;

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    time::{Duration, Instant},
};

use common::hold;
use usher_probes::{
    fragment::Date,
    run::{HarnessKind, Spec, run},
};
use usher_testkit::ScratchHome;

fn today() -> Date {
    Date::new(2026, 11, 20).expect("a real date")
}

/// Writes an executable `fakeh` with `body` after a `--version` line and
/// returns the directory that holds it.
fn script_harness(scratch: &ScratchHome, body: &str) -> PathBuf {
    let bin = scratch.root().join("bin");
    fs::create_dir_all(&bin).expect("bin dir");
    let path = bin.join("fakeh");
    fs::write(&path, format!("#!/bin/sh\necho 'fakeh 1.2.3'\n{body}\n")).expect("script written");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
    bin
}

/// A claim that starts something harmless, so the harness script runs only for
/// the version query.
fn claim_true(
    _: &usher_probes::run::Harness,
    launcher: &usher_probes::run::Launcher<'_>,
) -> Result<bool, usher_probes::run::Unmeasured> {
    launcher
        .run("/usr/bin/true", &[])
        .map_err(|err| usher_probes::run::Unmeasured::Broken(err.to_string()))?;
    Ok(true)
}

fn seen_names(scratch: &ScratchHome) -> Vec<String> {
    let text = fs::read_to_string(scratch.tmp().join("seen-env.txt"))
        .expect("the harness script wrote the environment it saw");
    let mut names: Vec<String> = text
        .lines()
        .filter_map(|line| line.split_once('=').map(|(name, _)| name.to_string()))
        // The shell that runs the script adds these itself.
        .filter(|name| !["PWD", "SHLVL", "_", "OLDPWD"].contains(&name.as_str()))
        .collect();
    names.sort();
    names
}

fn seen_value(scratch: &ScratchHome, name: &str) -> Option<String> {
    fs::read_to_string(scratch.tmp().join("seen-env.txt"))
        .expect("the saved environment")
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{name}=")).map(str::to_string))
}

#[test]
fn the_version_query_child_gets_exactly_the_cleared_environment() {
    // Control: this process holds variables that must not reach the harness.
    assert!(std::env::var_os("CARGO_PKG_NAME").is_some());

    let scratch = ScratchHome::new().expect("scratch home");
    let bin = script_harness(&scratch, "/usr/bin/env > \"$TMPDIR/seen-env.txt\"");
    let spec = Spec::new("demo-probe", "fakeh").search_path(bin.as_os_str());
    run(&spec, &scratch, today(), |_, _| Ok(()), claim_true).expect("a pass");
    assert_eq!(seen_names(&scratch), ["HOME", "LANG", "PATH", "TMPDIR"]);
    assert_eq!(
        seen_value(&scratch, "HOME").as_deref(),
        scratch.home().to_str()
    );
    assert_eq!(
        seen_value(&scratch, "TMPDIR").as_deref(),
        scratch.tmp().to_str()
    );

    // And a codex query adds its harness home, nothing else.
    let codex = ScratchHome::new().expect("scratch home");
    let bin = script_harness(&codex, "/usr/bin/env > \"$TMPDIR/seen-env.txt\"");
    let spec = Spec::new("demo-probe", "fakeh")
        .search_path(bin.as_os_str())
        .harness(HarnessKind::Codex);
    run(&spec, &codex, today(), |_, _| Ok(()), claim_true).expect("a pass");
    assert_eq!(
        seen_names(&codex),
        ["CODEX_HOME", "HOME", "LANG", "PATH", "TMPDIR"]
    );
    assert_eq!(
        seen_value(&codex, "CODEX_HOME").as_deref(),
        codex.codex_home().to_str()
    );
}

#[cfg(target_os = "macos")]
#[test]
fn the_version_query_is_refused_the_network_by_the_profile() {
    let scratch = ScratchHome::new().expect("scratch home");
    let connect = "/usr/bin/nc -v -z -w 1 -G 1 192.0.2.1 80 2> \"$TMPDIR/nc.txt\"; true";
    let bin = script_harness(&scratch, connect);

    // Outside half: the same script without the profile is not refused by a sandbox.
    let outside = scratch
        .command(bin.join("fakeh"))
        .output()
        .expect("the script runs without the profile");
    assert!(outside.status.success());
    let outside_text = fs::read_to_string(scratch.tmp().join("nc.txt")).expect("nc output");
    assert!(outside_text.contains("failed:"), "{outside_text}");
    assert!(
        !outside_text.contains("Operation not permitted"),
        "control: without the profile nothing refuses the connect: {outside_text}"
    );
    fs::remove_file(scratch.tmp().join("nc.txt")).expect("cleared");

    // Inside: the version query is the only start of the script, and the
    // profile's own refusal is what it saw.
    let spec = Spec::new("demo-probe", "fakeh").search_path(bin.as_os_str());
    run(&spec, &scratch, today(), |_, _| Ok(()), claim_true).expect("a pass");
    let inside = fs::read_to_string(scratch.tmp().join("nc.txt")).expect("nc output");
    assert!(inside.contains("Operation not permitted"), "{inside}");
}

#[test]
fn a_version_query_that_outlives_the_timeout_ends_timed_out() {
    // Control: a script that answers at once is a result.
    let quick = ScratchHome::new().expect("scratch home");
    let bin = script_harness(&quick, "true");
    let spec = Spec::new("demo-probe", "fakeh")
        .search_path(bin.as_os_str())
        .timeout(Duration::from_secs(1));
    assert!(run(&spec, &quick, today(), |_, _| Ok(()), claim_true).is_ok());

    let scratch = ScratchHome::new().expect("scratch home");
    let bin = script_harness(&scratch, "exec /bin/sleep 6");
    let spec = Spec::new("demo-probe", "fakeh")
        .search_path(bin.as_os_str())
        .timeout(Duration::from_secs(1));
    let claimed = std::cell::Cell::new(false);
    let started = Instant::now();
    let not_run = run(
        &spec,
        &scratch,
        today(),
        |_, _| Ok(()),
        |harness, launcher| {
            claimed.set(true);
            hold(harness, launcher)
        },
    )
    .expect_err("a version query that timed out is not a result");
    assert_eq!(not_run.to_string(), "not run: timed out after 1 s");
    assert!(
        !claimed.get(),
        "the claim ran after the version query timed out"
    );
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "the version query was not killed at the timeout: {:?}",
        started.elapsed()
    );
}

#[test]
fn a_claim_that_starts_nothing_still_reads_started_no_harness_after_a_version_query() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = script_harness(&scratch, "true");
    let spec = Spec::new("demo-probe", "fakeh").search_path(bin.as_os_str());
    // Control: the same spec with a claim that starts something is a result.
    assert!(run(&spec, &scratch, today(), |_, _| Ok(()), claim_true).is_ok());

    let not_run = run(&spec, &scratch, today(), |_, _| Ok(()), |_, _| Ok(true))
        .expect_err("the version query is not the claim's start");
    assert_eq!(not_run.to_string(), "not run: the claim started no harness");
}
