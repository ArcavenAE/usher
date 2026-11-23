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
    // The system's own name, asked of the system directly.
    let expected = nix::unistd::gethostname()
        .expect("the system host name")
        .to_string_lossy()
        .trim()
        .to_string();
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

/// The refusal for a host name that cannot be read says the system host name could not be
/// read, and names no program. The system call cannot be made to fail from a test, so the
/// text is pinned by reading the source that holds it.
#[test]
fn the_host_refusal_names_the_system_host_name_and_no_program() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/inventory.rs");
    let source = fs::read_to_string(&path).expect("the inventory source");
    let start = source
        .find("fn host_name()")
        .expect("the host name function");
    let body = &source[start..];
    let body = &body[..body.find("\n}\n").expect("the function ends")];
    assert!(
        body.contains(r#""the system host name can be read""#),
        "the check text changed:\n{body}"
    );
    assert!(
        body.contains(
            "fix the system host name so it can be read, then run `usher inventory` again"
        ),
        "the next action changed:\n{body}"
    );
    assert!(
        !body.contains("`hostname`") && !body.contains("hostname command"),
        "the refusal names a program:\n{body}"
    );
}
