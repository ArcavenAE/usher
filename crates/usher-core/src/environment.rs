//! The environment key (AD-17): host, OS user, harness, absolute directory and the
//! launch-context variables harness knowledge names for that harness.
//!
//! One function builds the key and every mode uses it, so one environment is never
//! keyed two ways.

use std::path::{Component, Path, PathBuf};

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
///
/// The directory must be absolute and free of `..`; `.`, empty and trailing segments
/// are dropped, so one directory spelled several ways is one key. The variables are
/// ordered by name, and a name given twice is refused.
pub fn environment_key(
    host: &str,
    os_user: &str,
    harness: &str,
    directory: &Path,
    launch: &[(&str, Option<&str>)],
) -> Result<EnvironmentKey, UsherError> {
    let directory = normalize_directory(directory)?;
    let mut launch: Vec<(String, Option<String>)> = launch
        .iter()
        .map(|(name, value)| ((*name).to_string(), value.map(str::to_string)))
        .collect();
    launch.sort();
    if let Some(pair) = launch.windows(2).find(|pair| pair[0].0 == pair[1].0) {
        return Err(key_error(
            "the same variable is named twice",
            format!("{} has two values", pair[0].0),
            "pass each launch variable once",
        ));
    }
    Ok(EnvironmentKey {
        host: host.to_string(),
        os_user: os_user.to_string(),
        harness: harness.to_string(),
        directory,
        launch,
    })
}

fn key_error(check: &str, status: String, next_action: &str) -> UsherError {
    UsherError::new("key an environment", check, status, next_action)
}

fn normalize_directory(directory: &Path) -> Result<PathBuf, UsherError> {
    if !directory.is_absolute() {
        return Err(key_error(
            "the directory must be absolute",
            format!("{} is relative", directory.display()),
            "pass the absolute path of the launch directory",
        ));
    }
    let mut out = PathBuf::from("/");
    for component in directory.components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::ParentDir => {
                return Err(key_error(
                    "the directory must not hold a `..` segment",
                    format!("{} holds `..`", directory.display()),
                    "resolve the directory, symlinks included, before keying it",
                ));
            }
            _ => {}
        }
    }
    Ok(out)
}

/// Percent-escapes the listed characters, and `%` itself, so a field never reads as
/// a separator.
fn escape(text: &str, also: &[char]) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch == '%' || also.contains(&ch) {
            out.push_str(&format!("%{:02X}", ch as u32));
        } else {
            out.push(ch);
        }
    }
    out
}

/// The variables that belong in an environment key: those named, set and non-empty, in the
/// order named. The one place that decides it, so no caller drops differently;
/// [`environment_key`] itself filters nothing.
pub fn key_variables(
    names: &[&str],
    lookup: impl Fn(&str) -> Option<String>,
) -> Vec<(String, String)> {
    let _ = (names, lookup);
    Vec::new()
}

