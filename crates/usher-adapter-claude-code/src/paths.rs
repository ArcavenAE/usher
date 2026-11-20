//! Placing a knowledge path pattern on this host: the five path tokens resolve from
//! the launch context, and a pattern written as absolute is placed under the system
//! root.

use std::path::{Component, Path, PathBuf};

use usher_core::{LaunchContext, SystemRoot};

/// Why a pattern could not be placed.
pub(crate) type Reason = String;

/// Places `pattern`. A pattern that starts with `/` goes under `system_root`; one that
/// starts with a token takes that token's value from `context`.
pub(crate) fn resolve(
    pattern: &str,
    context: &LaunchContext,
    system_root: &SystemRoot,
    config_dir_variable: &str,
) -> Result<PathBuf, Reason> {
    if let Some(absolute) = pattern.strip_prefix('/') {
        let rest = expand(absolute, context, config_dir_variable)?;
        return Ok(system_root.path().join(rest));
    }
    let expanded = expand(pattern, context, config_dir_variable)?;
    Ok(PathBuf::from(expanded))
}

fn expand(
    pattern: &str,
    context: &LaunchContext,
    config_dir_variable: &str,
) -> Result<String, Reason> {
    let mut out = String::new();
    let mut rest = pattern;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let from = &rest[start..];
        let end = from
            .find('}')
            .ok_or_else(|| format!("`{pattern}` has a token that is never closed"))?;
        let value = match &from[..=end] {
            "${HOME}" => path_text(&context.home, "the home directory")?,
            "${CONFIG_DIR}" => {
                if !context.config_dir.is_absolute() {
                    return Err(format!("{config_dir_variable} is not absolute"));
                }
                path_text(&context.config_dir, "the config directory")?
            }
            "${REPO}" => context
                .repo
                .as_deref()
                .ok_or_else(|| "the directory is not in a repository".to_string())
                .and_then(|repo| path_text(repo, "the repository"))?,
            "${LAUNCH_DIR}" => path_text(context.key.directory(), "the launch directory")?,
            "${USER}" => context.user.clone(),
            other => return Err(format!("{other} is not a path token")),
        };
        out.push_str(&value);
        rest = &from[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn path_text(path: &Path, what: &str) -> Result<String, Reason> {
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| format!("{what} is not valid text"))
}

/// Resolves `.` and `..` by text, never by asking the file system.
pub(crate) fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}
