//! Story 2.1, PR 1: `usher tools` reads brew and mise state from their files and runs no
//! installer. Every fixture lives in a scratch home.

use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
};

use usher_cli::{
    InventoryEnv, LaunchEnvironment, ToolsContext, read_brew, read_mise, read_tools, run, run_tools,
};
use usher_core::{
    BrewPaths, Hint, HintKind, InstallState, Installer, InstallerPaths, Reading, Recorded, ToolRow,
    ToolSpec, Verdict, render_rows, tool_specs,
};
use usher_knowledge::Platform;
use usher_testkit::{InstallerGuard, NamedHome, RealHomes, ScratchHome, observe};

const NOT_RUN: &str = "version check not run";

struct Rig {
    scratch: ScratchHome,
    paths: InstallerPaths,
}

impl Rig {
    fn new() -> Self {
        let scratch = ScratchHome::new().expect("scratch");
        let root = scratch.root().to_path_buf();
        let paths = InstallerPaths {
            brew: Some(BrewPaths {
                repository: root.join("brew"),
                prefix: root.join("brew"),
                cache: root.join("brew-cache"),
            }),
            brew_candidates: vec![],
            mise_data: root.join("mise/data"),
            mise_cache: root.join("mise/cache"),
            mise_state: root.join("mise/state"),
        };
        fs::create_dir_all(&paths.brew.as_ref().expect("brew").prefix).expect("brew prefix");
        fs::create_dir_all(&paths.mise_data).expect("mise data");
        Rig { scratch, paths }
    }

    fn brew(&self) -> &BrewPaths {
        self.paths.brew.as_ref().expect("brew paths")
    }

    fn cellar(&self, formula: &str, version: &str) -> PathBuf {
        let dir = self
            .brew()
            .prefix
            .join("Cellar")
            .join(formula)
            .join(version);
        fs::create_dir_all(&dir).expect("cellar dir");
        dir
    }

    fn receipt(&self, formula: &str, version: &str, text: &str) -> PathBuf {
        let path = self.cellar(formula, version).join("INSTALL_RECEIPT.json");
        fs::write(&path, text).expect("receipt");
        path
    }

    fn good_receipt(&self, formula: &str, version: &str) -> PathBuf {
        self.receipt(
            formula,
            version,
            r#"{"installed_on_request":true,"runtime_dependencies":[{"full_name":"oniguruma","version":"6.9.9"}],"source":{"tap":"homebrew/core","path":"/x","versions":{"stable":"9"}}}"#,
        )
    }

    fn receipt_with_tap(&self, formula: &str, version: &str, tap: &str) -> PathBuf {
        self.receipt(
            formula,
            version,
            &format!(r#"{{"source":{{"tap":"{tap}"}}}}"#),
        )
    }

    /// The `opt` link brew keeps for linked and keg-only formulae alike.
    fn link(&self, formula: &str, version: &str) {
        let dir = self.brew().prefix.join("opt");
        fs::create_dir_all(&dir).expect("opt dir");
        symlink(format!("../Cellar/{formula}/{version}"), dir.join(formula)).expect("opt link");
    }

    /// An entry in `var/homebrew/linked`, which the reader does not use.
    fn linked_entry(&self, formula: &str, version: &str) {
        let dir = self.brew().prefix.join("var/homebrew/linked");
        fs::create_dir_all(&dir).expect("linked dir");
        symlink(
            format!("../../../Cellar/{formula}/{version}"),
            dir.join(formula),
        )
        .expect("linked entry");
    }

    fn tap_formula(&self, tap: &str, sharded: bool, formula: &str) {
        let (org, name) = tap.split_once('/').expect("org/name");
        let mut dir = self
            .brew()
            .repository
            .join("Library/Taps")
            .join(org)
            .join(format!("homebrew-{name}"))
            .join("Formula");
        if sharded {
            dir = dir.join(&formula[..1]);
        }
        fs::create_dir_all(&dir).expect("tap dir");
        fs::write(
            dir.join(format!("{formula}.rb")),
            "class X < Formula\nend\n",
        )
        .expect("formula");
    }

    /// An install directory with the backend file mise writes, and one version.
    fn mise_tool(&self, dir: &str, short: &str, full: &str, version: &str) {
        self.mise_version(dir, version);
        fs::write(
            self.paths
                .mise_data
                .join("installs")
                .join(dir)
                .join(".mise.backend.toml"),
            format!("short = \"{short}\"\nfull = \"{full}\"\n"),
        )
        .expect("backend file");
    }

    fn mise_version(&self, dir: &str, version: &str) {
        fs::create_dir_all(
            self.paths
                .mise_data
                .join("installs")
                .join(dir)
                .join(version),
        )
        .expect("mise install");
    }
}

fn spec(name: &'static str, installer: Installer, package: &'static str) -> ToolSpec {
    ToolSpec {
        name,
        install: Some(Recorded { installer, package }),
    }
}

fn not_run() -> Reading {
    Reading::Unknown(NOT_RUN.to_string())
}

fn only(mut rows: Vec<ToolRow>) -> ToolRow {
    assert_eq!(rows.len(), 1, "one row expected: {rows:?}");
    rows.remove(0)
}

// ---- brew ----

fn brew_spec(name: &'static str, package: &'static str) -> ToolSpec {
    spec(name, Installer::Brew, package)
}

fn mise_spec(name: &'static str, package: &'static str) -> ToolSpec {
    spec(name, Installer::Mise, package)
}

#[test]
fn an_installed_formula_reads_its_version_from_the_cellar_and_its_tap_from_the_receipt() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(
        row,
        ToolRow {
            name: "jq".to_string(),
            state: InstallState::Installed,
            source: "brew homebrew/core".to_string(),
            installed: Reading::Known("1.7.1".to_string()),
            current: not_run(),
            verdict: Verdict::Unknown(NOT_RUN.to_string()),
            error: None,
            hint: None,
        }
    );
}

