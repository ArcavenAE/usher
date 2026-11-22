//! The launcher is the only way a claim starts a process, and the fragment's
//! network value comes from what the runner did, not from the probe.

mod common;

use std::cell::Cell;

use common::{fake_harness, hold};
use usher_probes::{
    fragment::{Date, Network},
    run::{Spec, Unmeasured, run, run_with_host_state},
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

/// A declared port is the one hole the launcher's profile leaves: a claim on a
/// spec that declares it reaches that port through the launcher, and a second
/// listener that the spec did not declare is refused by the profile.
#[cfg(target_os = "macos")]
#[test]
fn a_port_declared_on_the_spec_is_reached_through_the_launcher_and_another_is_not() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let declared = std::net::TcpListener::bind("127.0.0.1:0").expect("a local listener");
    let other = std::net::TcpListener::bind("127.0.0.1:0").expect("a second listener");
    let declared_port = declared.local_addr().expect("address").port();
    let other_port = other.local_addr().expect("address").port();
    let ports = [declared_port];
    let spec = Spec::new("demo-probe", "fakeh")
        .search_path(bin.as_os_str())
        .localhost_ports(&ports);

    // Control: outside any profile both listeners accept, so a refusal below is the profile's.
    for port in [declared_port, other_port] {
        let outside = scratch
            .command("/usr/bin/nc")
            .args([
                "-v",
                "-z",
                "-w",
                "1",
                "127.0.0.1",
                port.to_string().as_str(),
            ])
            .output()
            .expect("nc runs without the profile");
        assert!(outside.status.success(), "control: port {port} must accept");
    }

    let reached = Cell::new(false);
    let refused = Cell::new(false);
    run(
        &spec,
        &scratch,
        today(),
        |_, _| Ok(()),
        |_, launcher| {
            let connect = |port: u16| {
                launcher
                    .run(
                        "/usr/bin/nc",
                        &[
                            "-v",
                            "-z",
                            "-w",
                            "1",
                            "127.0.0.1",
                            port.to_string().as_str(),
                        ],
                    )
                    .map_err(|err| Unmeasured::Broken(err.to_string()))
            };
            let to_declared = connect(declared_port)?;
            reached.set(to_declared.status.success());
            let to_other = connect(other_port)?;
            refused.set(
                !to_other.status.success()
                    && String::from_utf8_lossy(&to_other.stderr)
                        .contains("Operation not permitted"),
            );
            Ok(true)
        },
    )
    .expect("the claim ran");
    assert!(
        reached.get(),
        "the declared port was not reached through the launcher"
    );
    assert!(
        refused.get(),
        "a port the spec did not declare was reachable through the launcher"
    );
}
