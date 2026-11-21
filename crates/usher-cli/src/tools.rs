//! The `tools` verb (story 2.1): the installed tools, their source and version, read from
//! the installers' own files.
//!
//! Nothing here runs a process. A formula's version comes from the Cellar and the `opt`
//! link, its tap from the install receipt, and a mise tool's versions and source from the
//! `installs` directory and the backend file mise writes beside them.

use std::{
    cmp::Ordering,
    fs, io,
    path::{Path, PathBuf},
};

use usher_core::{
    Hint, HintKind, InstallState, Installer, InstallerPaths, Reading, Recorded, ToolRow, ToolSpec,
    UsherError, Verdict, command_text, join_versions, render_rows,
};

const NOT_RUN: &str = "version check not run";
const UNKNOWN_SOURCE: &str = "unknown source";
const NO_INSTALLER: &str = "no installer recorded";
const RECEIPT: &str = "INSTALL_RECEIPT.json";
const BACKEND_FILE: &str = ".mise.backend.toml";

/// What the verb reads: the tools to look for and where the installers keep their state.
#[derive(Debug, Clone)]
pub struct ToolsContext {
    /// The tools to look for.
    pub specs: Vec<ToolSpec>,
    /// Where the installers keep their state.
    pub paths: InstallerPaths,
}

/// Refuses any argument after the command word, before anything is read.
pub(crate) fn check_tools_args(args: &[String]) -> Result<(), UsherError> {
    if args.len() > 1 {
        let extra: Vec<String> = args[1..]
            .iter()
            .map(|arg| format!("`{}`", arg.escape_debug()))
            .collect();
        return Err(UsherError::new(
            "run tools",
            "the verb takes no arguments",
            format!("usher tools takes no arguments; got {}", extra.join(", ")),
            "run `usher tools`",
        ));
    }
    Ok(())
}

/// Runs the verb over `context`. Its arguments are the command word and nothing else.
pub fn run_tools(args: &[String], context: &ToolsContext) -> Result<String, UsherError> {
    check_tools_args(args)?;
    Ok(render_rows(&read_tools(&context.specs, &context.paths)))
}

fn not_run() -> Reading {
    Reading::Unknown(NOT_RUN.to_string())
}

/// The row for a tool looked for and not found, carrying the owning installer's install
/// command.
fn missing_row(spec: &ToolSpec, recorded: Recorded, why: &str) -> ToolRow {
    let installer = recorded.installer.name();
    ToolRow {
        name: spec.name.to_string(),
        state: InstallState::Missing,
        source: installer.to_string(),
        installed: Reading::None,
        current: not_run(),
        verdict: Verdict::Unknown(why.to_string()),
        error: None,
        hint: Some(Hint {
            kind: HintKind::Install,
            command: command_text(recorded.installer, HintKind::Install, recorded.package),
            source: installer.to_string(),
        }),
    }
}

fn installed_row(spec: &ToolSpec, source: String, versions: &[String]) -> ToolRow {
    ToolRow {
        name: spec.name.to_string(),
        state: InstallState::Installed,
        source,
        installed: Reading::Known(join_versions(versions)),
        current: not_run(),
        verdict: Verdict::Unknown(NOT_RUN.to_string()),
        error: None,
        hint: None,
    }
}

fn error_row(name: &str, source: &str, unreadable: &str, error: String) -> ToolRow {
    ToolRow {
        name: name.to_string(),
        state: InstallState::Installed,
        source: source.to_string(),
        installed: Reading::Unknown(unreadable.to_string()),
        current: not_run(),
        verdict: Verdict::Error,
        error: Some(error),
        hint: None,
    }
}

/// The text of an io error, or `missing` for a file that is not there.
fn io_reason(err: &io::Error) -> String {
    if err.kind() == io::ErrorKind::NotFound {
        "missing".to_string()
    } else {
        err.to_string()
    }
}