#[test]
fn a_formula_with_no_cellar_directory_is_missing_with_the_install_command() {
    let rig = Rig::new();
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.state, InstallState::Missing);
    assert_eq!(row.source, "brew");
    assert_eq!(row.installed, Reading::None);
    assert_eq!(row.verdict, Verdict::Unknown(NOT_RUN.to_string()));
    assert_eq!(
        row.hint,
        Some(Hint {
            kind: HintKind::Install,
            command: "brew install jq".to_string(),
            source: "brew".to_string(),
        })
    );
}

#[test]
fn a_missing_tap_formula_prints_the_tap_qualified_install_command() {
    let rig = Rig::new();
    let row = only(read_brew(
        &brew_spec("marvel", "arcavenae/tap/marvel"),
        &rig.paths,
    ));
    assert_eq!(row.state, InstallState::Missing);
    assert_eq!(
        row.hint,
        Some(Hint {
            kind: HintKind::Install,
            command: "brew install arcavenae/tap/marvel".to_string(),
            source: "brew".to_string(),
        })
    );
}

#[test]
fn with_brew_not_found_every_brew_entry_is_a_missing_row_with_the_reason() {
    let mut rig = Rig::new();
    rig.paths.brew = None;
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.state, InstallState::Missing);
    assert_eq!(row.source, "brew");
    assert_eq!(row.installed, Reading::None);
    assert_eq!(row.verdict, Verdict::Unknown("brew not found".to_string()));
    assert_eq!(
        row.hint,
        Some(Hint {
            kind: HintKind::Install,
            command: "brew install jq".to_string(),
            source: "brew".to_string(),
        })
    );
}

#[test]
fn a_receipt_that_is_not_json_is_an_error_row_naming_the_file_never_current() {
    let rig = Rig::new();
    let path = rig.receipt("jq", "1.7.1", "{ not json");
    rig.link("jq", "1.7.1");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    assert_eq!(
        row.error,
        Some(format!("{}: not valid JSON", path.display()))
    );
    assert_ne!(row.verdict, Verdict::Current);
}

#[test]
fn a_receipt_without_a_source_tap_is_an_error_row_that_does_not_guess_a_source() {
    let rig = Rig::new();
    let path = rig.receipt("jq", "1.7.1", r#"{"source":{"path":"/x"}}"#);
    rig.link("jq", "1.7.1");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    assert_eq!(
        row.error,
        Some(format!("{}: source.tap is missing", path.display()))
    );
    assert_eq!(row.source, "unknown source");
}

#[test]
fn a_receipt_whose_source_tap_is_not_a_string_is_an_error_row() {
    let rig = Rig::new();
    let path = rig.receipt("jq", "1.7.1", r#"{"source":{"tap":null}}"#);
    rig.link("jq", "1.7.1");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    assert_eq!(
        row.error,
        Some(format!("{}: source.tap is not a string", path.display()))
    );
}

#[test]
fn a_cellar_version_with_no_receipt_is_an_installed_error_row_naming_the_missing_file() {
    let rig = Rig::new();
    let dir = rig.cellar("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.state, InstallState::Installed);
    assert_eq!(row.verdict, Verdict::Error);
    assert_eq!(
        row.error,
        Some(format!(
            "{}: missing",
            dir.join("INSTALL_RECEIPT.json").display()
        ))
    );
}

#[test]
fn an_unreadable_receipt_is_an_error_row_carrying_the_io_error() {
    let rig = Rig::new();
    let path = rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("chmod");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    let error = row.error.expect("an error row says why");
    assert!(
        error.starts_with(&format!("{}: ", path.display())),
        "{error}"
    );
    assert!(error.contains("Permission denied"), "{error}");
}

#[test]
fn with_two_versions_installed_the_opt_link_names_the_one_in_effect_and_the_others_follow() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.0");
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.installed, Reading::Known("1.7.1, 1.7.0".to_string()));
    assert_eq!(row.verdict, Verdict::Unknown(NOT_RUN.to_string()));
}

#[test]
fn a_keg_only_formula_with_an_opt_link_and_no_linked_entry_reads_its_version() {
    let rig = Rig::new();
    rig.good_receipt("openssl@3", "3.6.5");
    rig.link("openssl@3", "3.6.5");
    assert!(!rig.brew().prefix.join("var/homebrew/linked").exists());
    let row = only(read_brew(&brew_spec("openssl@3", "openssl@3"), &rig.paths));
    assert_eq!(row.installed, Reading::Known("3.6.5".to_string()));
    assert_ne!(row.verdict, Verdict::Error);
}

#[test]
fn one_installed_version_and_no_opt_link_reads_that_version() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.1");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.installed, Reading::Known("1.7.1".to_string()));
    assert_ne!(row.verdict, Verdict::Error);
}

