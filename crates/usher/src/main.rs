//! The usher binary. Mode selection only.
//!
//! It knows `--version` and `inventory` (Story 1.5). Every other argument, or
//! none, is an unknown mode and exits nonzero with a next action. The real
//! modes (`mcp`, `daemon`, `collector`) arrive with the stories that build them.

#![forbid(unsafe_code)]

use std::{env, ffi::OsString, path::PathBuf, process::ExitCode};

use usher_adapter_claude_code::{Environment, inventory};
use usher_core::UsherError;
use usher_knowledge::KnowledgeFile;

const STEP: &str = "select mode";
const NEXT: &str = "run `usher --version` or `usher inventory`; no other mode exists in this build";

fn main() -> ExitCode {
    match run(
        env::args_os()
            .skip(1)
            .map(|a| a.to_string_lossy().into_owned()),
        |name| env::var_os(name),
    ) {
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
fn run(
    mut args: impl Iterator<Item = String>,
    get: impl Fn(&str) -> Option<OsString>,
) -> Result<String, UsherError> {
    match args.next().as_deref() {
        Some("--version" | "-V") => Ok(format!("usher {}", env!("CARGO_PKG_VERSION"))),
        Some("inventory") => list_sources(args, get),
        Some(other) => Err(UsherError::new(
            STEP,
            format!("mode `{}` is one this build supports", other.escape_debug()),
            "unknown mode",
            NEXT,
        )),
        None => Err(UsherError::new(STEP, "a mode was given", "no mode", NEXT)),
    }
}

const INVENTORY_STEP: &str = "list settings sources";

/// `usher inventory [--root DIR]...`: lists Claude Code's settings sources.
fn list_sources(
    mut args: impl Iterator<Item = String>,
    get: impl Fn(&str) -> Option<OsString>,
) -> Result<String, UsherError> {
    let mut roots: Vec<PathBuf> = Vec::new();
    while let Some(arg) = args.next() {
        let value = if arg == "--root" {
            args.next().ok_or_else(|| {
                UsherError::new(
                    INVENTORY_STEP,
                    "`--root` is followed by a directory",
                    "`--root` has no value",
                    "pass a directory, as in `usher inventory --root ~/work/my-repo`",
                )
            })?
        } else if let Some(value) = arg.strip_prefix("--root=") {
            value.to_string()
        } else {
            return Err(UsherError::new(
                INVENTORY_STEP,
                format!(
                    "argument `{}` is one `usher inventory` takes",
                    arg.escape_debug()
                ),
                "unknown argument",
                "run `usher inventory`, with `--root <dir>` for each project to look in",
            ));
        };
        roots.push(absolute(PathBuf::from(value)));
    }
    let mut environment = Environment::from_process(get)?;
    for root in roots {
        environment = environment.with_root(root);
    }
    let knowledge = KnowledgeFile::embedded()?;
    let found = inventory(&environment, &knowledge)?;
    Ok(found.to_string().trim_end().to_string())
}

/// Makes a path absolute against the current directory without touching the disk.
fn absolute(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        env::current_dir().map_or(path.clone(), |cwd| cwd.join(path))
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
        let out = run(args(&["--version"]), |_| None).expect("--version is a known mode");
        assert_eq!(out, format!("usher {}", env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn unknown_mode_names_step_check_status_and_next_action() {
        let err = run(args(&["frobnicate"]), |_| None).expect_err("frobnicate is not a mode");
        assert_eq!(err.step(), STEP);
        assert!(err.check().contains("frobnicate"));
        assert_eq!(err.status(), "unknown mode");
        assert!(err.next_action().contains("usher --version"));
        assert!(err.next_action().contains("usher inventory"));
    }

    #[test]
    fn inventory_arguments_are_checked() {
        let none = |_: &str| None;
        let err = run(args(&["inventory", "--root"]), none).expect_err("no value");
        assert_eq!(err.status(), "`--root` has no value");
        let err = run(args(&["inventory", "--bogus"]), none).expect_err("unknown flag");
        assert_eq!(err.status(), "unknown argument");
        assert!(!err.next_action().is_empty());
        // Arguments are valid, so the next failure is the missing HOME.
        let err = run(
            args(&["inventory", "--root=/tmp/x", "--root", "/tmp/y"]),
            none,
        )
        .expect_err("no HOME");
        assert_eq!(err.status(), "HOME is not set");
    }

    #[test]
    fn missing_mode_is_an_error_with_a_next_action() {
        let err = run(args(&[]), |_| None).expect_err("a mode is required");
        assert_eq!(err.status(), "no mode");
        assert!(!err.next_action().is_empty());
    }
}
