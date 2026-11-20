//! The launcher is the only way a claim starts a process, and the fragment's
//! network value comes from what the runner did, not from the probe.

mod common;

use std::{cell::Cell, fs, path::Path};

use common::{fake_harness, hold};
use usher_probes::{
    fragment::{Date, Network},
    run::{Spec, Unmeasured, run, run_with_host_state},
    structural::find_launch_bypasses,
};
use usher_testkit::ScratchHome;

fn today() -> Date {
    Date::new(2026, 11, 20).expect("a real date")
}

#[test]
fn a_claim_that_started_nothing_is_not_run() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let spec = Spec::new("demo-probe", "fakeh").search_path(bin.as_os_str());
    // Control: a claim that starts the harness is a result.
    assert!(run(&spec, &scratch, today(), |_, _| Ok(()), hold).is_ok());

    // The control may start a process; the claim must start one itself.
    for held in [true, false] {
        let not_run = run(
            &spec,
            &scratch,
            today(),
            |harness, launcher| {
                launcher
                    .run(&harness.path, &["--version"])
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            },
            |_, _| Ok(held),
        )
        .expect_err("a claim that started no harness is not a result");
        assert_eq!(not_run.to_string(), "not run: the claim started no harness");
    }
}

#[test]
fn the_fragment_says_not_denied_on_a_host_that_cannot_deny_and_denied_on_one_that_can() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let spec = Spec::new("demo-probe", "fakeh").search_path(bin.as_os_str());

    let cannot = run_with_host_state(
        Network::NotDenied,
        &spec,
        &scratch,
        today(),
        |_, _| Ok(()),
        hold,
    )
    .expect("a pass on a host that cannot deny");
    assert!(
        cannot.render().contains(r#"network = "not denied""#),
        "{}",
        cannot.render()
    );

    let can = run_with_host_state(
        Network::Denied,
        &spec,
        &scratch,
        today(),
        |_, _| Ok(()),
        hold,
    )
    .expect("a pass on a host that can deny");
    assert!(
        can.render().contains(r#"network = "denied""#),
        "{}",
        can.render()
    );

    // The launcher has no second path: even where nothing is denied, a claim
    // that never started the harness has no result to record.
    let not_run = run_with_host_state(
        Network::NotDenied,
        &spec,
        &scratch,
        today(),
        |_, _| Ok(()),
        |_, _| Ok(true),
    )
    .expect_err("no start, no result");
    assert_eq!(not_run.reason(), "the claim started no harness");
}

#[cfg(target_os = "macos")]
#[test]
fn a_claim_run_through_the_launcher_records_denied_and_is_refused_by_the_profile() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let spec = Spec::new("demo-probe", "fakeh").search_path(bin.as_os_str());
    let refused = Cell::new(false);
    let fragment = run(
        &spec,
        &scratch,
        today(),
        |_, _| Ok(()),
        |_, launcher| {
            let output = launcher
                .run(
                    "/usr/bin/nc",
                    &["-v", "-z", "-w", "1", "-G", "1", "192.0.2.1", "80"],
                )
                .map_err(|err| Unmeasured::Broken(err.to_string()))?;
            let stderr = String::from_utf8_lossy(&output.stderr);
            refused.set(stderr.contains("Operation not permitted"));
            Ok(refused.get())
        },
    )
    .expect("the claim ran");
    assert!(refused.get(), "the connect was not refused by the profile");
    assert!(
        fragment.render().contains(r#"outcome = "passed""#)
            && fragment.render().contains(r#"network = "denied""#),
        "{}",
        fragment.render()
    );
}

/// The probe source files: everything under `src` except the two that own
/// process starts.
fn probe_sources() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir).expect("the src directory") {
        let path = entry.expect("an entry").path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a utf-8 file name")
            .to_string();
        if name.ends_with(".rs") && name != "run.rs" && name != "network.rs" {
            files.push((name, fs::read_to_string(&path).expect("a readable source")));
        }
    }
    files
}

#[test]
fn no_probe_source_starts_a_process_on_its_own() {
    // The scan must see a bypass before its silence on the real files means anything.
    for fixture in [
        "fn bypass() { std::process::Command::new(\"x\"); }",
        "use std::process::Command;",
        "fn bypass(s: &S) { s.scratch.command(\"x\"); }",
        "let c = Command::new(\"sh\");",
    ] {
        assert!(
            !find_launch_bypasses(fixture).is_empty(),
            "the scan missed: {fixture}"
        );
    }
    assert!(find_launch_bypasses("fn clean() { launcher.run(\"x\", &[]); }").is_empty());

    let files = probe_sources();
    assert!(!files.is_empty(), "no probe source was scanned");
    for (name, text) in files {
        let hits = find_launch_bypasses(&text);
        assert!(
            hits.is_empty(),
            "{name} starts a process on its own: {hits:?}"
        );
    }
}