#[test]
fn several_versions_and_no_opt_link_is_an_error_even_when_a_linked_entry_exists() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.0");
    rig.good_receipt("jq", "1.7.1");
    rig.linked_entry("jq", "1.7.0");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    assert_eq!(
        row.error,
        Some(format!(
            "{}: does not name an installed version",
            rig.brew().prefix.join("opt/jq").display()
        ))
    );
}

#[test]
fn an_opt_link_whose_target_is_not_an_installed_version_is_an_error_row() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "9.9.9");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    assert_eq!(
        row.error,
        Some(format!(
            "{}: does not name an installed version",
            rig.brew().prefix.join("opt/jq").display()
        ))
    );
}

#[test]
fn a_formula_in_two_taps_is_an_error_row_with_the_whole_ruled_line() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    rig.tap_formula("acme/tools", false, "jq");
    rig.tap_formula("other/extras", true, "jq");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    assert_eq!(
        row.error,
        Some(format!(
            "{}: installed from 2 taps, acme/tools and other/extras",
            rig.brew().prefix.join("Cellar/jq").display()
        ))
    );
}

#[test]
fn a_formula_in_three_taps_lists_them_joined_by_commas_and_an_and_before_the_last() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    for tap in ["acme/a", "acme/b", "acme/c"] {
        rig.tap_formula(tap, false, "jq");
    }
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(
        row.error,
        Some(format!(
            "{}: installed from 3 taps, acme/a, acme/b and acme/c",
            rig.brew().prefix.join("Cellar/jq").display()
        ))
    );
}

#[test]
fn a_formula_in_one_tap_is_not_an_error() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    rig.tap_formula("acme/tools", false, "jq");
    let row = only(read_brew(&brew_spec("jq", "jq"), &rig.paths));
    assert_eq!(row.state, InstallState::Installed);
    assert_ne!(row.verdict, Verdict::Error);
}

#[test]
fn a_tap_qualified_package_reads_the_cellar_directory_after_the_last_slash() {
    let rig = Rig::new();
    rig.receipt_with_tap("marvel", "0.9.0", "arcavenae/tap");
    rig.link("marvel", "0.9.0");
    let row = only(read_brew(
        &brew_spec("marvel", "arcavenae/tap/marvel"),
        &rig.paths,
    ));
    assert_eq!(row.state, InstallState::Installed);
    assert_eq!(row.source, "brew arcavenae/tap");
    assert_eq!(row.installed, Reading::Known("0.9.0".to_string()));
    assert_eq!(row.verdict, Verdict::Unknown(NOT_RUN.to_string()));
}

#[test]
fn a_receipt_naming_another_tap_is_an_installed_row_with_that_tap_as_its_source() {
    let rig = Rig::new();
    rig.receipt_with_tap("marvel", "0.9.0", "other/tap");
    rig.link("marvel", "0.9.0");
    let row = only(read_brew(
        &brew_spec("marvel", "arcavenae/tap/marvel"),
        &rig.paths,
    ));
    assert_eq!(row.state, InstallState::Installed);
    assert_eq!(row.source, "brew other/tap");
    assert_ne!(row.verdict, Verdict::Error);
}

// ---- mise ----

#[test]
fn a_mise_tool_reads_its_versions_from_the_installs_directory_names() {
    let rig = Rig::new();
    rig.mise_tool("go", "go", "core:go", "1.23.0");
    let row = only(read_mise(&mise_spec("go", "go"), &rig.paths));
    assert_eq!(
        row,
        ToolRow {
            name: "go".to_string(),
            state: InstallState::Installed,
            source: "mise core".to_string(),
            installed: Reading::Known("1.23.0".to_string()),
            current: not_run(),
            verdict: Verdict::Unknown(NOT_RUN.to_string()),
            error: None,
            hint: None,
        }
    );
}

