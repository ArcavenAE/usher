//! Installed tools (story 2.1): the row model, the installer tables and the paths the
//! readers in `usher-cli` use.
//!
//! Every answer here comes from files an installer already wrote. No installer is run:
//! a plain `mise` call makes its own network check, and a `brew` call that reads the
//! formula index can fetch it, so the first version of the story spawns nothing and the
//! behind-check waits for the egress port's client.

use std::path::{Path, PathBuf};

/// An installer usher reads the state of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Installer {
    /// Homebrew.
    Brew,
    /// mise.
    Mise,
}

impl Installer {
    /// The installer's name as it appears in a source column and in command text.
    pub fn name(self) -> &'static str {
        let _ = self;
        ""
    }
}

/// An installer and the package name it knows a tool by, as recorded for one tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recorded {
    /// The installer that owns the tool.
    pub installer: Installer,
    /// The package name the installer knows it by.
    pub package: &'static str,
}

/// One tool to look for: its name and, where one is recorded, its installer. An entry
/// with no recorded installer is never looked up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolSpec {
    /// The name shown to the person.
    pub name: &'static str,
    /// The recorded install, if any.
    pub install: Option<Recorded>,
}

/// The tools usher looks for. One function, so where the list lives can change without
/// touching the readers.
pub fn tool_specs() -> Vec<ToolSpec> {
    Vec::new()
}

/// Whether a tool is installed by the source a row names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallState {
    /// Found.
    Installed,
    /// Looked for and not found.
    Missing,
    /// Not looked for, because no installer is recorded.
    NoInstaller,
}

/// A version as read, or why it is not known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reading {
    /// The version text.
    Known(String),
    /// Not known, and why.
    Unknown(String),
    /// Looked for and found nothing.
    None,
}

/// Whether a tool is behind its source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Behind its source.
    Behind,
    /// At its source's current version.
    Current,
    /// Not known, and why.
    Unknown(String),
    /// A read failed.
    Error,
}

/// Which command a row hints at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintKind {
    /// How to install a missing tool.
    Install,
    /// How to update a behind tool.
    Update,
}

/// The owning installer's exact command and its source, printed and never run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    /// Install or update.
    pub kind: HintKind,
    /// The command text.
    pub command: String,
    /// The source it comes from.
    pub source: String,
}

/// One row of `usher tools`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRow {
    /// The tool's name.
    pub name: String,
    /// Installed or missing.
    pub state: InstallState,
    /// The source, in the words the story fixes.
    pub source: String,
    /// The installed version.
    pub installed: Reading,
    /// The source's current version.
    pub current: Reading,
    /// Behind, current, unknown or error.
    pub verdict: Verdict,
    /// For an error row: the failing step and its detail.
    pub error: Option<String>,
    /// The command to print under a missing or behind row.
    pub hint: Option<Hint>,
}

/// Where Homebrew keeps its state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewPaths {
    /// The Homebrew repository.
    pub repository: PathBuf,
    /// The Homebrew prefix (holds `Cellar` and `opt`).
    pub prefix: PathBuf,
    /// The Homebrew cache.
    pub cache: PathBuf,
}

/// Where each installer keeps its state, found without running the installer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerPaths {
    /// Homebrew's paths, or none when brew is not found.
    pub brew: Option<BrewPaths>,
    /// The mise data directory (holds `installs` and `shims`).
    pub mise_data: PathBuf,
    /// The mise cache directory.
    pub mise_cache: PathBuf,
    /// The mise state directory.
    pub mise_state: PathBuf,
}

impl InstallerPaths {
    /// Resolves the paths from `get` (environment lookup) and `exists` (a file test), on
    /// macOS when `macos` is set.
    pub fn resolve(
        get: impl Fn(&str) -> Option<String>,
        exists: impl Fn(&Path) -> bool,
        macos: bool,
    ) -> Self {
        let _ = (get, exists, macos);
        Self {
            brew: Some(BrewPaths {
                repository: PathBuf::new(),
                prefix: PathBuf::new(),
                cache: PathBuf::new(),
            }),
            mise_data: PathBuf::new(),
            mise_cache: PathBuf::new(),
            mise_state: PathBuf::new(),
        }
    }
}

/// The commands usher must never run for an installer.
pub fn never_call(installer: Installer) -> &'static [&'static str] {
    let _ = installer;
    &[]
}

/// The environment every query of an installer carries, so it does not update, clean up,
/// send analytics or reach the network on its own.
pub fn no_update_env(installer: Installer) -> &'static [(&'static str, &'static str)] {
    let _ = installer;
    &[]
}

/// The exact argument lists usher may run. Empty while no installer is run.
pub fn allowed_invocations() -> &'static [&'static [&'static str]] {
    &[]
}

/// The install or update command for a package, as text.
pub fn command_text(installer: Installer, kind: HintKind, package: &str) -> String {
    let _ = (installer, kind, package);
    String::new()
}

/// Renders the rows as the `usher tools` text.
pub fn render_rows(rows: &[ToolRow]) -> String {
    let _ = rows;
    String::new()
}
