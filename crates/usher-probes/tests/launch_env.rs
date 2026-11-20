//! A launched process starts with a cleared environment plus HOME (the scratch
//! home), PATH, TMPDIR and LANG, so proxy variables and secrets in the parent
//! never reach a harness. The parent's environment is set from outside, so a
//! child test process runs the probe and prints what the launched process saw.

mod common;

use std::process::{Command, Output};

use common::fake_harness;
use usher_probes::{
    fragment::Date,
    run::{HarnessKind, Spec, Unmeasured, run_or_panic},
};
use usher_testkit::ScratchHome;

const MODE: &str = "USHER_PROBE_ENV_CHILD";
const HARNESS: &str = "USHER_PROBE_ENV_HARNESS";

/// Run by the tests below as a child process, with variables set around it;
/// it returns at once when the mode is not set, so a plain run of this file
/// passes it.
#[test]
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
    println!("SCRATCH_TMP={}", scratch.tmp().display());
    println!("SCRATCH_ROOT={}", scratch.root().display());
    let kind = match std::env::var(HARNESS).as_deref() {
        Ok("codex") => HarnessKind::Codex,
        Ok("goose") => HarnessKind::Goose,
        _ => HarnessKind::ClaudeCode,
    };
    let spec = Spec::new("demo-probe", "fakeh")
        .search_path(bin.as_os_str())
        .harness(kind);
    run_or_panic(
        &spec,
        &scratch,
        Date::new(2026, 11, 20).expect("a real date"),
        |_, _| Ok(()),
        |_, launcher| {
            let output = launcher
                .run("/usr/bin/env", &[])
                .map_err(|err| Unmeasured::Broken(err.to_string()))?;
            println!("HARNESS_HOME={}", launcher.harness_home().display());
            println!("HARNESS_HOME_IS_DIR={}", launcher.harness_home().is_dir());
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                println!("LAUNCHED:{line}");
            }
            Ok(true)
        },
    );
}

fn run_child() -> Output {
    run_child_for(None)
}

fn run_child_for(harness: Option<&str>) -> Output {
    let exe = std::env::current_exe().expect("this test binary");
    let mut command = Command::new(exe);
    command
        .args([
            "--exact",
            "child_prints_the_launched_environment",
            "--nocapture",
        ])
        .env(MODE, "1")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("ALL_PROXY", "socks5://127.0.0.1:9")
        .env("USHER_PROBE_SECRET", "not-a-real-secret")
        .env("LANG", "en_US.UTF-8")
        // A parent value that must not reach a harness that does not read it.
        .env("CODEX_HOME", "/nonexistent-parent-codex-home");
    if let Some(harness) = harness {
        command.env(HARNESS, harness);
    }
    command.output().expect("the child runs")
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

#[test]
fn the_launched_environment_is_exactly_home_lang_path_and_tmpdir_with_the_scratch_tmp() {
    let output = run_child();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "the control child must succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Control: the parent held variables beyond the four, so "only" is a real claim.
    assert!(stdout.contains("PARENT_SAW_PROXY=true"), "{stdout}");

    let launched: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("LAUNCHED:"))
        .collect();
    assert!(
        !launched.is_empty(),
        "the launched process printed nothing:\n{stdout}"
    );
    let names: Vec<&str> = launched
        .iter()
        .map(|line| line.split('=').next().unwrap_or(""))
        .collect();
    // Only names are printed, never values: the parent holds secrets.
    let extra: Vec<&str> = names
        .iter()
        .copied()
        .filter(|name| !["HOME", "LANG", "PATH", "TMPDIR"].contains(name))
        .collect();
    assert!(
        extra.is_empty(),
        "the launched environment holds variables beyond HOME, LANG, PATH and TMPDIR: {extra:?}"
    );
    for want in ["HOME", "LANG", "PATH", "TMPDIR"] {
        assert!(names.contains(&want), "missing {want}");
    }
    let scratch_tmp = stdout
        .lines()
        .find_map(|line| line.strip_prefix("SCRATCH_TMP="))
        .expect("the child printed its scratch tmp");
    assert!(
        launched.contains(&format!("TMPDIR={scratch_tmp}").as_str()),
        "TMPDIR is not the scratch tmp {scratch_tmp}:\n{launched:?}"
    );
}

/// What a child run printed: the launched variables as `(name, value)` and the
/// lines the child itself printed.
struct Seen {
    vars: Vec<(String, String)>,
    stdout: String,
}

impl Seen {
    fn of(harness: &str) -> Seen {
        let output = run_child_for(Some(harness));
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(
            output.status.success(),
            "the control child must succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        // Control: the parent held CODEX_HOME, so its absence below is the launcher's doing.
        assert!(stdout.contains("PARENT_SAW_PROXY=true"), "{stdout}");
        let vars: Vec<(String, String)> = stdout
            .lines()
            .filter_map(|line| line.strip_prefix("LAUNCHED:"))
            .filter_map(|line| line.split_once('='))
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        assert!(
            !vars.is_empty(),
            "the launched process printed nothing:\n{stdout}"
        );
        Seen { vars, stdout }
    }

    fn line(&self, key: &str) -> &str {
        self.stdout
            .lines()
            .find_map(|line| line.strip_prefix(key))
            .unwrap_or_else(|| panic!("the child did not print {key}"))
    }

    /// The launched names beyond the four every launch gets, and any of the four missing.
    fn names_off(&self, want: &[&str]) -> Vec<String> {
        let mut off: Vec<String> = self
            .vars
            .iter()
            .map(|(name, _)| name.clone())
            .filter(|name| !want.contains(&name.as_str()))
            .collect();
        off.extend(
            want.iter()
                .filter(|name| !self.vars.iter().any(|(have, _)| have == *name))
                .map(|name| format!("missing {name}")),
        );
        off
    }

    fn value(&self, name: &str) -> Option<&str> {
        self.vars
            .iter()
            .find(|(have, _)| have == name)
            .map(|(_, value)| value.as_str())
    }
}

#[test]
fn a_codex_child_gets_the_four_variables_and_codex_home_which_is_its_harness_home() {
    let seen = Seen::of("codex");
    let off = seen.names_off(&["CODEX_HOME", "HOME", "LANG", "PATH", "TMPDIR"]);
    assert!(off.is_empty(), "the codex environment is off by: {off:?}");

    let harness_home = seen.line("HARNESS_HOME=");
    assert_eq!(
        seen.value("CODEX_HOME"),
        Some(harness_home),
        "CODEX_HOME is not the launcher's harness home"
    );
    assert_eq!(seen.line("HARNESS_HOME_IS_DIR="), "true");
    assert!(
        harness_home.starts_with(seen.line("SCRATCH_ROOT=")),
        "the codex home {harness_home} is outside the scratch root"
    );
    assert_ne!(
        harness_home,
        seen.line("SCRATCH_HOME="),
        "codex shares the scratch HOME as its home"
    );
}

#[test]
fn a_claude_code_or_goose_child_has_no_codex_home_and_works_in_the_scratch_home() {
    for harness in ["claude-code", "goose"] {
        let seen = Seen::of(harness);
        let off = seen.names_off(&["HOME", "LANG", "PATH", "TMPDIR"]);
        assert!(
            off.is_empty(),
            "the {harness} environment is off by: {off:?}"
        );
        assert_eq!(
            seen.line("HARNESS_HOME="),
            seen.line("SCRATCH_HOME="),
            "the {harness} harness home is not the scratch HOME"
        );
    }
}