impl std::fmt::Display for EnvironmentKey {
    /// The text form the store keys on: `host/user/harness/<directory>?NAME=value&NAME`.
    /// An unset variable is the bare name, an empty one is `NAME=`, and every
    /// separator inside a field is percent-escaped, so unequal keys never share a form.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            escape(&self.host, &['/']),
            escape(&self.os_user, &['/']),
            escape(&self.harness, &['/']),
            escape(&self.directory.to_string_lossy(), &['?', '&', '='])
        )?;
        for (index, (name, value)) in self.launch.iter().enumerate() {
            f.write_str(if index == 0 { "?" } else { "&" })?;
            f.write_str(&escape(name, &['?', '&', '=']))?;
            if let Some(value) = value {
                write!(f, "={}", escape(value, &['&', '=']))?;
            }
        }
        Ok(())
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
            "host-1/alex/claude-code//Users/alex/work/repo?CLAUDE_CONFIG_DIR=/c&CODEX_HOME"
        );
    }

    /// The text form is what the store keys on, so two unequal keys must never share it.
    #[test]
    fn unequal_keys_never_share_a_text_form() {
        let unset = key(&[("CODEX_HOME", None)]);
        let dash = key(&[("CODEX_HOME", Some("-"))]);
        let empty = key(&[("CODEX_HOME", Some(""))]);
        assert_ne!(unset, dash);
        assert_ne!(unset.to_string(), dash.to_string(), "unset against a dash");
        assert_ne!(unset.to_string(), empty.to_string(), "unset against empty");
        assert_ne!(empty.to_string(), dash.to_string(), "empty against a dash");

        // The payload sits on the name that sorts first, so an unescaped text form
        // is `?CLAUDE_CONFIG_DIR=/c&CODEX_HOME=/x` for both and they collide.
        let one = key(&[("CLAUDE_CONFIG_DIR", Some("/c&CODEX_HOME=/x"))]);
        let two = key(&[
            ("CLAUDE_CONFIG_DIR", Some("/c")),
            ("CODEX_HOME", Some("/x")),
        ]);
        assert_ne!(one, two);
        assert_ne!(
            one.to_string(),
            two.to_string(),
            "a value holding & and = against two variables"
        );
        let percent = key(&[("CODEX_HOME", Some("%26"))]);
        let ampersand = key(&[("CODEX_HOME", Some("&"))]);
        assert_ne!(
            percent.to_string(),
            ampersand.to_string(),
            "an escape sequence against the character it escapes"
        );
    }

    /// A separator inside the directory, host or user must not read as a boundary.
    #[test]
    fn a_separator_inside_a_field_never_collides_with_a_boundary() {
        let dir = |dir: &str, launch: &[(&str, Option<&str>)]| {
            environment_key("h", "u", "claude-code", Path::new(dir), launch).expect("absolute")
        };
        let question = dir("/Users/alex/work/repo?CODEX_HOME", &[]);
        let unset = dir("/Users/alex/work/repo", &[("CODEX_HOME", None)]);
        assert_ne!(question, unset);
        assert_ne!(
            question.to_string(),
            unset.to_string(),
            "a ? in the directory against an unset variable"
        );

        let ampersand = dir("/a?X=1&Y", &[]);
        let two = dir("/a", &[("X", Some("1")), ("Y", None)]);
        assert_ne!(
            ampersand.to_string(),
            two.to_string(),
            "a ?, = and & in the directory against two variables"
        );

        let host = |host: &str, user: &str| {
            environment_key(host, user, "claude-code", Path::new("/a"), &[]).expect("absolute")
        };
        assert_ne!(
            host("a/b", "c").to_string(),
            host("a", "b/c").to_string(),
            "a / in the host against a / in the user"
        );
    }

    /// AD-17 prevents one environment keyed several ways.
    #[test]
    fn one_directory_spelled_two_ways_is_one_key_and_one_text_form() {
        let spell = |dir: &str| {
            environment_key("h", "u", "claude-code", Path::new(dir), &[]).expect("absolute")
        };
        let plain = spell("/a/b");
        for other in ["/a/b/", "/a/./b", "/a//b", "/a/b/."] {
            let other = spell(other);
            assert_eq!(plain, other, "{other}");
            assert_eq!(plain.to_string(), other.to_string(), "{other}");
        }
        assert_eq!(plain.directory(), Path::new("/a/b"));
    }

    /// A `..` segment names a parent only once symlinks are resolved, which is the
    /// caller's step, so the key refuses it instead of guessing.
    #[test]
    fn a_directory_with_a_parent_segment_is_refused() {
        let err = environment_key("h", "u", "claude-code", Path::new("/a/../b"), &[])
            .expect_err("`..` is resolved by the caller, not guessed here");
        assert_eq!(err.step(), "key an environment");
        assert!(err.status().contains(".."), "status: {}", err.status());
    }

    /// The caller passes the variables harness knowledge names for the harness; the
    /// key holds exactly those, and does not filter further.
    #[test]
    fn the_key_holds_exactly_the_variables_it_is_given() {
        let key = key(&[("CLAUDE_CONFIG_DIR", Some("/c")), ("PATH", Some("/bin"))]);
        let names: Vec<&str> = key.launch().iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["CLAUDE_CONFIG_DIR", "PATH"]);
    }

    #[test]
    fn a_variable_named_twice_is_refused() {
        let err = environment_key(
            "h",
            "u",
            "claude-code",
            Path::new("/a"),
            &[("CODEX_HOME", Some("/x")), ("CODEX_HOME", None)],
        )
        .expect_err("two values for one name have no one meaning");
        assert_eq!(err.step(), "key an environment");
        assert!(
            err.status().contains("CODEX_HOME"),
            "status: {}",
            err.status()
        );
    }

    /// One function decides which variables enter a key: the named ones that are set and
    /// non-empty. A knowledge update that names a new variable then changes no key where
    /// that variable is unset.
    #[test]
    fn key_variables_keeps_the_named_set_and_non_empty_ones_in_order() {
        let lookup = |name: &str| match name {
            "A_SET" => Some("1".to_string()),
            "C_SET" => Some("3".to_string()),
            "D_EMPTY" => Some(String::new()),
            "NOT_NAMED" => Some("x".to_string()),
            _ => None,
        };
        let kept = key_variables(&["A_SET", "B_UNSET", "C_SET", "D_EMPTY"], lookup);
        assert_eq!(
            kept,
            [
                ("A_SET".to_string(), "1".to_string()),
                ("C_SET".to_string(), "3".to_string())
            ]
        );
    }

    #[test]
    fn key_variables_asks_only_for_the_names_it_is_given() {
        use std::cell::RefCell;
        let asked = RefCell::new(Vec::new());
        let kept = key_variables(&["ONE", "TWO"], |name| {
            asked.borrow_mut().push(name.to_string());
            Some("v".to_string())
        });
        assert_eq!(asked.into_inner(), ["ONE", "TWO"]);
        assert_eq!(kept.len(), 2);
    }
}