#[test]
fn several_mise_versions_print_sorted_and_joined() {
    let rig = Rig::new();
    rig.mise_tool("go", "go", "core:go", "1.23.0");
    rig.mise_version("go", "1.22.0");
    let row = only(read_mise(&mise_spec("go", "go"), &rig.paths));
    assert_eq!(row.installed, Reading::Known("1.22.0, 1.23.0".to_string()));
}

#[test]
fn a_symlink_in_the_installs_directory_is_not_a_version() {
    let rig = Rig::new();
    rig.mise_tool("go", "go", "core:go", "1.23.0");
    symlink("1.23.0", rig.paths.mise_data.join("installs/go/latest")).expect("alias");
    let row = only(read_mise(&mise_spec("go", "go"), &rig.paths));
    assert_eq!(row.installed, Reading::Known("1.23.0".to_string()));
}

#[test]
fn a_dot_entry_in_the_installs_directory_is_not_a_version() {
    let rig = Rig::new();
    rig.mise_tool("go", "go", "core:go", "1.23.0");
    rig.mise_version("go", ".partial");
    let row = only(read_mise(&mise_spec("go", "go"), &rig.paths));
    assert_eq!(row.installed, Reading::Known("1.23.0".to_string()));
}

#[test]
fn a_backend_package_is_matched_by_the_short_name_in_its_backend_file_not_by_its_directory_name() {
    let rig = Rig::new();
    rig.mise_tool(
        "github-arcaven-ae-marvel",
        "github:ArcavenAE/marvel",
        "github:ArcavenAE/marvel",
        "2.0.1",
    );
    let row = only(read_mise(
        &mise_spec("marvel", "github:ArcavenAE/marvel"),
        &rig.paths,
    ));
    assert_eq!(row.state, InstallState::Installed);
    assert_eq!(row.source, "mise github");
    assert_eq!(row.installed, Reading::Known("2.0.1".to_string()));
}

#[test]
fn the_source_word_is_the_text_of_full_before_its_first_colon() {
    let rig = Rig::new();
    rig.mise_tool("codex", "codex", "aqua:openai/codex", "0.160.0");
    let row = only(read_mise(&mise_spec("codex", "codex"), &rig.paths));
    assert_eq!(row.source, "mise aqua");
}

#[test]
fn a_full_with_no_colon_reads_mise_core() {
    let rig = Rig::new();
    rig.mise_tool("tool", "tool", "tool", "1.0.0");
    let row = only(read_mise(&mise_spec("tool", "tool"), &rig.paths));
    assert_eq!(row.source, "mise core");
}

fn backend_error_row(dir: &str, file: &Path, reason: &str) -> ToolRow {
    ToolRow {
        name: dir.to_string(),
        state: InstallState::Installed,
        source: "mise".to_string(),
        installed: Reading::Unknown("backend file unreadable".to_string()),
        current: not_run(),
        verdict: Verdict::Error,
        error: Some(format!("{}: {reason}", file.display())),
        hint: None,
    }
}

fn backend_file(rig: &Rig, dir: &str) -> PathBuf {
    rig.paths
        .mise_data
        .join("installs")
        .join(dir)
        .join(".mise.backend.toml")
}

#[test]
fn a_bad_backend_directory_prints_one_error_row_after_the_tool_rows_with_the_exact_reason() {
    let cases: [(&str, Option<&str>); 6] = [
        ("missing", None),
        ("not valid TOML", Some("short = = =")),
        ("short is missing", Some("full = \"core:go\"\n")),
        (
            "short is not a string",
            Some("short = 3\nfull = \"core:go\"\n"),
        ),
        ("full is missing", Some("short = \"go\"\n")),
        ("full is not a string", Some("short = \"go\"\nfull = 3\n")),
    ];
    for (reason, text) in cases {
        let rig = Rig::new();
        rig.mise_version("go", "1.23.0");
        rig.mise_tool("codex", "codex", "aqua:openai/codex", "0.160.0");
        let file = backend_file(&rig, "go");
        if let Some(text) = text {
            fs::write(&file, text).expect("backend file");
        }
        let rows = read_tools(
            &[mise_spec("go", "go"), mise_spec("codex", "codex")],
            &rig.paths,
        );
        assert_eq!(rows.len(), 3, "{reason}: {rows:?}");
        assert_eq!(rows[0].name, "go", "{reason}");
        assert_eq!(
            rows[0].state,
            InstallState::Missing,
            "the bad directory reads as absent: {reason}"
        );
        assert_eq!(rows[1].source, "mise aqua", "{reason}");
        assert_eq!(rows[2], backend_error_row("go", &file, reason), "{reason}");
        let errors = rows.iter().filter(|r| r.verdict == Verdict::Error).count();
        assert_eq!(errors, 1, "one fault is printed once: {reason}");
    }
}

