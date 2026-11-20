//! A started process that outlives the probe's timeout is killed, and the
//! probe ends not run; the default timeout is 120 seconds.

mod common;

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use common::{fake_harness, hold};
use usher_probes::{
    fragment::Date,
    run::{Harness, Launcher, Spec, Unmeasured, run},
};
use usher_testkit::ScratchHome;

fn today() -> Date {
    Date::new(2026, 11, 20).expect("a real date")
}

#[test]
fn a_spec_times_out_after_one_hundred_twenty_seconds_by_default() {
    assert_eq!(Spec::new("p", "b").timeout, Duration::from_secs(120));
    let custom = Spec::new("p", "b").timeout(Duration::from_secs(7));
    assert_eq!(custom.timeout, Duration::from_secs(7));
}

#[test]
fn a_start_that_outlives_the_timeout_is_killed_and_the_probe_is_not_run() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let spec = Spec::new("demo-probe", "fakeh")
        .search_path(bin.as_os_str())
        .timeout(Duration::from_secs(1));
    // Control: a start that returns at once is a result.
    assert!(run(&spec, &scratch, today(), |_, _| Ok(()), hold).is_ok());

    let started = Instant::now();
    let not_run = run(
        &spec,
        &scratch,
        today(),
        |_, _| Ok(()),
        |_, launcher| {
            // The outcome of the run is ignored: the timeout wins.
            let _ = launcher.run("/bin/sleep", &["6"]);
            Ok(true)
        },
    )
    .expect_err("a timed-out start is not a result");
    assert_eq!(not_run.to_string(), "not run: timed out after 1 s");
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "the process was not killed at the timeout: {:?}",
        started.elapsed()
    );
}

// The timeout wins however the claim handles the failed start: it may pass the
// io error on as Broken, name another reason, or carry on and return a result.
#[test]
fn the_timeout_wins_whatever_the_claim_returns_after_a_start_timed_out() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let spec = Spec::new("demo-probe", "fakeh")
        .search_path(bin.as_os_str())
        .timeout(Duration::from_secs(1));
    // Control: the same claim shapes on a start that returns at once are results.
    for held in [true, false] {
        assert!(
            run(
                &spec,
                &scratch,
                today(),
                |_, _| Ok(()),
                |_, launcher| {
                    launcher
                        .run("/bin/sleep", &["0"])
                        .map_err(|err| Unmeasured::Broken(err.to_string()))?;
                    Ok(held)
                },
            )
            .is_ok()
        );
    }

    type Claim = fn(&Harness, &Launcher<'_>) -> Result<bool, Unmeasured>;
    let claims: [(&str, Claim); 4] = [
        ("passes the error on as Broken", |_, launcher| {
            launcher
                .run("/bin/sleep", &["6"])
                .map_err(|err| Unmeasured::Broken(err.to_string()))?;
            Ok(true)
        }),
        ("returns Passed after the error", |_, launcher| {
            let _ = launcher.run("/bin/sleep", &["6"]).is_err();
            Ok(true)
        }),
        ("returns Failed after the error", |_, launcher| {
            let _ = launcher.run("/bin/sleep", &["6"]).is_err();
            Ok(false)
        }),
        ("names another reason", |_, launcher| {
            let _ = launcher.run("/bin/sleep", &["6"]).is_err();
            Err(Unmeasured::NeedsNetwork)
        }),
    ];
    for (what, claim) in claims {
        let started = Instant::now();
        let not_run = run(&spec, &scratch, today(), |_, _| Ok(()), claim)
            .expect_err("a timed-out start is not a result");
        assert_eq!(
            not_run.to_string(),
            "not run: timed out after 1 s",
            "a claim that {what}"
        );
        assert!(
            started.elapsed() < Duration::from_secs(4),
            "a claim that {what}: the process was not killed at the timeout"
        );
    }
}

// Any start the launcher kills reads "timed out", the control's included. A
// control that simply gives the wrong answer keeps "the control failed".
#[test]
fn a_control_that_outlives_the_timeout_ends_timed_out_and_the_claim_is_never_called() {
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    let spec = Spec::new("demo-probe", "fakeh")
        .search_path(bin.as_os_str())
        .timeout(Duration::from_secs(1));

    let claimed = std::cell::Cell::new(false);
    let started = Instant::now();
    let not_run = run(
        &spec,
        &scratch,
        today(),
        |_, launcher| {
            launcher
                .run("/bin/sleep", &["6"])
                .map(|_| ())
                .map_err(|err| err.to_string())
        },
        |harness, launcher| {
            claimed.set(true);
            hold(harness, launcher)
        },
    )
    .expect_err("a control that timed out is not a result");
    assert_eq!(not_run.to_string(), "not run: timed out after 1 s");
    assert!(!claimed.get(), "the claim ran after the control timed out");
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "the control's process was not killed at the timeout: {:?}",
        started.elapsed()
    );

    // Control: a control that answers wrongly, fast, keeps its own words.
    let wrong = run(
        &spec,
        &scratch,
        today(),
        |_, _| Err("it said no".to_string()),
        hold,
    )
    .expect_err("a failed control is not a result");
    assert_eq!(wrong.to_string(), "not run: the control failed: it said no");
}

/// Whether a process with this id still exists, asked of the system.
fn alive(pid: &str) -> bool {
    Command::new("/bin/kill")
        .args(["-0", pid])
        .stderr(Stdio::null())
        .status()
        .expect("kill runs")
        .success()
}

/// Waits up to `limit` for the process to be gone.
fn gone_within(pid: &str, limit: Duration) -> bool {
    let deadline = Instant::now() + limit;
    while Instant::now() < deadline {
        if !alive(pid) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    !alive(pid)
}

#[test]
fn a_start_that_outlives_the_timeout_is_gone_afterwards() {
    // Control: the check sees a live process as alive and a killed one as gone.
    let mut control = Command::new("/bin/sleep")
        .arg("20")
        .spawn()
        .expect("a control process");
    let control_pid = control.id().to_string();
    assert!(
        alive(&control_pid),
        "control: a live process reads as alive"
    );
    control.kill().expect("the control is killed");
    control.wait().expect("the control is reaped");
    assert!(
        gone_within(&control_pid, Duration::from_secs(2)),
        "control: a killed process reads as gone"
    );

    // A harness that records its own pid and then sleeps well past the timeout.
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = scratch.root().join("bin");
    fs::create_dir_all(&bin).expect("bin dir");
    let path = bin.join("fakeh");
    fs::write(
        &path,
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'fakeh 1.2.3'; exit 0; fi\necho $$ > \"$TMPDIR/hang.pid\"\nexec /bin/sleep 20\n",
    )
    .expect("script written");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
    let spec = Spec::new("demo-probe", "fakeh")
        .search_path(bin.as_os_str())
        .timeout(Duration::from_secs(1));
    let not_run = run(
        &spec,
        &scratch,
        today(),
        |_, _| Ok(()),
        |harness, launcher| {
            let _ = launcher.run(&harness.path, &["hang"]);
            Ok(true)
        },
    )
    .expect_err("a timed-out start is not a result");
    assert_eq!(not_run.to_string(), "not run: timed out after 1 s");

    let pid =
        fs::read_to_string(scratch.tmp().join("hang.pid")).expect("the harness wrote its pid");
    let pid = pid.trim();
    assert!(!pid.is_empty());
    assert!(
        gone_within(pid, Duration::from_secs(2)),
        "the timed-out process {pid} is still running"
    );
}
