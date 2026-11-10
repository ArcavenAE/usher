//! Reads `USHER_TRUSTED_KEYS` and writes the build-time trusted-key list
//! (AD-12). A release build with an empty list, a malformed key, or the
//! `test-keys` feature on does not compile.

#[path = "build/keys.rs"]
mod keys;

use std::{
    env::{self, VarError},
    fs,
    path::PathBuf,
    process::exit,
};

const VARIABLE: &str = "USHER_TRUSTED_KEYS";

fn stop(message: &str) -> ! {
    eprintln!("usher-knowledge build: {message}");
    exit(1);
}

fn main() {
    println!("cargo:rerun-if-env-changed={VARIABLE}");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build/keys.rs");

    let release = env::var("PROFILE").is_ok_and(|profile| profile == "release");
    let list = match env::var(VARIABLE) {
        Ok(list) => list,
        Err(VarError::NotPresent) => String::new(),
        Err(VarError::NotUnicode(_)) => stop(&format!("{VARIABLE} is not valid UTF-8")),
    };
    let entries = match keys::parse(&list) {
        Ok(entries) => entries,
        Err(why) => stop(&format!("{VARIABLE} is not valid: {why}")),
    };
    let test_keys = env::var_os("CARGO_FEATURE_TEST_KEYS").is_some();
    if let Err(why) = keys::check_release(release, test_keys, &entries) {
        stop(&why);
    }

    let Some(out_dir) = env::var_os("OUT_DIR") else {
        stop("cargo did not set OUT_DIR");
    };
    let target = PathBuf::from(out_dir).join("trusted_keys.rs");
    if let Err(why) = fs::write(&target, keys::render(&entries)) {
        stop(&format!("cannot write {}: {why}", target.display()));
    }
}
