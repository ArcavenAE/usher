//! Story 2.1, PR 1: the row model, the installer tables and the path rules. Nothing here
//! runs an installer or reads the host.

use std::path::{Path, PathBuf};

use usher_core::{
    BrewPaths, Hint, HintKind, InstallState, Installer, InstallerPaths, Reading, Recorded, ToolRow,
    ToolSpec, Verdict, allowed_invocations, command_text, never_call, no_update_env, render_rows,
    tool_specs,
};

const NOT_RUN: &str = "version check not run";

fn installed(name: &str, source: &str, version: &str) -> ToolRow {
    ToolRow {
        name: name.to_string(),
        state: InstallState::Installed,
        source: source.to_string(),
        installed: Reading::Known(version.to_string()),
        current: Reading::Unknown(NOT_RUN.to_string()),
        verdict: Verdict::Unknown(NOT_RUN.to_string()),
        error: None,
        hint: None,
    }
}

fn missing(name: &str, installer: &str, command: &str) -> ToolRow {
    ToolRow {
        name: name.to_string(),
        state: InstallState::Missing,
        source: installer.to_string(),
        installed: Reading::None,
        current: Reading::Unknown(NOT_RUN.to_string()),
        verdict: Verdict::Unknown(NOT_RUN.to_string()),
        error: None,
        hint: Some(Hint {
            kind: HintKind::Install,
            command: command.to_string(),
            source: installer.to_string(),
        }),
    }
}

// ---- the row text ----

#[test]
fn a_row_prints_six_fields_in_order_separated_by_two_spaces() {
    let text = render_rows(&[installed("jq", "brew homebrew/core", "1.7.1")]);
    assert_eq!(
        text,
        "jq  installed  brew homebrew/core  1.7.1  unknown (version check not run)  unknown (version check not run)"
    );
}

#[test]
fn rows_print_one_per_line_in_the_order_given() {
    let text = render_rows(&[
        installed("go", "mise core", "1.23.0"),
        installed("jq", "brew homebrew/core", "1.7.1"),
    ]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "{text}");
    assert!(lines[0].starts_with("go  installed  mise core  1.23.0"));
    assert!(lines[1].starts_with("jq  installed  brew homebrew/core  1.7.1"));
}

#[test]
fn a_missing_tool_prints_the_owning_installers_install_command_and_source_under_its_row() {
    let text = render_rows(&[missing("jq", "brew", "brew install jq")]);
    assert_eq!(
        text,
        "jq  missing  brew  none  unknown (version check not run)  unknown (version check not run)\n    install: brew install jq  (source brew)"
    );
}

#[test]
fn a_behind_tool_prints_the_update_command_under_its_row() {
    let mut row = installed("jq", "brew homebrew/core", "1.7.0");
    row.current = Reading::Known("1.7.1".to_string());
    row.verdict = Verdict::Behind;
    row.hint = Some(Hint {
        kind: HintKind::Update,
        command: "brew upgrade jq".to_string(),
        source: "brew homebrew/core".to_string(),
    });
    let text = render_rows(&[row]);
    assert_eq!(
        text,
        "jq  installed  brew homebrew/core  1.7.0  1.7.1  behind\n    update: brew upgrade jq  (source brew homebrew/core)"
    );
}

#[test]
fn an_error_row_says_error_and_gives_the_failing_step_on_its_own_line_never_current() {
    let row = ToolRow {
        name: "jq".to_string(),
        state: InstallState::Installed,
        source: "unknown source".to_string(),
        installed: Reading::Unknown("receipt unreadable".to_string()),
        current: Reading::Unknown(NOT_RUN.to_string()),
        verdict: Verdict::Error,
        error: Some("/p/INSTALL_RECEIPT.json: not valid JSON".to_string()),
        hint: None,
    };
    let text = render_rows(&[row]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "{text}");
    assert!(lines[0].ends_with("  error"), "{text}");
    assert_eq!(
        lines[1],
        "    error: /p/INSTALL_RECEIPT.json: not valid JSON"
    );
    assert!(
        !text.contains("current"),
        "an error never reads as current: {text}"
    );
}