#[test]
fn a_bad_backend_directory_still_prints_its_row_when_no_mise_entry_is_listed() {
    let rig = Rig::new();
    rig.mise_version("orphan", "1.0.0");
    let file = backend_file(&rig, "orphan");
    let rows = read_tools(&[brew_spec("jq", "jq")], &rig.paths);
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0].name, "jq");
    assert_eq!(rows[1], backend_error_row("orphan", &file, "missing"));
}

#[test]
fn bad_backend_directories_print_one_row_each_in_directory_name_order() {
    let rig = Rig::new();
    rig.mise_version("b-tool", "1.0.0");
    rig.mise_version("a-tool", "1.0.0");
    let rows = read_tools(&[brew_spec("jq", "jq")], &rig.paths);
    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["jq", "a-tool", "b-tool"]);
}

#[test]
fn a_bad_backend_directory_reads_as_absent_to_the_entry_that_would_have_matched_it() {
    let rig = Rig::new();
    rig.mise_version("go", "1.23.0");
    let row = only(read_mise(&mise_spec("go", "go"), &rig.paths));
    assert_eq!(row.state, InstallState::Missing);
    assert_eq!(row.installed, Reading::None);
}

#[test]
fn an_unreadable_backend_file_gets_its_own_error_row_carrying_the_io_error() {
    let rig = Rig::new();
    rig.mise_tool("go", "go", "core:go", "1.23.0");
    let file = backend_file(&rig, "go");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).expect("chmod");
    let rows = read_tools(&[mise_spec("go", "go")], &rig.paths);
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0].state, InstallState::Missing);
    let row = &rows[1];
    assert_eq!(row.verdict, Verdict::Error);
    assert_eq!(row.name, "go");
    let error = row.error.clone().expect("an error row says why");
    assert!(
        error.starts_with(&format!("{}: ", file.display())),
        "{error}"
    );
    assert!(error.contains("Permission denied"), "{error}");
}

#[test]
fn a_mise_tool_with_no_installs_directory_is_missing_with_the_install_command() {
    let rig = Rig::new();
    let row = only(read_mise(&mise_spec("go", "go"), &rig.paths));
    assert_eq!(row.state, InstallState::Missing);
    assert_eq!(row.source, "mise");
    assert_eq!(row.installed, Reading::None);
    assert_eq!(
        row.hint,
        Some(Hint {
            kind: HintKind::Install,
            command: "mise use --global go".to_string(),
            source: "mise".to_string(),
        })
    );
}

#[test]
fn read_tools_asks_the_owning_installer_and_keeps_the_order_given() {
    let rig = Rig::new();
    rig.mise_tool("go", "go", "core:go", "1.23.0");
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    let rows = read_tools(&[mise_spec("go", "go"), brew_spec("jq", "jq")], &rig.paths);
    let names: Vec<&str> = rows.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(names, ["go", "jq"]);
    assert_eq!(rows[0].source, "mise core");
    assert_eq!(rows[1].source, "brew homebrew/core");
}

// ---- one name under two installers ----

fn marvel_specs() -> [ToolSpec; 2] {
    [
        brew_spec("marvel", "arcavenae/tap/marvel"),
        mise_spec("marvel", "github:ArcavenAE/marvel"),
    ]
}

fn install_marvel_with_brew(rig: &Rig) {
    rig.receipt_with_tap("marvel", "0.9.0", "arcavenae/tap");
    rig.link("marvel", "0.9.0");
}

fn install_marvel_with_mise(rig: &Rig) {
    rig.mise_tool(
        "github-arcaven-ae-marvel",
        "github:ArcavenAE/marvel",
        "github:ArcavenAE/marvel",
        "2.0.1",
    );
}

#[test]
fn a_name_installed_by_both_installers_gets_one_row_for_each() {
    let rig = Rig::new();
    install_marvel_with_brew(&rig);
    install_marvel_with_mise(&rig);
    let rows = read_tools(&marvel_specs(), &rig.paths);
    let sources: Vec<&str> = rows.iter().map(|row| row.source.as_str()).collect();
    assert_eq!(sources, ["brew arcavenae/tap", "mise github"], "{rows:?}");
    assert!(rows.iter().all(|row| row.name == "marvel"));
}

#[test]
fn a_name_installed_by_mise_only_prints_the_mise_row_and_no_brew_row() {
    let rig = Rig::new();
    install_marvel_with_mise(&rig);
    let row = only(read_tools(&marvel_specs(), &rig.paths));
    assert_eq!(row.source, "mise github");
    assert_eq!(row.state, InstallState::Installed);
    assert_eq!(row.hint, None);
}