/// Orders versions by their numeric parts, so `1.10.0` follows `1.9.0`.
fn version_order(a: &str, b: &str) -> Ordering {
    let parts = |text: &str| -> Vec<String> {
        text.split(['.', '-', '+', '_'])
            .map(str::to_string)
            .collect()
    };
    for (left, right) in parts(a).iter().zip(parts(b).iter()) {
        let order = match (left.parse::<u64>(), right.parse::<u64>()) {
            (Ok(x), Ok(y)) => x.cmp(&y),
            _ => left.cmp(right),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// The names of the real, non-hidden directories directly under `dir`. A missing or
/// unreadable directory has none.
fn subdirectories(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.'))
        .collect();
    names.sort();
    names
}

// ---- brew ----

/// Reads one brew tool from the Cellar, the link directory, the receipts and the taps.
pub fn read_brew(spec: &ToolSpec, paths: &InstallerPaths) -> Vec<ToolRow> {
    let Some(recorded) = spec.install else {
        return Vec::new();
    };
    let Some(brew) = &paths.brew else {
        return vec![missing_row(spec, recorded, "brew not found")];
    };
    let formula = recorded
        .package
        .rsplit('/')
        .next()
        .unwrap_or(recorded.package);
    let cellar = brew.prefix.join("Cellar").join(formula);
    let mut versions = subdirectories(&cellar);
    if versions.is_empty() {
        return vec![missing_row(spec, recorded, NOT_RUN)];
    }

    let taps = taps_holding(&brew.repository, formula);
    if taps.len() > 1 {
        return vec![error_row(
            spec.name,
            UNKNOWN_SOURCE,
            "installed from more than one tap",
            format!(
                "{}: installed from {} taps, {}",
                cellar.display(),
                taps.len(),
                list_with_and(&taps)
            ),
        )];
    }

    let opt = brew.prefix.join("opt").join(formula);
    let in_effect = match fs::read_link(&opt) {
        Ok(target) => target
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| versions.contains(name)),
        Err(err) if err.kind() == io::ErrorKind::NotFound && versions.len() == 1 => {
            Some(versions[0].clone())
        }
        Err(_) => None,
    };
    let Some(in_effect) = in_effect else {
        return vec![error_row(
            spec.name,
            UNKNOWN_SOURCE,
            "the version in effect is not known",
            format!("{}: does not name an installed version", opt.display()),
        )];
    };

    let receipt = cellar.join(&in_effect).join(RECEIPT);
    let tap = match read_receipt_tap(&receipt) {
        Ok(tap) => tap,
        Err(reason) => {
            return vec![error_row(
                spec.name,
                UNKNOWN_SOURCE,
                "receipt unreadable",
                format!("{}: {reason}", receipt.display()),
            )];
        }
    };

    versions.retain(|version| *version != in_effect);
    versions.sort_by(|a, b| version_order(b, a));
    versions.insert(0, in_effect);
    vec![installed_row(spec, format!("brew {tap}"), &versions)]
}

/// The tap named in an install receipt, or why it cannot be read.
fn read_receipt_tap(receipt: &Path) -> Result<String, String> {
    let text = fs::read_to_string(receipt).map_err(|err| io_reason(&err))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|_| "not valid JSON".to_string())?;
    match value.get("source").and_then(|source| source.get("tap")) {
        None => Err("source.tap is missing".to_string()),
        Some(tap) => tap
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "source.tap is not a string".to_string()),
    }
}

/// The taps, as `org/name`, that carry a formula file for `formula`, in name order.
fn taps_holding(repository: &Path, formula: &str) -> Vec<String> {
    let taps_dir = repository.join("Library/Taps");
    let mut found = Vec::new();
    for org in subdirectories(&taps_dir) {
        for repo in subdirectories(&taps_dir.join(&org)) {
            let Some(name) = repo.strip_prefix("homebrew-") else {
                continue;
            };
            let formulae = taps_dir.join(&org).join(&repo).join("Formula");
            let file = format!("{formula}.rb");
            let shard = formula.chars().next().map(String::from).unwrap_or_default();
            if formulae.join(&file).is_file() || formulae.join(shard).join(&file).is_file() {
                found.push(format!("{org}/{name}"));
            }
        }
    }
    found.sort();
    found
}

