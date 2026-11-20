//! The usher binary. Mode selection only.
//!
//! Story 1.1 knows one flag, `--version`. Every other argument, or none, is an
//! unknown mode and exits nonzero with a next action. The real modes (`mcp`,
//! `daemon`, `collector`) arrive with the stories that build them.

#![forbid(unsafe_code)]

use std::{env, process::ExitCode};

use usher_core::UsherError;

const STEP: &str = "select mode";

fn main() -> ExitCode {
    match run(env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned()))
    {
        Ok(out) => {
            println!("{out}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("usher: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Picks a mode from the arguments and returns the text to print.
fn run(mut args: impl Iterator<Item = String>) -> Result<String, UsherError> {
    match args.next().as_deref() {
        Some("--version" | "-V") => Ok(format!("usher {}", env!("CARGO_PKG_VERSION"))),
        Some(other) => Err(UsherError::new(
            STEP,
            format!("mode `{}` is one this build supports", other.escape_debug()),
            "unknown mode",
            "run `usher --version`; no other mode exists in this build",
        )),
        None => Err(UsherError::new(
            STEP,
            "a mode was given",
            "no mode",
            "run `usher --version`; no other mode exists in this build",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> impl Iterator<Item = String> {
        list.iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn version_flag_prints_name_and_version() {
        let out = run(args(&["--version"])).expect("--version is a known mode");
        assert_eq!(out, format!("usher {}", env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn unknown_mode_names_step_check_status_and_next_action() {
        let err = run(args(&["frobnicate"])).expect_err("frobnicate is not a mode");
        assert_eq!(err.step(), STEP);
        assert!(err.check().contains("frobnicate"));
        assert_eq!(err.status(), "unknown mode");
        assert!(err.next_action().contains("usher --version"));
    }

    #[test]
    fn missing_mode_is_an_error_with_a_next_action() {
        let err = run(args(&[])).expect_err("a mode is required");
        assert_eq!(err.status(), "no mode");
        assert!(!err.next_action().is_empty());
    }
}