#[test]
fn a_name_installed_by_brew_only_prints_the_brew_row_and_no_mise_row() {
    let rig = Rig::new();
    install_marvel_with_brew(&rig);
    let row = only(read_tools(&marvel_specs(), &rig.paths));
    assert_eq!(row.source, "brew arcavenae/tap");
    assert_eq!(row.state, InstallState::Installed);
}

#[test]
fn a_name_installed_by_neither_prints_one_missing_row_with_the_first_entrys_install_line() {
    let rig = Rig::new();
    let row = only(read_tools(&marvel_specs(), &rig.paths));
    assert_eq!(row.state, InstallState::Missing);
    assert_eq!(row.source, "brew");
    assert_eq!(
        row.hint,
        Some(Hint {
            kind: HintKind::Install,
            command: "brew install arcavenae/tap/marvel".to_string(),
            source: "brew".to_string(),
        })
    );
}

#[test]
fn a_brew_error_and_an_installed_mise_entry_print_both_rows() {
    let rig = Rig::new();
    rig.receipt("marvel", "0.9.0", "{ not json");
    rig.link("marvel", "0.9.0");
    install_marvel_with_mise(&rig);
    let rows = read_tools(&marvel_specs(), &rig.paths);
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0].verdict, Verdict::Error);
    assert_eq!(rows[1].source, "mise github");
}

