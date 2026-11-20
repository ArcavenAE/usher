//! The host name `usher inventory` records is the system's own, read from the system and
//! not by running a program found on `PATH`. A child process runs with a planted `hostname`
//! program first on its `PATH`; the host it reports must still be the system's name.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use usher_cli::InventoryEnv;

const PLANTED: &str = "planted-host-name";

/// Run by the test below, as a child with the planted program on its `PATH`.
#[test]
#[ignore = "run only by the_host_name_is_not_answered_by_a_program_on_the_path, as a child process"]
fn child_prints_the_host_it_reads() {
    let env = InventoryEnv::from_process().expect("this process's environment");
    println!("HOST={}", env.host);
}

#[test]
fn the_host_name_is_not_answered_by_a_program_on_the_path() {
    let scratch = usher_testkit::ScratchHome::new().expect("scratch");
    let bin = scratch.root().join("planted-bin");
    fs::create_dir_all(&bin).expect("bin dir");
    let program = bin.join("hostname");
    fs::write(&program, format!("#!/bin/sh\necho {PLANTED}\n")).expect("a program");
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("mode");
    // The system's own name, from the system's hostname program by absolute path, which
    // asks the kernel the same question gethostname does.
    let system = Command::new("/bin/hostname")
        .output()
        .expect("/bin/hostname runs");
    let expected = String::from_utf8_lossy(&system.stdout).trim().to_string();
    assert!(!expected.is_empty() && expected != PLANTED);
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            "--ignored",
            "--exact",
            "child_prints_the_host_it_reads",
            "--nocapture",
        ])
        .env("PATH", path)
        .output()
        .expect("the child runs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "the child failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let line = stdout
        .lines()
        .find_map(|l| l.strip_prefix("HOST="))
        .unwrap_or_else(|| panic!("the child printed no host:\n{stdout}"));
    assert_eq!(
        line, expected,
        "the host name came from a program on the PATH"
    );
}
