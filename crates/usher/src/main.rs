//! The usher binary. Mode selection only.
//!
//! Knows `--version` and hands `inventory` and `tools` to the `usher-cli` crate, which holds the
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
    let result = if hands_to_cli(args.first().map(String::as_str)) {
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

/// Whether the first argument names a verb the `usher-cli` crate holds.
fn hands_to_cli(first: Option<&str>) -> bool {
    matches!(first, Some("inventory" | "tools"))
}

/// Picks a mode from the arguments and returns the text to print.
fn run(mut args: impl Iterator<Item = String>) -> Result<String, UsherError> {
    match args.next().as_deref() {
        Some("--version" | "-V") => Ok(format!("usher {}", env!("CARGO_PKG_VERSION"))),
        Some(other) => Err(UsherError::new(
            STEP,
            format!("mode `{}` is one this build supports", other.escape_debug()),
            "unknown mode",
            "run `usher inventory`, `usher tools` or `usher --version`",
        )),
        None => Err(UsherError::new(
            STEP,
            "a mode was given",
            "no mode",
            "run `usher inventory`, `usher tools` or `usher --version`",
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
    fn the_tools_verb_is_handed_to_the_cli_crate_beside_inventory() {
        assert!(hands_to_cli(Some("inventory")));
        assert!(hands_to_cli(Some("tools")));
        assert!(!hands_to_cli(Some("--version")));
        assert!(!hands_to_cli(None));
    }

    #[test]
    fn the_unknown_and_missing_mode_next_actions_name_the_tools_verb() {
        let unknown = run(args(&["frobnicate"])).expect_err("not a mode");
        assert!(unknown.next_action().contains("usher tools"), "{unknown}");
        let missing = run(args(&[])).expect_err("a mode is required");
        assert!(missing.next_action().contains("usher tools"), "{missing}");
    }

    #[test]
    fn missing_mode_is_an_error_with_a_next_action() {
        let err = run(args(&[])).expect_err("a mode is required");
        assert_eq!(err.status(), "no mode");
        assert!(!err.next_action().is_empty());
    }
}
