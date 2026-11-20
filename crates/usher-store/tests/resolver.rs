//! Story 1.5, PR C: where the state directory is, resolved from the account database and
//! never from `$HOME`, and who may own it. Runs against a scratch home; the account
//! database is read, never written.

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use usher_store::{
    Platform, Store, account_home, account_home_of, ensure_owned, require_absolute_home,
    state_dir_under,
};
use usher_testkit::ScratchHome;

/// A uid that the account database of this host maps to no account, found by asking it:
/// the top of the uid range holds `nobody` on macOS, so no constant is safe everywhere.
fn unmapped_uid() -> u32 {
    (0..64u32)
        .map(|step| 3_999_999_999 - step)
        .find(|uid| {
            matches!(
                nix::unistd::User::from_uid(nix::unistd::Uid::from_raw(*uid)),
                Ok(None)
            )
        })
        .expect("a uid among the 64 below 3_999_999_999 that no account has")
}

/// The uid of whoever runs the tests: the owner of a file the test just made.
fn current_uid() -> u32 {
    let scratch = ScratchHome::new().expect("scratch");
    fs::metadata(scratch.home()).expect("metadata").uid()
}

#[test]
fn the_state_directory_is_the_platform_suffix_under_the_account_home() {
    let home = Path::new("/home/alex");
    assert_eq!(
        state_dir_under(home, Platform::Linux),
        PathBuf::from("/home/alex/.local/state/usher")
    );
    assert_eq!(
        state_dir_under(Path::new("/Users/alex"), Platform::Macos),
        PathBuf::from("/Users/alex/Library/Application Support/usher")
    );
}

#[test]
fn the_account_home_of_the_current_uid_is_an_absolute_path() {
    let home = account_home_of(current_uid()).expect("the account database knows this user");
    assert!(home.is_absolute(), "{}", home.display());
}

#[test]
fn the_current_account_home_is_that_of_the_current_uid() {
    assert_eq!(
        account_home().expect("the current account home"),
        account_home_of(current_uid()).expect("by uid")
    );
}

#[test]
fn an_unknown_uid_is_refused_naming_it_with_a_next_action() {
    let unknown = unmapped_uid();
    let err = account_home_of(unknown).expect_err("no such account");
    assert_eq!(err.step(), "resolve the account home");
    assert!(
        err.status().contains(&unknown.to_string()),
        "{}",
        err.status()
    );
    assert!(!err.next_action().is_empty());
}

#[test]
fn a_state_directory_owned_by_another_user_is_refused_naming_both_uids() {
    let dir = Path::new("/somewhere/usher");
    ensure_owned(dir, 501, 501).expect("the owner may use it");
    let err = ensure_owned(dir, 0, 501).expect_err("another owner is refused");
    assert_eq!(err.step(), "check the state directory");
    let status = err.status();
    assert!(status.contains("/somewhere/usher"), "{status}");
    assert!(status.contains('0') && status.contains("501"), "{status}");
    assert!(!err.next_action().is_empty());
}

/// Run by the test below as a child process with `HOME` pointed elsewhere: it prints the
/// account home it resolves and does nothing otherwise.
#[test]
#[ignore = "run only by the_account_home_does_not_follow_home, as a child process"]
fn child_reports_the_account_home() {
    if std::env::var_os("USHER_RESOLVER_CHILD").is_some() {
        println!(
            "ACCOUNT_HOME={}",
            account_home().expect("an account home").display()
        );
    }
}

/// The account database, not `$HOME`, names the home. Setting `HOME` in this process is
/// unsafe in edition 2024, so a child test process runs with `HOME` elsewhere.
#[test]
fn the_account_home_does_not_follow_home() {
    let scratch = ScratchHome::new().expect("scratch");
    let elsewhere = scratch.root().join("not-the-account-home");
    fs::create_dir_all(&elsewhere).expect("dirs");
    let out = std::process::Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            "--ignored",
            "--exact",
            "child_reports_the_account_home",
            "--nocapture",
        ])
        .env("USHER_RESOLVER_CHILD", "1")
        .env("HOME", &elsewhere)
        .output()
        .expect("the child runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let child_home = stdout
        .lines()
        .find_map(|line| line.strip_prefix("ACCOUNT_HOME="))
        .unwrap_or_else(|| panic!("the child printed no account home: {stdout}"));
    assert_ne!(Path::new(child_home), elsewhere, "the child followed HOME");
    assert_eq!(
        Path::new(child_home),
        account_home().expect("this process's account home"),
        "the same user resolves the same home whatever HOME says"
    );
    assert!(Path::new(child_home).is_absolute(), "{child_home}");
}

/// The store open checks the directory's real owner against the uid it is given.
#[test]
fn the_store_open_refuses_a_directory_owned_by_another_uid() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = scratch.home().join(".local/state/usher");
    fs::create_dir_all(&dir).expect("dirs");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).expect("mode");
    let mine = fs::metadata(&dir).expect("metadata").uid();
    Store::open_with_owner(&dir, mine).expect("the owner may open it");
    let err = Store::open_with_owner(&dir, mine + 1).expect_err("another expected owner");
    assert_eq!(err.step(), "check the state directory");
    assert!(err.status().contains(&mine.to_string()), "{}", err.status());
    assert!(
        err.status().contains(&(mine + 1).to_string()),
        "{}",
        err.status()
    );
}

/// A home the account database gives as a relative path would put the store under the
/// launch directory, so it is refused naming the uid and the path.
#[test]
fn a_relative_account_home_is_refused_naming_the_uid_and_the_path() {
    assert_eq!(
        require_absolute_home(501, Path::new("/home/alex")).expect("an absolute home"),
        PathBuf::from("/home/alex")
    );
    for relative in ["unresolved", "./home", ""] {
        let err = require_absolute_home(501, Path::new(relative)).expect_err(relative);
        assert_eq!(err.step(), "resolve the account home");
        assert!(err.status().contains("501"), "{}", err.status());
        assert!(err.status().contains(relative), "{}", err.status());
        assert!(
            err.status().contains("not an absolute path"),
            "{}",
            err.status()
        );
        assert!(!err.next_action().is_empty());
    }
}
