//! Story 1.5, PR C: where the state directory is, resolved from the account database and
//! never from `$HOME`, and who may own it. Runs against a scratch home; the account
//! database is read, never written.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use usher_store::{Platform, account_home, account_home_of, ensure_owned, state_dir_under};
use usher_testkit::ScratchHome;

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
    let err = account_home_of(u32::MAX - 1).expect_err("no such account");
    assert_eq!(err.step(), "resolve the account home");
    assert!(
        err.status().contains(&(u32::MAX - 1).to_string()),
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