#[test]
fn a_name_whose_only_readings_are_errors_prints_those_rows_and_no_missing_row() {
    let rig = Rig::new();
    rig.receipt("marvel", "0.9.0", "{ not json");
    rig.link("marvel", "0.9.0");
    let row = only(read_tools(&marvel_specs(), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    assert_ne!(row.state, InstallState::Missing);
}

// ---- a tool with no recorded installer ----

const NO_INSTALLER_NAMES: [&str; 5] = ["director", "aq", "ax", "akey", "tmux-cmc"];

fn no_installer_row(name: &str) -> ToolRow {
    ToolRow {
        name: name.to_string(),
        state: InstallState::NoInstaller,
        source: "unknown source".to_string(),
        installed: Reading::Unknown("no installer recorded".to_string()),
        current: not_run(),
        verdict: Verdict::Unknown("no installer recorded".to_string()),
        error: None,
        hint: None,
    }
}

fn none_spec(name: &'static str) -> ToolSpec {
    ToolSpec {
        name,
        install: None,
    }
}

#[test]
fn each_tool_with_no_recorded_installer_reads_exactly_the_ruled_row_with_no_command_lines() {
    let rig = Rig::new();
    let specs: Vec<ToolSpec> = NO_INSTALLER_NAMES.iter().map(|n| none_spec(n)).collect();
    let rows = read_tools(&specs, &rig.paths);
    let expected: Vec<ToolRow> = NO_INSTALLER_NAMES
        .iter()
        .map(|name| no_installer_row(name))
        .collect();
    assert_eq!(rows, expected);
    let text = render_rows(&rows);
    assert!(
        !text.contains("    install:") && !text.contains("    update:"),
        "{text}"
    );
}

/// PATH is not read in this change: the verb reads installer files only, so a placed
/// executable cannot be found and cannot be run. This red places executables named like
/// the tool on disk and proves the row is unchanged and nothing is touched. PATH comes
/// with the readers that need it, each with its own red.
#[test]
fn an_executable_with_the_tools_name_changes_nothing_because_nothing_is_looked_up() {
    let rig = Rig::new();
    for dir in [
        rig.scratch.home().join(".director/bin"),
        rig.scratch.home().join("bin"),
    ] {
        fs::create_dir_all(&dir).expect("bin dir");
        let exe = dir.join("director");
        fs::write(&exe, "#!/bin/sh\n").expect("exe");
        fs::set_permissions(&exe, fs::Permissions::from_mode(0o755)).expect("chmod");
    }
    let guard = InstallerGuard::snapshot(&rig.paths).expect("a snapshot");
    let row = only(read_tools(&[none_spec("director")], &rig.paths));
    assert_eq!(row, no_installer_row("director"));
    guard.verify().expect("nothing new under installer state");
}

// ---- the verb ----

fn context(rig: &Rig) -> ToolsContext {
    ToolsContext {
        specs: vec![
            spec("go", Installer::Mise, "go"),
            spec("jq", Installer::Brew, "jq"),
        ],
        paths: rig.paths.clone(),
    }
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn the_verb_prints_the_rows_for_the_tools_it_is_given() {
    let rig = Rig::new();
    rig.mise_tool("go", "go", "core:go", "1.23.0");
    let context = context(&rig);
    let out = run_tools(&args(&["tools"]), &context).expect("rows");
    let expected = render_rows(&read_tools(&context.specs, &context.paths));
    assert!(!expected.is_empty(), "the fixture has rows");
    assert_eq!(out, expected);
    assert!(out.contains("go  installed  mise core  1.23.0"), "{out}");
    assert!(out.contains("jq  missing  brew"), "{out}");
}

#[test]
fn an_error_row_does_not_fail_the_verb() {
    let rig = Rig::new();
    rig.receipt("jq", "1.7.1", "{ not json");
    rig.link("jq", "1.7.1");
    let out = run_tools(&args(&["tools"]), &context(&rig)).expect("the verb still prints");
    assert!(out.contains("  error"), "{out}");
}

#[test]
fn the_verb_takes_no_arguments_and_says_so_with_the_four_fields() {
    let rig = Rig::new();
    let err = run_tools(&args(&["tools", "extra"]), &context(&rig))
        .expect_err("an argument must be refused");
    assert_eq!(err.step(), "run tools");
    assert_eq!(err.check(), "the verb takes no arguments");
    assert_eq!(err.status(), "usher tools takes no arguments; got `extra`");
    assert_eq!(err.next_action(), "run `usher tools`");
}

fn env_in(scratch: &ScratchHome) -> InventoryEnv {
    InventoryEnv {
        host: "host-1".to_string(),
        user: "alex".to_string(),
        home: scratch.home().to_path_buf(),
        launch_dir: scratch.home().to_path_buf(),
        launch: LaunchEnvironment::default(),
        path: None,
        state_dir: scratch.home().join("state"),
        now: "2026-11-20T12:00:00Z".to_string(),
        platform: Platform::Linux,
    }
}

#[test]
fn an_unknown_command_names_the_tools_verb_in_its_next_action() {
    let scratch = ScratchHome::new().expect("scratch");
    let err = run(&args(&["whatever"]), &env_in(&scratch)).expect_err("not a command");
    assert!(err.next_action().contains("usher tools"), "{err}");
}

#[test]
fn no_command_names_the_tools_verb_in_its_next_action() {
    let scratch = ScratchHome::new().expect("scratch");
    let err = run(&args(&[]), &env_in(&scratch)).expect_err("a command is required");
    assert!(err.next_action().contains("usher tools"), "{err}");
}

#[test]
fn run_hands_the_tools_command_to_the_tools_verb_and_not_to_inventory() {
    let scratch = ScratchHome::new().expect("scratch");
    let result = run(&args(&["tools"]), &env_in(&scratch));
    assert!(result.is_ok(), "`usher tools` is a command: {result:?}");
    let out = result.unwrap_or_default();
    assert!(
        out.contains("director  unknown (no installer recorded)  unknown source"),
        "the tools rows were expected, and the director row needs no host state: {out}"
    );
}

#[test]
fn run_refuses_an_argument_to_tools_with_the_four_fields_before_reading_anything() {
    let scratch = ScratchHome::new().expect("scratch");
    let env = env_in(&scratch);
    let err = run(&args(&["tools", "extra"]), &env).expect_err("an argument must be refused");
    assert_eq!(err.step(), "run tools");
    assert_eq!(err.check(), "the verb takes no arguments");
    assert_eq!(err.status(), "usher tools takes no arguments; got `extra`");
    assert_eq!(err.next_action(), "run `usher tools`");
    assert!(!env.state_dir.exists(), "nothing was read or created");
}

// ---- nothing runs an installer ----

const FORBIDDEN: [&str; 9] = [
    "Command::new",
    "std::process",
    "process::Command",
    ".spawn(",
    ".output(",
    ".status(",
    "process::{",
    ".exec(",
    "CommandExt",
];

/// The process-call forms in `text`. `.status(` also names `UsherError::status()`, which
/// the test modules of both crates call, so it counts only in a file that names
/// `Command`; a file cannot spawn a process without naming it, and the other forms
/// already refuse the imports that would let it.
fn forbidden_forms(text: &str) -> Vec<&'static str> {
    FORBIDDEN
        .iter()
        .copied()
        .filter(|form| text.contains(form) && (*form != ".status(" || text.contains("Command")))
        .collect()
}

fn rust_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("a readable directory") {
            rust_files(&entry.expect("an entry").path(), out);
        }
    } else if path.extension().is_some_and(|ext| ext == "rs") {
        out.push(path.to_path_buf());
    }
}

#[test]
fn the_scan_finds_each_process_call_form_in_text_that_has_it() {
    for form in FORBIDDEN {
        let text = format!("fn f() {{ let _ = x{form}y; }}\n// Command");
        assert_eq!(forbidden_forms(&text), [form], "{form}");
    }
    assert!(forbidden_forms("fn f() {}").is_empty());
}

