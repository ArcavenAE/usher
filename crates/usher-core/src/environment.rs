//! The environment key (AD-17): host, OS user, harness, absolute directory and the
//! launch-context variables harness knowledge names for that harness.
//!
//! One function builds the key and every mode uses it, so one environment is never
//! keyed two ways.

use std::path::{Path, PathBuf};

use crate::UsherError;

/// The key of one environment. Build it with [`environment_key`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EnvironmentKey {
    host: String,
    os_user: String,
    harness: String,
    directory: PathBuf,
    launch: Vec<(String, Option<String>)>,
}

impl EnvironmentKey {
    /// The host id.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// The OS user.
    pub fn os_user(&self) -> &str {
        &self.os_user
    }

    /// The harness id.
    pub fn harness(&self) -> &str {
        &self.harness
    }

    /// The absolute directory.
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// The launch-context variables, by name, `None` where a variable is unset.
    pub fn launch(&self) -> &[(String, Option<String>)] {
        &self.launch
    }
}

/// Builds the key of an environment.
pub fn environment_key(
    host: &str,
    os_user: &str,
    harness: &str,
    directory: &Path,
    launch: &[(&str, Option<&str>)],
) -> Result<EnvironmentKey, UsherError> {
    let _ = (host, os_user, harness, directory, launch);
    Ok(EnvironmentKey {
        host: String::new(),
        os_user: String::new(),
        harness: String::new(),
        directory: PathBuf::new(),
        launch: Vec::new(),
    })
}

impl std::fmt::Display for EnvironmentKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(launch: &[(&str, Option<&str>)]) -> EnvironmentKey {
        environment_key(
            "host-1",
            "alex",
            "claude-code",
            Path::new("/Users/alex/work/repo"),
            launch,
        )
        .expect("an absolute directory is accepted")
    }

    #[test]
    fn the_key_carries_host_user_harness_and_directory() {
        let key = key(&[]);
        assert_eq!(key.host(), "host-1");
        assert_eq!(key.os_user(), "alex");
        assert_eq!(key.harness(), "claude-code");
        assert_eq!(key.directory(), Path::new("/Users/alex/work/repo"));
    }

    #[test]
    fn a_relative_directory_is_refused_with_the_four_fields() {
        let err = environment_key("h", "u", "claude-code", Path::new("repo"), &[])
            .expect_err("a relative directory has no one meaning");
        assert_eq!(err.step(), "key an environment");
        assert!(err.check().contains("absolute"), "check: {}", err.check());
        assert!(err.status().contains("repo"), "status: {}", err.status());
        assert!(!err.next_action().is_empty());
    }

    #[test]
    fn launch_variables_are_ordered_by_name_whatever_order_they_arrive_in() {
        let a = key(&[
            ("CLAUDE_CONFIG_DIR", Some("/c")),
            ("CODEX_HOME", Some("/x")),
        ]);
        let b = key(&[
            ("CODEX_HOME", Some("/x")),
            ("CLAUDE_CONFIG_DIR", Some("/c")),
        ]);
        assert_eq!(a, b);
        let names: Vec<&str> = a.launch().iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["CLAUDE_CONFIG_DIR", "CODEX_HOME"]);
    }

    #[test]
    fn a_moved_config_directory_is_another_environment() {
        let home = key(&[("CLAUDE_CONFIG_DIR", None)]);
        let moved = key(&[("CLAUDE_CONFIG_DIR", Some("/elsewhere"))]);
        assert_ne!(home, moved);
    }

    #[test]
    fn an_unset_variable_and_an_empty_one_are_different_environments() {
        let unset = key(&[("CLAUDE_CONFIG_DIR", None)]);
        let empty = key(&[("CLAUDE_CONFIG_DIR", Some(""))]);
        assert_ne!(unset, empty);
    }

    #[test]
    fn the_text_form_is_stable_and_names_every_part() {
        let text = key(&[("CLAUDE_CONFIG_DIR", Some("/c")), ("CODEX_HOME", None)]).to_string();
        assert_eq!(
            text,
            "host-1/alex/claude-code//Users/alex/work/repo?CLAUDE_CONFIG_DIR=/c&CODEX_HOME=-"
        );
    }
}
