//! A scratch copy of a codex home holds config files only, never credentials,
//! and is deleted when it is dropped.

use std::fs;

use usher_probes::codex_home::CodexCopy;
use usher_testkit::ScratchHome;

fn source_home(scratch: &ScratchHome) -> std::path::PathBuf {
    let source = scratch.root().join("real-codex-home");
    fs::create_dir_all(source.join("sessions")).expect("source dirs");
    fs::write(source.join("config.toml"), "model = \"demo\"\n").expect("config");
    fs::write(source.join("extra.toml"), "x = 1\n").expect("extra toml");
    fs::write(source.join("auth.json"), "{\"token\":\"not-a-real-token\"}").expect("auth");
    fs::write(source.join("history.jsonl"), "{}\n").expect("history");
    fs::write(source.join("sessions").join("one.jsonl"), "{}\n").expect("session");
    source
}

#[test]
fn only_top_level_toml_config_files_are_copied() {
    let scratch = ScratchHome::new().expect("scratch home");
    let source = source_home(&scratch);
    let copy = CodexCopy::create(&source, &scratch).expect("a copy");
    let mut names: Vec<String> = fs::read_dir(copy.path())
        .expect("copy dir")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    assert_eq!(names, ["config.toml", "extra.toml"]);
    assert_eq!(
        fs::read_to_string(copy.path().join("config.toml")).expect("copied config"),
        "model = \"demo\"\n"
    );
}

#[test]
fn credentials_are_never_copied() {
    let scratch = ScratchHome::new().expect("scratch home");
    let source = source_home(&scratch);
    let copy = CodexCopy::create(&source, &scratch).expect("a copy");
    assert!(!copy.path().join("auth.json").exists());
}

#[test]
fn the_copy_lives_under_the_scratch_root_and_not_in_the_source() {
    let scratch = ScratchHome::new().expect("scratch home");
    let source = source_home(&scratch);
    let copy = CodexCopy::create(&source, &scratch).expect("a copy");
    assert!(copy.path().starts_with(scratch.root()));
    assert_ne!(copy.path(), source.as_path());
}

#[test]
fn dropping_the_copy_deletes_it_and_leaves_the_source() {
    let scratch = ScratchHome::new().expect("scratch home");
    let source = source_home(&scratch);
    let copy = CodexCopy::create(&source, &scratch).expect("a copy");
    let path = copy.path().to_path_buf();
    assert!(
        path.join("config.toml").exists(),
        "control: the copy exists"
    );
    drop(copy);
    assert!(!path.exists(), "the copy outlived its owner");
    assert!(source.join("auth.json").exists(), "the source was touched");
}

#[test]
fn a_missing_source_is_an_error_naming_not_found() {
    let scratch = ScratchHome::new().expect("scratch home");
    let missing = scratch.root().join("no-such-home");
    let err = CodexCopy::create(&missing, &scratch).expect_err("no source, no copy");
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn a_symlinked_toml_file_is_not_copied_and_no_directory_is_entered() {
    let scratch = ScratchHome::new().expect("scratch home");
    let source = source_home(&scratch);
    // A symlink to a file outside the source, one to a file inside it, and a
    // directory whose name ends in .toml: none is followed or copied.
    let outside = scratch.root().join("outside.toml");
    fs::write(&outside, "secret = true\n").expect("outside file");
    std::os::unix::fs::symlink(&outside, source.join("linked-out.toml")).expect("symlink out");
    std::os::unix::fs::symlink(source.join("config.toml"), source.join("linked-in.toml"))
        .expect("symlink in");
    fs::create_dir_all(source.join("dir.toml")).expect("a directory named like a toml file");
    fs::write(source.join("dir.toml").join("inner.toml"), "y = 2\n").expect("inner file");

    let copy = CodexCopy::create(&source, &scratch).expect("a copy");
    let mut names: Vec<String> = fs::read_dir(copy.path())
        .expect("copy dir")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    assert_eq!(names, ["config.toml", "extra.toml"]);
}
