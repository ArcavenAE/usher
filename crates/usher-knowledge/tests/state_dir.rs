//! Story 1.5, PR D0: a sixth path token, `${GLOBAL_STATE_DIR}`, for the harness's
//! global state file, which one launch variable can move together with the config
//! directory. One variable may bind two tokens; no two binders may bind the same token
//! over versions that overlap. Public API only.

use usher_knowledge::{KnowledgeFile, Version};

const CONFIG: &str = r#"
[[item]]
id = "demo.config-dir"
harness = "demo"
layer = "slow"
class = "paths"
versions = ">=1.0.0, <2.0.0"
summary = "DEMO_HOME relocates the config directory."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/env" }
last_verified = 2026-11-20
variable = { name = "DEMO_HOME", binds = "${CONFIG_DIR}", default = "${HOME}/.demo" }
"#;

const STATE: &str = r#"
[[item]]
id = "demo.global-state-dir"
harness = "demo"
layer = "slow"
class = "paths"
versions = ">=1.0.0, <2.0.0"
summary = "DEMO_HOME also relocates the global state file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/env" }
last_verified = 2026-11-20
variable = { name = "DEMO_HOME", binds = "${GLOBAL_STATE_DIR}", default = "${HOME}" }
"#;

const SOURCE: &str = r#"
[[item]]
id = "demo.global-state"
harness = "demo"
layer = "slow"
class = "sources"
versions = ">=1.0.0, <2.0.0"
summary = "The global state file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/files" }
last_verified = 2026-11-20
source = { path = "${GLOBAL_STATE_DIR}/.demo.json", layer = "global-state", read = "read", rank = 1, always_present = false }
"#;

const STATE_VARIABLE: &str =
    r#"variable = { name = "DEMO_HOME", binds = "${GLOBAL_STATE_DIR}", default = "${HOME}" }"#;

fn file(items: &[&str]) -> String {
    format!("schema_version = 4\nsequence = 9\n{}", items.concat())
}

fn rejects(text: &str, needles: &[&str]) {
    let err = KnowledgeFile::parse(text).expect_err("the check must fail");
    let shown = err.to_string();
    for needle in needles {
        assert!(shown.contains(needle), "`{needle}` not in `{shown}`");
    }
    assert!(!err.step().is_empty() && !err.check().is_empty());
    assert!(!err.status().is_empty() && !err.next_action().is_empty());
}

fn version(text: &str) -> Version {
    Version::parse(text).expect("a version")
}

#[test]
fn one_variable_binds_the_config_directory_and_the_global_state_directory() {
    let parsed = KnowledgeFile::parse(&file(&[CONFIG, STATE, SOURCE])).expect("a valid file");
    let config = parsed
        .binding("demo", &version("1.2.0"), "${CONFIG_DIR}")
        .expect("the config directory is bound");
    assert_eq!(config.variable(), "DEMO_HOME");
    assert_eq!(config.default(), "${HOME}/.demo");
    let state = parsed
        .binding("demo", &version("1.2.0"), "${GLOBAL_STATE_DIR}")
        .expect("the global state directory is bound");
    assert_eq!(state.variable(), "DEMO_HOME");
    assert_eq!(state.default(), "${HOME}");
    assert!(
        parsed
            .binding("demo", &version("2.0.0"), "${GLOBAL_STATE_DIR}")
            .is_none()
    );
    assert!(
        parsed
            .binding("other", &version("1.2.0"), "${GLOBAL_STATE_DIR}")
            .is_none()
    );
}

#[test]
fn the_key_lists_a_variable_that_binds_two_tokens_once() {
    let parsed = KnowledgeFile::parse(&file(&[CONFIG, STATE, SOURCE])).expect("a valid file");
    assert_eq!(parsed.launch_variables("demo"), ["DEMO_HOME"]);
}

#[test]
fn only_the_config_and_global_state_tokens_can_be_bound() {
    for token in ["${HOME}", "${REPO}", "${USER}", "${LAUNCH_DIR}", "${NOPE}"] {
        let bad = STATE.replace("${GLOBAL_STATE_DIR}", token);
        rejects(
            &file(&[CONFIG, &bad]),
            &[
                "demo.global-state-dir",
                token,
                "${GLOBAL_STATE_DIR}",
                "${CONFIG_DIR}",
            ],
        );
    }
}

