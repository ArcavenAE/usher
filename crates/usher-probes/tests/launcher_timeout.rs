//! A started process that outlives the probe's timeout is killed, and the
//! probe ends not run; the default timeout is 120 seconds.

mod common;

use std::time::{Duration, Instant};

use common::{fake_harness, hold};
use usher_probes::{
    fragment::Date,
    run::{Spec, run},
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