/// `a`, `a and b`, `a, b and c`.
fn list_with_and(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [only] => only.clone(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

// ---- mise ----

/// One directory under mise's `installs`, with what its backend file says.
struct MiseDir {
    dir: String,
    file: PathBuf,
    backend: Result<Backend, String>,
    versions: Vec<String>,
}

struct Backend {
    short: String,
    full: String,
}

fn scan_mise(paths: &InstallerPaths) -> Vec<MiseDir> {
    let installs = paths.mise_data.join("installs");
    subdirectories(&installs)
        .into_iter()
        .map(|dir| {
            let root = installs.join(&dir);
            let file = root.join(BACKEND_FILE);
            let mut versions: Vec<String> = fs::read_dir(&root)
                .map(|entries| {
                    entries
                        .flatten()
                        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
                        .map(|entry| entry.file_name().to_string_lossy().into_owned())
                        .filter(|name| !name.starts_with('.'))
                        .collect()
                })
                .unwrap_or_default();
            versions.sort_by(|a, b| version_order(a, b));
            let backend = read_backend(&file);
            MiseDir {
                dir,
                file,
                backend,
                versions,
            }
        })
        .collect()
}

fn read_backend(file: &Path) -> Result<Backend, String> {
    let text = fs::read_to_string(file).map_err(|err| io_reason(&err))?;
    let doc: toml_edit::DocumentMut = text.parse().map_err(|_| "not valid TOML".to_string())?;
    let field = |name: &str| -> Result<String, String> {
        match doc.get(name) {
            None => Err(format!("{name} is missing")),
            Some(item) => item
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{name} is not a string")),
        }
    };
    Ok(Backend {
        short: field("short")?,
        full: field("full")?,
    })
}

/// Reads one mise tool from the `installs` directory.
pub fn read_mise(spec: &ToolSpec, paths: &InstallerPaths) -> Vec<ToolRow> {
    let Some(recorded) = spec.install else {
        return Vec::new();
    };
    let rows: Vec<ToolRow> = scan_mise(paths)
        .into_iter()
        .filter_map(|found| {
            let backend = found.backend.ok()?;
            if backend.short != recorded.package || found.versions.is_empty() {
                return None;
            }
            let word = backend
                .full
                .split_once(':')
                .map_or("core", |(word, _)| word);
            Some(installed_row(spec, format!("mise {word}"), &found.versions))
        })
        .collect();
    if rows.is_empty() {
        vec![missing_row(spec, recorded, NOT_RUN)]
    } else {
        rows
    }
}

/// The error rows for directories whose backend file cannot be read, once each, in
/// directory name order.
fn backend_error_rows(paths: &InstallerPaths) -> Vec<ToolRow> {
    scan_mise(paths)
        .into_iter()
        .filter_map(|found| {
            let reason = found.backend.err()?;
            Some(error_row(
                &found.dir,
                "mise",
                "backend file unreadable",
                format!("{}: {reason}", found.file.display()),
            ))
        })
        .collect()
}

// ---- all tools ----

fn no_installer_row(spec: &ToolSpec) -> ToolRow {
    ToolRow {
        name: spec.name.to_string(),
        state: InstallState::NoInstaller,
        source: UNKNOWN_SOURCE.to_string(),
        installed: Reading::Unknown(NO_INSTALLER.to_string()),
        current: not_run(),
        verdict: Verdict::Unknown(NO_INSTALLER.to_string()),
        error: None,
        hint: None,
    }
}

/// Reads every tool from the installer that owns it, one row per installed entry of a
/// name, or one row for a name with none installed.
pub fn read_tools(specs: &[ToolSpec], paths: &InstallerPaths) -> Vec<ToolRow> {
    let mut names: Vec<&str> = Vec::new();
    for spec in specs {
        if !names.contains(&spec.name) {
            names.push(spec.name);
        }
    }
    let mut rows = Vec::new();
    for name in names {
        let mut found = Vec::new();
        let mut first_missing: Option<ToolRow> = None;
        for spec in specs.iter().filter(|spec| spec.name == name) {
            let Some(recorded) = spec.install else {
                found.push(no_installer_row(spec));
                continue;
            };
            let read = match recorded.installer {
                Installer::Brew => read_brew(spec, paths),
                Installer::Mise => read_mise(spec, paths),
            };
            for row in read {
                if row.state == InstallState::Missing {
                    first_missing.get_or_insert(row);
                } else {
                    found.push(row);
                }
            }
        }
        if found.is_empty() {
            found.extend(first_missing);
        }
        rows.extend(found);
    }
    rows.extend(backend_error_rows(paths));
    rows
}