#[test]
fn with_no_rows_the_text_is_empty() {
    assert_eq!(render_rows(&[]), "");
}

#[test]
fn a_missing_tool_reads_none_for_its_installed_version_and_never_unknown() {
    let text = render_rows(&[missing("jq", "brew", "brew install jq")]);
    let fields: Vec<&str> = text.lines().next().expect("a row").split("  ").collect();
    assert_eq!(fields[3], "none", "{text}");
}

#[test]
fn a_tool_with_no_recorded_installer_prints_its_exact_row_and_no_command_lines() {
    let row = ToolRow {
        name: "director".to_string(),
        state: InstallState::NoInstaller,
        source: "unknown source".to_string(),
        installed: Reading::Unknown("no installer recorded".to_string()),
        current: Reading::Unknown(NOT_RUN.to_string()),
        verdict: Verdict::Unknown("no installer recorded".to_string()),
        error: None,
        hint: None,
    };
    assert_eq!(
        render_rows(&[row]),
        "director  unknown (no installer recorded)  unknown source  unknown (no installer recorded)  unknown (version check not run)  unknown (no installer recorded)"
    );
}

#[test]
fn a_brew_entry_with_no_brew_prints_a_missing_row_with_the_reason_and_the_install_line() {
    let mut row = missing("jq", "brew", "brew install jq");
    row.verdict = Verdict::Unknown("brew not found".to_string());
    assert_eq!(
        render_rows(&[row]),
        "jq  missing  brew  none  unknown (version check not run)  unknown (brew not found)\n    install: brew install jq  (source brew)"
    );
}

// ---- the tools list ----

fn recorded(installer: Installer, package: &'static str) -> Option<Recorded> {
    Some(Recorded { installer, package })
}

#[test]
fn the_tools_list_is_exactly_the_twelve_ruled_entries_in_order() {
    let entry = |name, install| ToolSpec { name, install };
    assert_eq!(
        tool_specs(),
        vec![
            entry("codex", recorded(Installer::Mise, "codex")),
            entry("goose", recorded(Installer::Brew, "block-goose-cli")),
            entry("marvel", recorded(Installer::Brew, "arcavenae/tap/marvel")),
            entry(
                "marvel",
                recorded(Installer::Mise, "github:ArcavenAE/marvel")
            ),
            entry("director", None),
            entry(
                "sideshow",
                recorded(Installer::Brew, "arcavenae/tap/sideshow")
            ),
            entry("kos", recorded(Installer::Brew, "arcavenae/tap/kos")),
            entry("aq", None),
            entry("ax", None),
            entry("akey", None),
            entry("tmux-cmc", None),
            entry("mise", recorded(Installer::Brew, "mise")),
        ]
    );
}

// ---- the installer tables ----

#[test]
fn installers_are_named_as_their_commands_are() {
    assert_eq!(Installer::Brew.name(), "brew");
    assert_eq!(Installer::Mise.name(), "mise");
}

#[test]
fn the_brew_never_call_list_is_exactly_the_ruled_one() {
    assert_eq!(
        never_call(Installer::Brew),
        [
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
        ]
    );
}

#[test]
fn the_mise_never_call_list_is_exactly_the_ruled_one() {
    assert_eq!(
        never_call(Installer::Mise),
        [
            "install",
            "upgrade",
            "up",
            "use",
            "prune",
            "reshim",
            "self-update",
            "plugins update",
            "cache clear",
        ]
    );
}

#[test]
fn the_brew_no_update_environment_is_exactly_the_ruled_one() {
    let env = no_update_env(Installer::Brew);
    assert_eq!(
        env,
        [
            ("HOMEBREW_NO_AUTO_UPDATE", "1"),
            ("HOMEBREW_NO_INSTALL_CLEANUP", "1"),
            ("HOMEBREW_NO_ANALYTICS", "1"),
            ("HOMEBREW_NO_ENV_HINTS", "1"),
        ]
    );
    assert!(
        env.iter()
            .all(|(name, _)| *name != "HOMEBREW_NO_INSTALL_FROM_API"),
        "that switch changes which formula source brew reads"
    );
}

