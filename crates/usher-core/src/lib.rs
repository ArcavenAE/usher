//! Domain types shared by every usher crate.
//!
//! Story 1.1 holds one type, [`UsherError`]. Later stories add the map, the
//! proposal and journal rules, the report schema and the environment key.

#![forbid(unsafe_code)]

use std::{error::Error, fmt};

mod environment;
mod listing;
mod version;

pub use environment::{EnvironmentKey, environment_key, key_variables};
pub use listing::{
    AbsentLayer, CheckedPath, KeyNames, LaunchContext, ListedSource, NotCheckedLayer, SeatLine,
    SourceLister, SourceListing, SourceStatus, SystemRoot,
};
pub use version::{Version, VersionRange};

/// The one error type every usher surface reports through.
///
/// A person who hits a failure learns four things: the step that was running,
/// the command or check that failed, its exit status or error text, and at
/// least one thing to do next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsherError {
    step: String,
    check: String,
    status: String,
    next_action: String,
}

impl UsherError {
    /// Builds an error from the four fields every failure must carry.
    pub fn new(
        step: impl Into<String>,
        check: impl Into<String>,
        status: impl Into<String>,
        next_action: impl Into<String>,
    ) -> Self {
        Self {
            step: step.into(),
            check: check.into(),
            status: status.into(),
            next_action: next_action.into(),
        }
    }

    /// The step that was running when the failure happened.
    pub fn step(&self) -> &str {
        &self.step
    }

    /// The command or check that failed.
    pub fn check(&self) -> &str {
        &self.check
    }

    /// The exit status or error text of the failed check.
    pub fn status(&self) -> &str {
        &self.status
    }

    /// What the person can do next.
    pub fn next_action(&self) -> &str {
        &self.next_action
    }
}

impl fmt::Display for UsherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "step: {}\ncheck: {}\nstatus: {}\nnext: {}",
            self.step, self.check, self.status, self.next_action
        )
    }
}

impl Error for UsherError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> UsherError {
        UsherError::new(
            "select mode",
            "mode `frobnicate` is known",
            "unknown mode",
            "run `usher --version`, or name a mode this build supports",
        )
    }

    #[test]
    fn usher_error_keeps_all_four_fields() {
        let err = sample();
        assert_eq!(err.step(), "select mode");
        assert_eq!(err.check(), "mode `frobnicate` is known");
        assert_eq!(err.status(), "unknown mode");
        assert_eq!(
            err.next_action(),
            "run `usher --version`, or name a mode this build supports"
        );
    }

    #[test]
    fn usher_error_display_names_step_check_status_and_next_action() {
        let text = sample().to_string();
        assert!(text.contains("step: select mode"));
        assert!(text.contains("check: mode `frobnicate` is known"));
        assert!(text.contains("status: unknown mode"));
        assert!(text.contains("next: run `usher --version`"));
    }

    #[test]
    fn usher_error_is_a_std_error() {
        let err: Box<dyn Error> = Box::new(sample());
        assert!(err.to_string().contains("next:"));
    }
}
