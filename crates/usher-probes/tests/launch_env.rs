//! A launched process starts with a cleared environment plus HOME (the scratch
//! home), PATH, TMPDIR and LANG, so proxy variables and secrets in the parent
//! never reach a harness. The parent's environment is set from outside, so a
//! child test process runs the probe and prints what the launched process saw.

mod common;

use std::process::{Command, Output};

use common::fake_harness;
use usher_probes::{
    fragment::Date,
    run::{Spec, Unmeasured, run_or_panic},
};
use usher_testkit::ScratchHome;

const MODE: &str = "USHER_PROBE_ENV_CHILD";

/// Run by the test below as a child process, with variables set around it.
#[test]
#[ignore = "run only by the test below, as a child process"]
fn child_prints_the_launched_environment() {
    if std::env::var_os(MODE).is_none() {
        return;
    }
    let scratch = ScratchHome::new().expect("scratch home");
    let bin = fake_harness(&scratch, "fakeh", "fakeh 1.2.3");
    println!(
        "PARENT_SAW_PROXY={}",
        std::env::var_os("HTTPS_PROXY").is_some()
    );
    println!("SCRATCH_HOME={}", scratch.home().display());
    let spec = Spec::new("demo-probe", "fakeh").search_path(bin.as_os_str());
    run_or_panic(
        &spec,
        &scratch,
        Date::new(2026, 11, 20).expect("a real date"),
        |_, _| Ok(()),
        |_, launcher| {
            let output = launcher
                .run("/usr/bin/env", &[])
                .map_err(|err| Unmeasured::Broken(err.to_string()))?;
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                println!("LAUNCHED:{line}");
            }
            Ok(true)
        },
    );
}

fn run_child() -> Output {
    let exe = std::env::current_exe().expect("this test binary");
    Command::new(exe)
        .args([
            "--ignored",
            "--exact",
            "child_prints_the_launched_environment",
            "--nocapture",
        ])
        .env(MODE, "1")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("ALL_PROXY", "socks5://127.0.0.1:9")
        .env("USHER_PROBE_SECRET", "not-a-real-secret")
        .env("LANG", "en_US.UTF-8")
        .output()
        .expect("the child runs")
}

#[test]
fn a_launched_process_gets_only_home_path_tmpdir_and_lang() {
    let output = run_child();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "the control child must succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Control: the child process itself did see the proxy variable.
    assert!(stdout.contains("PARENT_SAW_PROXY=true"), "{stdout}");

    let launched: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("LAUNCHED:"))
        .collect();
    assert!(
        !launched.is_empty(),
        "the launched process printed nothing:\n{stdout}"
    );
    for line in &launched {
        let upper = line.to_ascii_uppercase();
        assert!(
            !upper.contains("PROXY"),
            "a proxy variable reached the process: {line}"
        );
        assert!(
            !line.contains("USHER_PROBE_SECRET"),
            "a parent variable leaked: {line}"
        );
    }
    let scratch_home = stdout
        .lines()
        .find_map(|line| line.strip_prefix("SCRATCH_HOME="))
        .expect("the child printed its scratch home");
    for want in [
        format!("HOME={scratch_home}"),
        "LANG=en_US.UTF-8".to_string(),
    ] {
        assert!(
            launched.contains(&want.as_str()),
            "missing {want}:\n{launched:?}"
        );
    }
    for name in ["PATH=", "TMPDIR="] {
        assert!(
            launched.iter().any(|line| line.starts_with(name)),
            "missing {name}:\n{launched:?}"
        );
    }
}
