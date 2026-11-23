//! `usher tools` through `run`'s own dispatch and path resolution, over a scratch home
//! and scratch brew locations. The child runs `run` with the installer variables set by
//! the parent, so nothing of the host's /opt/homebrew or real home is read.

use std::{fs, os::unix::fs::symlink, path::PathBuf};

use usher_cli::{InventoryEnv, LaunchEnvironment, run};
use usher_knowledge::Platform;
use usher_testkit::ScratchHome;

const INSTALLER_VARIABLES: [&str; 9] = [
    "HOMEBREW_REPOSITORY",
    "HOMEBREW_PREFIX",
    "HOMEBREW_CACHE",
    "MISE_DATA_DIR",
    "MISE_CACHE_DIR",
    "MISE_STATE_DIR",
    "XDG_DATA_HOME",
    "XDG_CACHE_HOME",
    "XDG_STATE_HOME",
];

/// Run by the test below as a child process with the installer variables set.
#[test]
#[ignore = "run only by run_reads_one_brew_row_and_one_mise_row_from_scratch, as a child process"]
fn child_runs_tools_over_the_process_environment() {
    let Some(root) = std::env::var_os("USHER_TOOLS_CHILD_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let home = root.join("home");
    let env = InventoryEnv {
        host: "host-1".to_string(),
        user: "alex".to_string(),
        home: home.clone(),
        launch_dir: home.clone(),
        launch: LaunchEnvironment::default(),
        path: None,
        state_dir: home.join("state"),
        now: "2026-11-20T12:00:00Z".to_string(),
        platform: Platform::Linux,
    };
    let out = run(&["tools".to_string()], &env).expect("`usher tools` is a command");
    assert!(
        out.contains("goose  installed  brew homebrew/core  1.2.3"),
        "the brew row is not from the scratch brew:\n{out}"
    );
    assert!(
        out.contains("codex  installed  mise aqua  0.160.0"),
        "the mise row is not from the scratch home:\n{out}"
    );
    assert!(
        !out.contains("inventory"),
        "tools printed the inventory:\n{out}"
    );
    println!("CHILD_OK");
}

#[test]
fn run_reads_one_brew_row_and_one_mise_row_from_scratch() {
    let scratch = ScratchHome::new().expect("scratch");
    let root = scratch.root().to_path_buf();
    let brew = root.join("brew");
    let cellar = brew.join("Cellar/block-goose-cli/1.2.3");
    fs::create_dir_all(&cellar).expect("a cellar entry");
    fs::write(
        cellar.join("INSTALL_RECEIPT.json"),
        r#"{"source":{"tap":"homebrew/core"}}"#,
    )
    .expect("receipt");
    fs::create_dir_all(brew.join("opt")).expect("opt");
    symlink(
        "../Cellar/block-goose-cli/1.2.3",
        brew.join("opt/block-goose-cli"),
    )
    .expect("opt link");
    let home = root.join("home");
    let installs = home.join(".local/share/mise/installs/codex");
    fs::create_dir_all(installs.join("0.160.0")).expect("a mise install");
    fs::write(
        installs.join(".mise.backend.toml"),
        "short = \"codex\"\nfull = \"aqua:openai/codex\"\n",
    )
    .expect("backend file");
    let mut child = std::process::Command::new(std::env::current_exe().expect("this test binary"));
    child.args([
        "--ignored",
        "--exact",
        "child_runs_tools_over_the_process_environment",
        "--nocapture",
    ]);
    for name in INSTALLER_VARIABLES {
        child.env_remove(name);
    }
    let out = child
        .env("HOME", &home)
        .env("HOMEBREW_REPOSITORY", &brew)
        .env("HOMEBREW_PREFIX", &brew)
        .env("USHER_TOOLS_CHILD_ROOT", &root)
        .output()
        .expect("a child run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success() && stdout.contains("CHILD_OK"),
        "{stdout}{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
