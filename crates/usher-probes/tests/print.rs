//! A probe that did not run prints no fragment line, and one that ran prints
//! its line. The panic is caught in-process by the other tests, so stdout is
//! read here from a child process run with `--nocapture`.

mod common;

use std::process::{Command, Output};

use common::fake_harness;
use usher_probes::{
    fragment::{Date, Network},
    run::{Spec, Unmeasured, run_or_panic},
};
use usher_testkit::ScratchHome;

const MODE: &str = "USHER_PROBE_PRINT_CHILD";

/// Run by the tests below as a child process, which sets the mode.
#[test]
#[ignore = "run only by the tests below, as a child process"]
fn child_runs_one_probe() {
    let Some(mode) = std::env::var_os(MODE) else {
        return;
    };
    let scratch = ScratchHome::new().expect("scratch home");
    // A missing binary is one way not to run; a claim that needs credentials or a
    // network is the others.
    let bin = if mode == "missing-binary" {
        let empty = scratch.root().join("empty");
        std::fs::create_dir_all(&empty).expect("empty dir");
        empty
    } else {
        fake_harness(&scratch, "fakeh", "fakeh 1.2.3")
    };
    let spec = Spec {
        name: "demo-probe",
        binary: "fakeh",
        search_path: Some(bin.as_os_str()),
        network: Network::Denied,
    };
    run_or_panic(
        &spec,
        &scratch,
        Date::new(2026, 11, 20).expect("a real date"),
        |_| Ok(()),
        |_| match mode.to_str() {
            Some("needs-credentials") => Err(Unmeasured::NeedsCredentials),
            Some("needs-network") => Err(Unmeasured::NeedsNetwork),
            _ => Ok(true),
        },
    );
}

fn run_child(mode: &str) -> Output {
    let exe = std::env::current_exe().expect("this test binary");
    Command::new(exe)
        .args([
            "--ignored",
            "--exact",
            "child_runs_one_probe",
            "--nocapture",
        ])
        .env(MODE, mode)
        .output()
        .expect("the child runs")
}

fn probe_lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.starts_with("probe "))
        .map(str::to_string)
        .collect()
}

#[test]
fn a_probe_that_ran_prints_its_fragment_line() {
    let output = run_child("ran");
    assert!(
        output.status.success(),
        "the control child must succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        probe_lines(&output),
        [
            r#"probe demo-probe: { version = "1.2.3", outcome = "passed", date = 2026-11-20, network = "denied" }"#
        ]
    );
}

#[test]
fn a_probe_that_did_not_run_prints_no_fragment_line() {
    let output = run_child("missing-binary");
    assert!(
        !output.status.success(),
        "a probe that cannot run must fail the run"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("not run: "),
        "the panic names why: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        probe_lines(&output),
        Vec::<String>::new(),
        "a fragment line was printed for a probe that did not run"
    );
}

#[test]
fn a_claim_that_needs_credentials_or_a_network_prints_no_fragment_line() {
    for mode in ["needs-credentials", "needs-network"] {
        let output = run_child(mode);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{mode}: the run must fail");
        assert!(stderr.contains("not run: needs "), "{mode}: {stderr}");
        assert_eq!(
            probe_lines(&output),
            Vec::<String>::new(),
            "{mode}: a fragment line was printed for a probe that did not run"
        );
    }
}
