//! The runner checks the control first, never reads a failed control as a
//! refuted claim, and never reports a probe that did not run as ok.

mod common;

use std::cell::RefCell;

use common::fake_harness;
use usher_probes::{
    codex_home::probe_home,
    fragment::{Date, Network},
    run::{Spec, Unmeasured, run, run_or_panic},
};
use usher_testkit::ScratchHome;

fn today() -> Date {
    Date::new(2026, 11, 20).expect("a real date")
}

fn spec<'a>(binary: &'a str, search: &'a std::ffi::OsStr) -> Spec<'a> {
    Spec {
        name: "demo-probe",
        binary,
        search_path: Some(search),
        network: Network::Denied,
    }
}

#[test]
fn the_control_runs_before_the_claim_and_a_pass_is_reported() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let order = RefCell::new(Vec::new());
    let fragment = run(
        &spec("fakeh", bin.as_os_str()),
        &scratch,
        today(),
        |_| {
            order.borrow_mut().push("control");
            Ok(())
        },
        |_| {
            order.borrow_mut().push("claim");
            Ok(true)
        },
    )
    .expect("a held control and claim is a pass");
    assert_eq!(*order.borrow(), ["control", "claim"]);
    assert_eq!(
        fragment.render(),
        r#"{ version = "1.2.3", outcome = "passed", date = 2026-11-20, network = "denied" }"#
    );
}

#[test]
fn a_claim_that_does_not_hold_is_a_failed_result() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let fragment = run(
        &spec("fakeh", bin.as_os_str()),
        &scratch,
        today(),
        |_| Ok(()),
        |_| Ok(false),
    )
    .expect("a refuted claim with a held control still produces a result");
    assert!(
        fragment.render().contains(r#"outcome = "failed""#),
        "{}",
        fragment.render()
    );
}

#[test]
fn a_failed_control_is_not_run_and_the_claim_is_never_called() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let claim_called = RefCell::new(false);
    let result = run(
        &spec("fakeh", bin.as_os_str()),
        &scratch,
        today(),
        |_| Err("the control read no rules".to_string()),
        |_| {
            *claim_called.borrow_mut() = true;
            Ok(true)
        },
    );
    let not_run = result.expect_err("a failed control is not a result");
    assert!(not_run.reason().contains("control"), "{not_run}");
    assert!(
        not_run.reason().contains("the control read no rules"),
        "{not_run}"
    );
    assert!(not_run.to_string().starts_with("not run: "), "{not_run}");
    assert!(
        !*claim_called.borrow(),
        "the claim ran after a failed control"
    );
}

#[test]
fn a_claim_that_cannot_be_measured_is_not_run() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let not_run = run(
        &spec("fakeh", bin.as_os_str()),
        &scratch,
        today(),
        |_| Ok(()),
        |_| {
            Err(Unmeasured::Broken(
                "the count could not be read".to_string(),
            ))
        },
    )
    .expect_err("an unmeasurable claim is not a result");
    assert!(
        not_run.reason().contains("the count could not be read"),
        "{not_run}"
    );
}

#[test]
fn a_missing_binary_is_not_run_and_neither_closure_runs() {
    let scratch = ScratchHome::new().expect("scratch home");
    let empty = scratch.root().join("empty");
    std::fs::create_dir_all(&empty).expect("empty dir");
    let called = RefCell::new(false);
    let not_run = run(
        &spec("usher-no-such-harness", empty.as_os_str()),
        &scratch,
        today(),
        |_| {
            *called.borrow_mut() = true;
            Ok(())
        },
        |_| {
            *called.borrow_mut() = true;
            Ok(true)
        },
    )
    .expect_err("no binary, no result");
    assert!(
        not_run.reason().contains("usher-no-such-harness"),
        "{not_run}"
    );
    assert!(!*called.borrow(), "a closure ran without a harness");
}

#[test]
fn a_version_line_with_no_version_in_it_is_not_run() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh has no number");
    let not_run = run(
        &spec("fakeh", bin.as_os_str()),
        &scratch,
        today(),
        |_| Ok(()),
        |_| Ok(true),
    )
    .expect_err("a version that cannot be read is not a result");
    assert!(not_run.reason().contains("version"), "{not_run}");
}

#[test]
fn the_closures_get_the_bare_version_the_path_and_the_scratch_home() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh-cli 4.5.6");
    let seen = RefCell::new(None);
    run(
        &spec("fakeh", bin.as_os_str()),
        &scratch,
        today(),
        |harness| {
            *seen.borrow_mut() = Some((
                harness.version.clone(),
                harness.path.clone(),
                harness.scratch.root().to_path_buf(),
            ));
            Ok(())
        },
        |_| Ok(true),
    )
    .expect("a pass");
    let (version, path, root) = seen.borrow().clone().expect("the control saw the harness");
    assert_eq!(version, "4.5.6");
    assert_eq!(path, bin.join("fakeh"));
    assert_eq!(root, scratch.root());
}

#[test]
#[should_panic(expected = "usher-no-such-harness")]
fn run_or_panic_panics_with_the_not_run_reason() {
    let scratch = ScratchHome::new().expect("scratch home");
    let empty = scratch.root().join("empty");
    std::fs::create_dir_all(&empty).expect("empty dir");
    run_or_panic(
        &spec("usher-no-such-harness", empty.as_os_str()),
        &scratch,
        today(),
        |_| Ok(()),
        |_| Ok(true),
    );
}

#[test]
fn run_or_panic_returns_the_fragment_when_the_probe_ran() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let fragment = run_or_panic(
        &spec("fakeh", bin.as_os_str()),
        &scratch,
        today(),
        |_| Ok(()),
        |_| Ok(true),
    );
    assert!(fragment.render().contains(r#"version = "1.2.3""#));
}

#[test]
fn a_codex_probe_home_holds_only_the_files_the_probe_wrote() {
    let scratch = ScratchHome::new().expect("scratch home");
    // Anything already in the scratch codex home must not leak into the probe.
    std::fs::create_dir_all(scratch.codex_home().join("sessions")).expect("stray dirs");
    std::fs::write(scratch.codex_home().join("auth.json"), "{}").expect("stray auth");
    std::fs::write(scratch.codex_home().join("history.jsonl"), "{}").expect("stray history");

    let home =
        probe_home(&scratch, &[("config.toml", "model = \"demo\"\n")]).expect("a probe home");
    let mut names: Vec<String> = std::fs::read_dir(&home)
        .expect("probe home dir")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    assert_eq!(names, ["config.toml"]);
    assert_eq!(
        std::fs::read_to_string(home.join("config.toml")).expect("written config"),
        "model = \"demo\"\n"
    );
    assert!(
        home.starts_with(scratch.root()),
        "the probe home is outside the scratch root"
    );
}

#[test]
fn a_probe_home_refuses_credentials_and_names_that_leave_the_directory() {
    let scratch = ScratchHome::new().expect("scratch home");
    assert!(
        probe_home(&scratch, &[("config.toml", "x = 1\n")]).is_ok(),
        "control: a plain config file is accepted"
    );
    for name in ["auth.json", "../escape.toml", "sub/config.toml", ""] {
        assert!(
            probe_home(&scratch, &[(name, "x = 1\n")]).is_err(),
            "`{name}` must be refused"
        );
    }
}