#[test]
fn the_scan_finds_a_spawn_through_an_import_alias_and_an_exec() {
    let text = "use std::os::unix::process::{CommandExt as _};\nuse std::{process::{Command as K}};\nfn f() { K::new(\"brew\").arg(\"update\").exec(); }";
    assert!(!forbidden_forms(text).is_empty(), "{text}");
}

#[test]
fn the_scan_lets_an_error_status_call_through_in_a_file_that_runs_no_command() {
    assert!(forbidden_forms("assert_eq!(err.status(), \"x\");").is_empty());
}

#[test]
fn no_file_in_the_cli_or_core_source_runs_a_process_except_the_hostname_lookup() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for relative in ["src", "../usher-core/src", "../usher/src"] {
        rust_files(&manifest.join(relative), &mut files);
    }
    let names: Vec<String> = files
        .iter()
        .filter_map(|f| f.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    for expected in ["lib.rs", "inventory.rs", "tools.rs", "egress.rs", "main.rs"] {
        assert!(
            names.iter().any(|n| n == expected),
            "the scan must reach every file, and missed {expected}: {names:?}"
        );
    }
    for file in &files {
        let text = fs::read_to_string(file).expect("a readable file");
        if *file == manifest.join("src/inventory.rs") {
            continue;
        }
        assert!(
            forbidden_forms(&text).is_empty(),
            "{} runs a process: {:?}",
            file.display(),
            forbidden_forms(&text)
        );
    }
}

#[test]
fn the_one_exempt_file_still_holds_exactly_its_hostname_spawn() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let text = fs::read_to_string(manifest.join("src/inventory.rs")).expect("inventory.rs");
    assert_eq!(text.matches("Command::new").count(), 1);
    assert_eq!(text.matches(".output(").count(), 1);
    for form in [".spawn(", ".exec(", "CommandExt", "process::{"] {
        assert!(!text.contains(form), "the exemption does not cover {form}");
    }
}

// ---- the observation invariant covers installer state ----

#[test]
fn running_the_verb_leaves_installer_state_and_a_watched_harness_home_unchanged() {
    let rig = Rig::new();
    rig.mise_tool("go", "go", "core:go", "1.23.0");
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    let harness_home = rig.scratch.claude_config_dir().to_path_buf();
    fs::create_dir_all(&harness_home).expect("a harness home");
    let homes = RealHomes::from_homes(vec![NamedHome {
        name: "claude-code",
        path: harness_home,
    }]);
    let guard = InstallerGuard::snapshot(&rig.paths).expect("a snapshot");
    observe(&homes, |_scratch| {
        run_tools(&args(&["tools"]), &context(&rig)).expect("rows");
    });
    guard.verify().expect("the verb changed no installer state");
}

/// Runs the verb against the real installer state, then again with scratch harness
/// homes under observation. The real harness homes are left out on purpose: the verb
/// opens none, and a harness session on the same host writes there while the test runs,
/// so a change would be a false report against the verb. A brew or mise run by the person
/// while this test runs reads as a violation: that is a true report, not a flake to
/// retry. The first verb that reads a harness home must watch that home in its real
/// location, and say how it tells usher's writes from the session's.
#[test]
fn the_verb_changes_no_real_installer_state_and_no_scratch_harness_home() {
    let paths = InstallerPaths::resolve(
        |name| std::env::var(name).ok(),
        |path| path.exists(),
        cfg!(target_os = "macos"),
    );
    let real = ToolsContext {
        specs: tool_specs(),
        paths: paths.clone(),
    };
    let guard = InstallerGuard::snapshot(&paths).expect("a snapshot of the real state");
    run_tools(&args(&["tools"]), &real).expect("rows");
    guard
        .verify()
        .expect("the verb changed no real installer state");

    let rig = Rig::new();
    let home = rig.scratch.home();
    fs::create_dir_all(home.join(".claude")).expect("a claude home");
    fs::write(home.join(".claude.json"), "{}").expect("global state");
    fs::create_dir_all(home.join(".codex")).expect("a codex home");
    let watched = RealHomes::from_homes(vec![
        NamedHome {
            name: "claude-code",
            path: home.join(".claude"),
        },
        NamedHome {
            name: "claude-code-global-state",
            path: home.join(".claude.json"),
        },
        NamedHome {
            name: "codex",
            path: home.join(".codex"),
        },
    ]);
    observe(&watched, |_scratch| {
        run_tools(&args(&["tools"]), &context(&rig)).expect("rows");
    });
}

#[test]
fn the_installer_guard_fires_when_a_run_writes_under_a_watched_path() {
    let rig = Rig::new();
    let guard = InstallerGuard::snapshot(&rig.paths).expect("a snapshot");
    rig.mise_version("go", "9.9.9");
    assert!(
        guard.verify().is_err(),
        "a write under the mise installs directory must be seen"
    );
}
