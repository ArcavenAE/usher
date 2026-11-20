//! Story 1.5, PR C: where `InventoryEnv::from_process` gets the home, the state directory
//! and the user (AD-1): from the account database, never from `$HOME`. Read-only: it opens
//! no store and writes nothing.

use std::path::{Path, PathBuf};

use usher_cli::{InventoryEnv, inventory_platform};
use usher_store::{Platform, account_home, account_name, state_dir_under};
use usher_testkit::ScratchHome;

fn host_store_platform() -> Platform {
    match inventory_platform() {
        usher_knowledge::Platform::Macos => Platform::Macos,
        _ => Platform::Linux,
    }
}

fn assert_from_the_account_database(env: &InventoryEnv) {
    let home = account_home().expect("the account home");
    assert_eq!(env.home, home);
    assert_eq!(env.state_dir, state_dir_under(&home, host_store_platform()));
    assert_eq!(env.user, account_name().expect("the account name"));
}

#[test]
fn the_process_environment_takes_home_state_directory_and_user_from_the_account_database() {
    let env = InventoryEnv::from_process().expect("this process's environment");
    assert_from_the_account_database(&env);
}

/// Run by the test below as a child process with `HOME` pointed elsewhere.
#[test]
#[ignore = "run only by home_does_not_move_the_verbs_home, as a child process"]
fn child_checks_the_process_environment() {
    if let Some(elsewhere) = std::env::var_os("USHER_ENV_CHILD") {
        let env = InventoryEnv::from_process().expect("this process's environment");
        assert_ne!(
            env.home,
            PathBuf::from(&elsewhere),
            "the child followed HOME"
        );
        assert!(
            !env.state_dir.starts_with(Path::new(&elsewhere)),
            "the state directory followed HOME: {}",
            env.state_dir.display()
        );
        assert_from_the_account_database(&env);
        println!("CHILD_OK");
    }
}

#[test]
fn home_does_not_move_the_verbs_home() {
    let scratch = ScratchHome::new().expect("scratch");
    let elsewhere = scratch.root().join("not-the-account-home");
    std::fs::create_dir_all(&elsewhere).expect("dirs");
    let out = std::process::Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            "--ignored",
            "--exact",
            "child_checks_the_process_environment",
            "--nocapture",
        ])
        .env("USHER_ENV_CHILD", &elsewhere)
        .env("HOME", &elsewhere)
        .output()
        .expect("the child runs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success() && stdout.contains("CHILD_OK"),
        "{stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
