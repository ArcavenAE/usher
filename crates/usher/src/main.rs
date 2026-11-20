//! The usher binary. Mode selection only.
//!
//! Knows `--version` and hands `inventory` to the `usher-cli` crate, which holds the
//! verb. Every other argument, or none, is an unknown mode and exits nonzero with a next
//! action. The real modes (`mcp`, `daemon`, `collector`) arrive with the stories that
//! build them.

#![forbid(unsafe_code)]

use std::{env, process::ExitCode};

use usher_core::UsherError;

const STEP: &str = "select mode";

fn main() -> ExitCode {
    let args: Vec<String> = env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let result = if matches!(args.first().map(String::as_str), Some("inventory" | "map")) {
        usher_cli::run_process(&args)
    } else {
        run(args.into_iter())
    };
    match result {
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
            "run `usher --version` or `usher inventory`",
        )),
        None => Err(UsherError::new(
            STEP,
            "a mode was given",
            "no mode",
            "run `usher --version` or `usher inventory`",
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

    /// A mistyped or missing mode is told every mode there is, including the map verb.
    #[test]
    fn a_mistyped_or_missing_mode_is_told_the_next_action_names_every_verb() {
        let expected = "run `usher inventory`, `usher map <directory>` or `usher --version`";
        let err = run(args(&["mpa"])).expect_err("mpa is not a mode");
        assert_eq!(err.next_action(), expected);
        let err = run(args(&[])).expect_err("a mode is required");
        assert_eq!(err.next_action(), expected);
    }

    #[test]
    fn missing_mode_is_an_error_with_a_next_action() {
        let err = run(args(&[])).expect_err("a mode is required");
        assert_eq!(err.status(), "no mode");
        assert!(!err.next_action().is_empty());
    }
}
