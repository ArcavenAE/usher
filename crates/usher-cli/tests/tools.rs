//! Story 2.1, PR 1: `usher tools` reads brew and mise state from their files and runs no
//! installer. Every fixture lives in a scratch home.

use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use usher_cli::{
    InventoryEnv, LaunchEnvironment, ToolsContext, read_brew, read_mise, read_tools, run, run_tools,
};
use usher_core::{
    Hint, HintKind, InstallState, Installer, InstallerPaths, Reading, ToolRow, ToolSpec, Verdict,
    render_rows,
};
use usher_knowledge::Platform;
use usher_testkit::ScratchHome;

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
            brew_repository: root.join("brew"),
            brew_prefix: root.join("brew"),
            brew_cache: root.join("brew-cache"),
            mise_data: root.join("mise/data"),
            mise_cache: root.join("mise/cache"),
            mise_state: root.join("mise/state"),
        };
        fs::create_dir_all(&paths.brew_prefix).expect("brew prefix");
        fs::create_dir_all(&paths.mise_data).expect("mise data");
        Rig { scratch, paths }
    }

    fn cellar(&self, formula: &str, version: &str) -> PathBuf {
        let dir = self
            .paths
            .brew_prefix
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

    fn link(&self, formula: &str, version: &str) {
        let dir = self.paths.brew_prefix.join("var/homebrew/linked");
        fs::create_dir_all(&dir).expect("linked dir");
        symlink(
            format!("../../../Cellar/{formula}/{version}"),
            dir.join(formula),
        )
        .expect("link");
    }

    fn tap_formula(&self, tap: &str, sharded: bool, formula: &str) {
        let (org, name) = tap.split_once('/').expect("org/name");
        let mut dir = self
            .paths
            .brew_repository
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

fn spec(name: &str, installer: Installer, package: &str) -> ToolSpec {
    ToolSpec {
        name: name.to_string(),
        installer,
        package: package.to_string(),
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

#[test]
fn an_installed_formula_reads_its_version_from_the_cellar_and_its_tap_from_the_receipt() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    let row = only(read_brew(&spec("jq", Installer::Brew, "jq"), &rig.paths));
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
    let row = only(read_brew(&spec("jq", Installer::Brew, "jq"), &rig.paths));
    assert_eq!(row.state, InstallState::Missing);
    assert_eq!(row.source, "brew");
    assert_eq!(row.installed, Reading::Unknown("not installed".to_string()));
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
fn a_receipt_that_is_not_json_is_an_error_row_naming_the_file_never_current() {
    let rig = Rig::new();
    let path = rig.receipt("jq", "1.7.1", "{ not json");
    rig.link("jq", "1.7.1");
    let row = only(read_brew(&spec("jq", Installer::Brew, "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    let error = row.error.expect("an error row says why");
    assert!(error.contains(&path.display().to_string()), "{error}");
    assert!(error.contains("not valid JSON"), "{error}");
    assert_ne!(row.verdict, Verdict::Current);
}

#[test]
fn a_receipt_without_a_source_tap_is_an_error_row_that_does_not_guess_a_source() {
    let rig = Rig::new();
    let path = rig.receipt("jq", "1.7.1", r#"{"source":{"path":"/x"}}"#);
    rig.link("jq", "1.7.1");
    let row = only(read_brew(&spec("jq", Installer::Brew, "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    let error = row.error.expect("an error row says why");
    assert!(error.contains(&path.display().to_string()), "{error}");
    assert!(error.contains("source.tap is missing"), "{error}");
    assert_eq!(row.source, "unknown source");
}

#[test]
fn a_receipt_whose_source_tap_is_not_a_string_is_an_error_row() {
    let rig = Rig::new();
    rig.receipt("jq", "1.7.1", r#"{"source":{"tap":null}}"#);
    rig.link("jq", "1.7.1");
    let row = only(read_brew(&spec("jq", Installer::Brew, "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    let error = row.error.expect("an error row says why");
    assert!(error.contains("source.tap is not a string"), "{error}");
}

#[test]
fn with_two_versions_installed_the_linked_one_is_read() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.0");
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.0");
    let row = only(read_brew(&spec("jq", Installer::Brew, "jq"), &rig.paths));
    assert_eq!(row.installed, Reading::Known("1.7.0".to_string()));
    assert_eq!(row.verdict, Verdict::Unknown(NOT_RUN.to_string()));
}

#[test]
fn with_two_versions_installed_and_none_linked_the_row_is_an_error_naming_both() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.0");
    rig.good_receipt("jq", "1.7.1");
    let row = only(read_brew(&spec("jq", Installer::Brew, "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    let error = row.error.expect("an error row says why");
    assert!(
        error.contains("1.7.0") && error.contains("1.7.1"),
        "{error}"
    );
    assert!(error.contains("none is linked"), "{error}");
}

#[test]
fn a_formula_in_two_taps_is_an_error_row_naming_both_taps() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    rig.tap_formula("acme/tools", false, "jq");
    rig.tap_formula("other/extras", true, "jq");
    let row = only(read_brew(&spec("jq", Installer::Brew, "jq"), &rig.paths));
    assert_eq!(row.verdict, Verdict::Error);
    let error = row.error.expect("an error row says why");
    assert!(
        error.contains("acme/tools") && error.contains("other/extras"),
        "{error}"
    );
}

#[test]
fn a_formula_in_one_tap_is_not_an_error() {
    let rig = Rig::new();
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    rig.tap_formula("acme/tools", false, "jq");
    let row = only(read_brew(&spec("jq", Installer::Brew, "jq"), &rig.paths));
    assert_eq!(row.state, InstallState::Installed);
    assert_ne!(row.verdict, Verdict::Error);
}

// ---- mise ----

#[test]
fn a_mise_tool_reads_its_versions_from_the_installs_directory_names() {
    let rig = Rig::new();
    rig.mise_version("go", "1.23.0");
    let row = only(read_mise(&spec("go", Installer::Mise, "go"), &rig.paths));
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
    rig.mise_version("go", "1.23.0");
    rig.mise_version("go", "1.22.0");
    let row = only(read_mise(&spec("go", Installer::Mise, "go"), &rig.paths));
    assert_eq!(row.installed, Reading::Known("1.22.0, 1.23.0".to_string()));
}

#[test]
fn a_symlink_in_the_installs_directory_is_not_a_version() {
    let rig = Rig::new();
    rig.mise_version("go", "1.23.0");
    symlink("1.23.0", rig.paths.mise_data.join("installs/go/latest")).expect("alias");
    let row = only(read_mise(&spec("go", Installer::Mise, "go"), &rig.paths));
    assert_eq!(row.installed, Reading::Known("1.23.0".to_string()));
}

#[test]
fn a_backend_package_reads_its_directory_and_names_its_backend() {
    let rig = Rig::new();
    rig.mise_version("github-acme-tool", "2.0.1");
    let row = only(read_mise(
        &spec("tool", Installer::Mise, "github:acme/tool"),
        &rig.paths,
    ));
    assert_eq!(row.source, "mise github");
    assert_eq!(row.installed, Reading::Known("2.0.1".to_string()));
}

#[test]
fn a_mise_tool_with_no_installs_directory_is_missing_with_the_install_command() {
    let rig = Rig::new();
    let row = only(read_mise(&spec("go", Installer::Mise, "go"), &rig.paths));
    assert_eq!(row.state, InstallState::Missing);
    assert_eq!(row.source, "mise");
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
    rig.mise_version("go", "1.23.0");
    rig.good_receipt("jq", "1.7.1");
    rig.link("jq", "1.7.1");
    let rows = read_tools(
        &[
            spec("go", Installer::Mise, "go"),
            spec("jq", Installer::Brew, "jq"),
        ],
        &rig.paths,
    );
    let names: Vec<&str> = rows.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(names, ["go", "jq"]);
    assert_eq!(rows[0].source, "mise core");
    assert_eq!(rows[1].source, "brew homebrew/core");
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
    rig.mise_version("go", "1.23.0");
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

#[test]
fn an_unknown_command_names_the_tools_verb_in_its_next_action() {
    let scratch = ScratchHome::new().expect("scratch");
    let env = InventoryEnv {
        host: "host-1".to_string(),
        user: "alex".to_string(),
        home: scratch.home().to_path_buf(),
        launch_dir: scratch.home().to_path_buf(),
        launch: LaunchEnvironment::default(),
        path: None,
        state_dir: scratch.home().join("state"),
        now: "2026-11-20T12:00:00Z".to_string(),
        platform: Platform::Linux,
    };
    let err = run(&args(&["whatever"]), &env).expect_err("not a command");
    assert!(err.next_action().contains("usher tools"), "{err}");
}

// ---- nothing runs an installer ----

const FORBIDDEN: [&str; 6] = [
    "Command::new",
    "std::process",
    "process::Command",
    ".spawn(",
    ".output(",
    ".status(",
];

fn forbidden_forms(text: &str) -> Vec<&'static str> {
    FORBIDDEN
        .iter()
        .copied()
        .filter(|form| text.contains(form))
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
        let text = format!("fn f() {{ let _ = x{form}y; }}");
        assert_eq!(forbidden_forms(&text), [form], "{form}");
    }
    assert!(forbidden_forms("fn f() {}").is_empty());
}

#[test]
fn the_new_code_holds_no_process_call_form() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for relative in [
        "src/tools.rs",
        "src/tools",
        "../usher-core/src/tools.rs",
        "../usher-core/src/tools",
    ] {
        let path = manifest.join(relative);
        if path.exists() {
            rust_files(&path, &mut files);
        }
    }
    assert!(
        files.len() >= 2,
        "the scan saw {} files; the model and the verb are both new code",
        files.len()
    );
    for file in &files {
        let text = fs::read_to_string(file).expect("a readable file");
        assert!(
            forbidden_forms(&text).is_empty(),
            "{} runs a process: {:?}",
            file.display(),
            forbidden_forms(&text)
        );
    }
}
