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
        match self {
            Installer::Brew => "brew",
            Installer::Mise => "mise",
        }
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
    let entry = |name, installer, package| ToolSpec {
        name,
        install: Some(Recorded { installer, package }),
    };
    let unrecorded = |name| ToolSpec {
        name,
        install: None,
    };
    vec![
        entry("codex", Installer::Mise, "codex"),
        entry("goose", Installer::Brew, "block-goose-cli"),
        entry("marvel", Installer::Brew, "arcavenae/tap/marvel"),
        entry("marvel", Installer::Mise, "github:ArcavenAE/marvel"),
        unrecorded("director"),
        entry("sideshow", Installer::Brew, "arcavenae/tap/sideshow"),
        entry("kos", Installer::Brew, "arcavenae/tap/kos"),
        unrecorded("aq"),
        unrecorded("ax"),
        unrecorded("akey"),
        unrecorded("tmux-cmc"),
        entry("mise", Installer::Brew, "mise"),
    ]
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
    /// The fixed brew locations watched as absent paths when `brew` is none.
    pub brew_candidates: Vec<PathBuf>,
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
        // A variable counts only when it holds an absolute path.
        let absolute = |name: &str| {
            get(name)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
        };
        // A home counts only when it is an absolute path: a defaulted path under an empty
        // or relative home would be read from the current directory.
        let home = absolute("HOME");
        let under_home = |tail: &str| {
            home.as_ref()
                .map(|home| home.join(tail))
                .unwrap_or_default()
        };

        let brew = brew_paths(&absolute, &exists, &under_home, macos);
        let brew_candidates = if brew.is_none() {
            BREW_LOCATIONS
                .iter()
                .map(|(repository, _)| PathBuf::from(repository))
                .collect()
        } else {
            Vec::new()
        };

        let mise_data = absolute("MISE_DATA_DIR")
            .or_else(|| absolute("XDG_DATA_HOME").map(|dir| dir.join("mise")))
            .unwrap_or_else(|| under_home(".local/share/mise"));
        let mise_cache = absolute("MISE_CACHE_DIR").unwrap_or_else(|| {
            if macos {
                under_home("Library/Caches/mise")
            } else {
                absolute("XDG_CACHE_HOME")
                    .map(|dir| dir.join("mise"))
                    .unwrap_or_else(|| under_home(".cache/mise"))
            }
        });
        let mise_state = absolute("MISE_STATE_DIR")
            .or_else(|| absolute("XDG_STATE_HOME").map(|dir| dir.join("mise")))
            .unwrap_or_else(|| under_home(".local/state/mise"));
        Self {
            brew,
            brew_candidates,
            mise_data,
            mise_cache,
            mise_state,
        }
    }
}

/// The two fixed Homebrew locations, as repository and prefix, in the order tried.
const BREW_LOCATIONS: [(&str, &str); 2] = [
    ("/opt/homebrew", "/opt/homebrew"),
    ("/usr/local/Homebrew", "/usr/local"),
];

fn brew_paths(
    absolute: &impl Fn(&str) -> Option<PathBuf>,
    exists: &impl Fn(&Path) -> bool,
    under_home: &impl Fn(&str) -> PathBuf,
    macos: bool,
) -> Option<BrewPaths> {
    let (repository, prefix) = match (absolute("HOMEBREW_REPOSITORY"), absolute("HOMEBREW_PREFIX"))
    {
        (Some(repository), Some(prefix)) => (repository, prefix),
        _ => BREW_LOCATIONS
            .iter()
            .find(|(repository, _)| exists(Path::new(repository)))
            .map(|(repository, prefix)| (PathBuf::from(repository), PathBuf::from(prefix)))?,
    };
    let cache = absolute("HOMEBREW_CACHE").unwrap_or_else(|| {
        if macos {
            under_home("Library/Caches/Homebrew")
        } else {
            under_home(".cache/Homebrew")
        }
    });
    Some(BrewPaths {
        repository,
        prefix,
        cache,
    })
}