#[test]
fn the_mise_no_update_environment_keeps_mise_offline() {
    assert_eq!(no_update_env(Installer::Mise), [("MISE_OFFLINE", "1")]);
}

#[test]
fn no_invocation_is_allowed_while_nothing_runs_an_installer() {
    assert!(allowed_invocations().is_empty());
}

#[test]
fn the_command_text_for_each_installer_is_pinned() {
    assert_eq!(
        command_text(Installer::Brew, HintKind::Install, "jq"),
        "brew install jq"
    );
    assert_eq!(
        command_text(Installer::Brew, HintKind::Update, "jq"),
        "brew upgrade jq"
    );
    assert_eq!(
        command_text(Installer::Mise, HintKind::Install, "go"),
        "mise use --global go"
    );
    assert_eq!(
        command_text(Installer::Mise, HintKind::Update, "go"),
        "mise upgrade go"
    );
}

#[test]
fn a_tap_qualified_brew_package_prints_its_full_name_in_both_commands() {
    assert_eq!(
        command_text(Installer::Brew, HintKind::Install, "arcavenae/tap/marvel"),
        "brew install arcavenae/tap/marvel"
    );
    assert_eq!(
        command_text(Installer::Brew, HintKind::Update, "arcavenae/tap/marvel"),
        "brew upgrade arcavenae/tap/marvel"
    );
}

#[test]
fn a_mise_backend_package_prints_its_spec_in_both_commands() {
    assert_eq!(
        command_text(
            Installer::Mise,
            HintKind::Install,
            "github:ArcavenAE/marvel"
        ),
        "mise use --global github:ArcavenAE/marvel"
    );
    assert_eq!(
        command_text(Installer::Mise, HintKind::Update, "github:ArcavenAE/marvel"),
        "mise upgrade github:ArcavenAE/marvel"
    );
}

// ---- where the installers keep their state ----

fn resolve(env: &[(&str, &str)], existing: &[&str], macos: bool) -> InstallerPaths {
    let get = |key: &str| {
        env.iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| (*value).to_string())
    };
    let exists = |path: &Path| existing.iter().any(|e| Path::new(e) == path);
    InstallerPaths::resolve(get, exists, macos)
}

fn brew(paths: &InstallerPaths) -> &BrewPaths {
    paths.brew.as_ref().expect("brew paths were found")
}

fn p(text: &str) -> PathBuf {
    PathBuf::from(text)
}

#[test]
fn brew_paths_come_from_the_environment_when_it_is_absolute() {
    let paths = resolve(
        &[
            ("HOME", "/h"),
            ("HOMEBREW_REPOSITORY", "/b/repo"),
            ("HOMEBREW_PREFIX", "/b/prefix"),
            ("HOMEBREW_CACHE", "/b/cache"),
        ],
        &["/opt/homebrew"],
        true,
    );
    assert_eq!(brew(&paths).repository, p("/b/repo"));
    assert_eq!(brew(&paths).prefix, p("/b/prefix"));
    assert_eq!(brew(&paths).cache, p("/b/cache"));
}

#[test]
fn an_absolute_brew_environment_gives_brew_paths_when_no_fixed_location_exists() {
    let paths = resolve(
        &[
            ("HOME", "/h"),
            ("HOMEBREW_REPOSITORY", "/custom/repo"),
            ("HOMEBREW_PREFIX", "/custom/prefix"),
        ],
        &[],
        true,
    );
    assert_eq!(brew(&paths).repository, p("/custom/repo"));
    assert_eq!(brew(&paths).prefix, p("/custom/prefix"));
}

#[test]
fn with_no_brew_found_the_two_fixed_locations_are_carried_as_candidates_and_not_usr_local() {
    let paths = resolve(&[("HOME", "/h")], &[], true);
    assert_eq!(
        paths.brew_candidates,
        [p("/opt/homebrew"), p("/usr/local/Homebrew")]
    );
}