#[test]
fn no_two_binders_bind_the_same_token_over_overlapping_versions() {
    let same_name = STATE.replace("demo.global-state-dir", "demo.second");
    let other_name = same_name.replace("DEMO_HOME", "DEMO_OTHER");
    for second in [&same_name, &other_name] {
        let text = file(&[CONFIG, STATE, second]);
        rejects(
            &text,
            &[
                "demo.global-state-dir",
                "demo.second",
                "${GLOBAL_STATE_DIR}",
            ],
        );
    }
    let config_twice = CONFIG
        .replace("demo.config-dir", "demo.config-dir-two")
        .replace("DEMO_HOME", "DEMO_ELSEWHERE");
    rejects(
        &file(&[CONFIG, &config_twice]),
        &["demo.config-dir", "demo.config-dir-two", "${CONFIG_DIR}"],
    );
}

#[test]
fn binders_of_one_token_over_disjoint_versions_are_allowed() {
    let later = STATE
        .replace("demo.global-state-dir", "demo.global-state-dir-2")
        .replace(">=1.0.0, <2.0.0", ">=2.0.0, <3.0.0");
    let parsed = KnowledgeFile::parse(&file(&[CONFIG, STATE, &later])).expect("valid");
    assert!(
        parsed
            .binding("demo", &version("2.5.0"), "${GLOBAL_STATE_DIR}")
            .is_some()
    );
}

#[test]
fn a_source_using_the_global_state_token_needs_a_binder_over_the_same_versions() {
    rejects(
        &file(&[CONFIG, SOURCE]),
        &["demo.global-state", "${GLOBAL_STATE_DIR}", "same versions"],
    );
    let narrower = STATE.replace(">=1.0.0, <2.0.0", ">=1.0.0, <1.5.0");
    rejects(
        &file(&[CONFIG, &narrower, SOURCE]),
        &["demo.global-state", "${GLOBAL_STATE_DIR}", "same versions"],
    );
}

#[test]
fn a_binder_of_the_config_directory_does_not_stand_in_for_the_global_state_binder() {
    // The config-directory source is bound and passes; the global-state source beside it
    // has only that binder over its versions, which binds the other token.
    let config_source = SOURCE
        .replace("demo.global-state", "demo.user-settings")
        .replace(
            "${GLOBAL_STATE_DIR}/.demo.json",
            "${CONFIG_DIR}/settings.json",
        )
        .replace("global-state\"", "user\"");
    rejects(
        &file(&[CONFIG, &config_source, SOURCE]),
        &["demo.global-state", "${GLOBAL_STATE_DIR}", "same versions"],
    );
    KnowledgeFile::parse(&file(&[CONFIG, &config_source]))
        .expect("the config source alone is bound");
}

#[test]
fn a_default_may_not_name_the_token_it_is_the_default_of() {
    let bad = STATE.replace(STATE_VARIABLE, r#"variable = { name = "DEMO_HOME", binds = "${GLOBAL_STATE_DIR}", default = "${GLOBAL_STATE_DIR}/x" }"#);
    rejects(
        &file(&[CONFIG, &bad]),
        &["demo.global-state-dir", "${GLOBAL_STATE_DIR}", "default"],
    );
}

#[test]
fn the_new_token_is_a_path_token_everywhere_a_path_is_checked() {
    let seat = r#"
[[item]]
id = "demo.seat-state"
harness = "demo"
layer = "slow"
class = "seat"
versions = ">=1.0.0, <2.0.0"
summary = "Presence of the global state file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/files" }
last_verified = 2026-11-20
seat = { part = "mcp", path = "${GLOBAL_STATE_DIR}/.demo.json", read = "names", key_paths = ["mcpServers.*"] }
"#;
    KnowledgeFile::parse(&file(&[CONFIG, STATE, SOURCE, seat])).expect("a seat path may use it");
}