/// The commands usher must never run for an installer.
pub fn never_call(installer: Installer) -> &'static [&'static str] {
    match installer {
        Installer::Brew => &[
            "update",
            "upgrade",
            "cleanup",
            "autoremove",
            "link",
            "unlink",
            "reinstall",
            "install",
            "uninstall",
            "tap",
            "untap",
            "pin",
            "unpin",
            "postinstall",
            "doctor",
        ],
        Installer::Mise => &[
            "install",
            "upgrade",
            "up",
            "use",
            "prune",
            "reshim",
            "self-update",
            "plugins update",
            "cache clear",
        ],
    }
}

/// The environment every query of an installer carries, so it does not update, clean up,
/// send analytics or reach the network on its own.
pub fn no_update_env(installer: Installer) -> &'static [(&'static str, &'static str)] {
    match installer {
        Installer::Brew => &[
            ("HOMEBREW_NO_AUTO_UPDATE", "1"),
            ("HOMEBREW_NO_INSTALL_CLEANUP", "1"),
            ("HOMEBREW_NO_ANALYTICS", "1"),
            ("HOMEBREW_NO_ENV_HINTS", "1"),
        ],
        Installer::Mise => &[("MISE_OFFLINE", "1")],
    }
}

/// The exact argument lists usher may run. Empty while no installer is run.
pub fn allowed_invocations() -> &'static [&'static [&'static str]] {
    &[]
}

/// The install or update command for a package, as text.
pub fn command_text(installer: Installer, kind: HintKind, package: &str) -> String {
    match (installer, kind) {
        (Installer::Brew, HintKind::Install) => format!("brew install {package}"),
        (Installer::Brew, HintKind::Update) => format!("brew upgrade {package}"),
        (Installer::Mise, HintKind::Install) => format!("mise use --global {package}"),
        (Installer::Mise, HintKind::Update) => format!("mise upgrade {package}"),
    }
}

/// Renders the rows as the `usher tools` text.
pub fn render_rows(rows: &[ToolRow]) -> String {
    let mut lines = Vec::new();
    for row in rows {
        let state = match row.state {
            InstallState::Installed => "installed".to_string(),
            InstallState::Missing => "missing".to_string(),
            InstallState::NoInstaller => "unknown (no installer recorded)".to_string(),
        };
        let verdict = match &row.verdict {
            Verdict::Behind => "behind".to_string(),
            Verdict::Current => "current".to_string(),
            Verdict::Unknown(why) => format!("unknown ({why})"),
            Verdict::Error => "error".to_string(),
        };
        lines.push(format!(
            "{}  {}  {}  {}  {}  {}",
            row.name,
            state,
            row.source,
            reading_text(&row.installed),
            reading_text(&row.current),
            verdict
        ));
        if let Some(error) = &row.error {
            lines.push(format!("    error: {error}"));
        }
        if let Some(hint) = &row.hint {
            let word = match hint.kind {
                HintKind::Install => "install",
                HintKind::Update => "update",
            };
            lines.push(format!(
                "    {word}: {}  (source {})",
                hint.command, hint.source
            ));
        }
    }
    lines.join("\n")
}

fn reading_text(reading: &Reading) -> String {
    match reading {
        Reading::Known(text) => text.clone(),
        Reading::Unknown(why) => format!("unknown ({why})"),
        Reading::None => "none".to_string(),
    }
}

/// Versions joined the way a row prints them: newest first, `, ` between each. The order
/// compares dot-separated parts as numbers where both are digits and as text otherwise; a
/// version that is a prefix of another sorts after it, and text order breaks any tie.
pub fn join_versions(versions: &[String]) -> String {
    let mut sorted: Vec<&String> = versions.iter().collect();
    sorted.sort_by(|a, b| version_order(b, a));
    sorted
        .iter()
        .map(|version| version.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn version_order(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut left = a.split('.');
    let mut right = b.split('.');
    loop {
        match (left.next(), right.next()) {
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let order = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(x), Ok(y)) => x.cmp(&y),
                    _ => x.cmp(y),
                };
                if order != Ordering::Equal {
                    return order;
                }
            }
        }
    }
}