#[test]
fn a_relative_brew_variable_is_ignored() {
    let paths = resolve(
        &[("HOME", "/h"), ("HOMEBREW_REPOSITORY", "repo")],
        &["/opt/homebrew"],
        true,
    );
    assert_eq!(brew(&paths).repository, p("/opt/homebrew"));
}

#[test]
fn brew_falls_back_to_the_first_fixed_location_that_exists() {
    let apple = resolve(&[("HOME", "/h")], &["/opt/homebrew"], true);
    assert_eq!(brew(&apple).repository, p("/opt/homebrew"));
    assert_eq!(brew(&apple).prefix, p("/opt/homebrew"));

    let intel = resolve(&[("HOME", "/h")], &["/usr/local/Homebrew"], true);
    assert_eq!(brew(&intel).repository, p("/usr/local/Homebrew"));
    assert_eq!(brew(&intel).prefix, p("/usr/local"));

    let both = resolve(
        &[("HOME", "/h")],
        &["/opt/homebrew", "/usr/local/Homebrew"],
        true,
    );
    assert_eq!(brew(&both).repository, p("/opt/homebrew"));
}

#[test]
fn the_brew_cache_defaults_by_platform() {
    let mac = resolve(&[("HOME", "/h")], &["/opt/homebrew"], true);
    assert_eq!(brew(&mac).cache, p("/h/Library/Caches/Homebrew"));
    let linux = resolve(&[("HOME", "/h")], &["/opt/homebrew"], false);
    assert_eq!(brew(&linux).cache, p("/h/.cache/Homebrew"));
}

#[test]
fn the_mise_data_directory_follows_mise_data_dir_then_xdg_then_home() {
    let set = resolve(
        &[
            ("HOME", "/h"),
            ("MISE_DATA_DIR", "/m/data"),
            ("XDG_DATA_HOME", "/x"),
        ],
        &[],
        true,
    );
    assert_eq!(set.mise_data, p("/m/data"));
    let xdg = resolve(&[("HOME", "/h"), ("XDG_DATA_HOME", "/x")], &[], true);
    assert_eq!(xdg.mise_data, p("/x/mise"));
    let home = resolve(&[("HOME", "/h")], &[], true);
    assert_eq!(home.mise_data, p("/h/.local/share/mise"));
}

#[test]
fn the_mise_cache_and_state_directories_follow_their_variables_then_the_platform() {
    let set = resolve(
        &[
            ("HOME", "/h"),
            ("MISE_CACHE_DIR", "/m/cache"),
            ("MISE_STATE_DIR", "/m/state"),
        ],
        &[],
        true,
    );
    assert_eq!(set.mise_cache, p("/m/cache"));
    assert_eq!(set.mise_state, p("/m/state"));

    let mac = resolve(&[("HOME", "/h"), ("XDG_CACHE_HOME", "/xc")], &[], true);
    assert_eq!(mac.mise_cache, p("/h/Library/Caches/mise"));
    let linux_xdg = resolve(&[("HOME", "/h"), ("XDG_CACHE_HOME", "/xc")], &[], false);
    assert_eq!(linux_xdg.mise_cache, p("/xc/mise"));
    let linux_home = resolve(&[("HOME", "/h")], &[], false);
    assert_eq!(linux_home.mise_cache, p("/h/.cache/mise"));

    let state_xdg = resolve(&[("HOME", "/h"), ("XDG_STATE_HOME", "/xs")], &[], true);
    assert_eq!(state_xdg.mise_state, p("/xs/mise"));
    let state_home = resolve(&[("HOME", "/h")], &[], true);
    assert_eq!(state_home.mise_state, p("/h/.local/state/mise"));
}

#[test]
fn with_no_brew_variable_and_no_fixed_location_there_are_no_brew_paths() {
    let none = resolve(&[("HOME", "/h")], &[], true);
    assert_eq!(none.brew, None);
}

#[test]
fn a_brew_prefix_with_no_repository_gives_no_brew_paths() {
    let prefix_only = resolve(
        &[("HOME", "/h"), ("HOMEBREW_PREFIX", "/b/prefix")],
        &[],
        true,
    );
    assert_eq!(prefix_only.brew, None);
}
