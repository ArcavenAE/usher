//! Story 1.5, PR C0: the `variable` field on `paths` items (which launch variables a
//! harness reads and which path token each moves) and the one lookup every caller
//! uses to key an environment. Public API only.

use usher_knowledge::{Class, KnowledgeFile, Version};

const BINDER: &str = r#"
[[item]]
id = "demo.config-dir-variable"
harness = "demo"
layer = "slow"
class = "paths"
versions = ">=1.0.0, <2.0.0"
summary = "DEMO_CONFIG_DIR relocates the user config home."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/env" }
last_verified = 2026-11-20
variable = { name = "DEMO_CONFIG_DIR", binds = "${CONFIG_DIR}", default = "${HOME}/.demo" }
"#;

const SOURCE: &str = r#"
[[item]]
id = "demo.user-settings"
harness = "demo"
layer = "slow"
class = "sources"
versions = ">=1.0.0, <2.0.0"
summary = "The user settings file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/settings" }
last_verified = 2026-11-20
source = { path = "${CONFIG_DIR}/settings.json", layer = "user", read = "read", rank = 1, always_present = true }
"#;

const VARIABLE: &str = r#"variable = { name = "DEMO_CONFIG_DIR", binds = "${CONFIG_DIR}", default = "${HOME}/.demo" }"#;

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

fn with_variable(variable: &str) -> String {
    file(&[&BINDER.replace(VARIABLE, variable), SOURCE])
}

fn version(text: &str) -> Version {
    Version::parse(text).expect("a version")
}

#[test]
fn a_paths_item_names_a_variable_the_token_it_moves_and_its_default() {
    let parsed = KnowledgeFile::parse(&file(&[BINDER, SOURCE])).expect("a valid file");
    let item = &parsed.items()[0];
    assert_eq!(item.class(), Class::Paths);
    let variable = item.variable().expect("the item names a variable");
    assert_eq!(variable.name(), "DEMO_CONFIG_DIR");
    assert_eq!(variable.binds(), Some("${CONFIG_DIR}"));
    assert_eq!(variable.default(), Some("${HOME}/.demo"));
    assert!(parsed.items()[1].variable().is_none());
}

#[test]
fn a_variable_may_bind_nothing() {
    let text = with_variable(r#"variable = { name = "DEMO_FLAG" }"#);
    let rejected = KnowledgeFile::parse(&text);
    // The source now has no binder, so the file is refused for that reason alone.
    let err = rejected.expect_err("an unbound ${CONFIG_DIR} source");
    assert!(err.to_string().contains("demo.user-settings"), "{err}");
    let alone = file(&[&BINDER.replace(VARIABLE, r#"variable = { name = "DEMO_FLAG" }"#)]);
    let parsed = KnowledgeFile::parse(&alone).expect("a variable that binds nothing");
    let variable = parsed.items()[0].variable().expect("a variable");
    assert_eq!(variable.name(), "DEMO_FLAG");
    assert_eq!(variable.binds(), None);
    assert_eq!(variable.default(), None);
}

#[test]
fn only_a_paths_item_may_carry_a_variable() {
    let text = file(&[
        BINDER,
        &SOURCE.replace(
            "rank = 1, always_present = true }",
            "rank = 1, always_present = true }\nvariable = { name = \"DEMO_X\" }",
        ),
    ]);
    rejects(&text, &["demo.user-settings", "variable", "paths"]);
}

#[test]
fn a_variable_name_is_upper_case_letters_digits_and_underscores() {
    for bad in [
        "demo_config_dir",
        "DEMO_dir",
        "1DEMO",
        "_DEMO",
        "DEMO-DIR",
        "DEMO DIR",
        "",
    ] {
        let text = with_variable(&format!(
            r#"variable = {{ name = "{bad}", binds = "${{CONFIG_DIR}}", default = "${{HOME}}/.demo" }}"#
        ));
        rejects(&text, &["demo.config-dir-variable", "name"]);
    }
}

#[test]
fn a_name_may_hold_digits_after_its_first_letter() {
    let text = with_variable(
        r#"variable = { name = "DEMO2_DIR", binds = "${CONFIG_DIR}", default = "${HOME}/.demo" }"#,
    );
    let parsed = KnowledgeFile::parse(&text).expect("DEMO2_DIR is a valid name");
    assert_eq!(
        parsed.items()[0].variable().map(|v| v.name()),
        Some("DEMO2_DIR")
    );
}

#[test]
fn a_source_without_the_config_dir_token_needs_no_binder() {
    let source = SOURCE.replace("${CONFIG_DIR}/settings.json", "${HOME}/.demo.json");
    let parsed = KnowledgeFile::parse(&file(&[&source])).expect("no binder is needed");
    assert_eq!(parsed.items().len(), 1);
}

#[test]
fn only_the_config_dir_token_may_be_bound() {
    for token in [
        "${HOME}",
        "${USER}",
        "${REPO}",
        "${LAUNCH_DIR}",
        "${VERSION}",
        "${CONFIG_DIR}/x",
        "${ELSEWHERE}",
        "/x",
    ] {
        let text = with_variable(&format!(
            r#"variable = {{ name = "DEMO_CONFIG_DIR", binds = "{token}", default = "${{HOME}}/.demo" }}"#
        ));
        rejects(&text, &["demo.config-dir-variable", token]);
    }
}

#[test]
fn a_default_comes_with_a_bound_token_and_never_without_one() {
    rejects(
        &with_variable(r#"variable = { name = "DEMO_CONFIG_DIR", binds = "${CONFIG_DIR}" }"#),
        &["demo.config-dir-variable", "default"],
    );
    rejects(
        &with_variable(r#"variable = { name = "DEMO_CONFIG_DIR", default = "${HOME}/.demo" }"#),
        &["demo.config-dir-variable", "default"],
    );
}

#[test]
fn a_default_is_an_absolute_path_that_names_neither_its_own_token_nor_the_version() {
    for (default, needle) in [
        ("demo", "absolute"),
        ("${CONFIG_DIR}/x", "${CONFIG_DIR}"),
        ("${HOME}/${VERSION}", "${VERSION}"),
        ("${ELSEWHERE}/x", "${ELSEWHERE}"),
    ] {
        let text = with_variable(&format!(
            r#"variable = {{ name = "DEMO_CONFIG_DIR", binds = "${{CONFIG_DIR}}", default = "{default}" }}"#
        ));
        rejects(&text, &["demo.config-dir-variable", needle]);
    }
}

#[test]
fn two_items_of_one_harness_may_not_bind_one_token_over_overlapping_versions() {
    let overlapping = BINDER
        .replace("demo.config-dir-variable", "demo.second")
        .replace(">=1.0.0, <2.0.0", ">=1.5.0, <3.0.0");
    rejects(
        &file(&[BINDER, &overlapping, SOURCE]),
        &["demo.config-dir-variable", "demo.second"],
    );
    let same_token = overlapping.replace("DEMO_CONFIG_DIR", "DEMO_OTHER_DIR");
    rejects(
        &file(&[BINDER, &same_token, SOURCE]),
        &["demo.config-dir-variable", "demo.second", "${CONFIG_DIR}"],
    );
}

#[test]
fn the_same_name_over_different_versions_or_in_another_harness_is_fine() {
    let later = BINDER
        .replace("demo.config-dir-variable", "demo.later")
        .replace(">=1.0.0, <2.0.0", ">=2.0.0, <3.0.0");
    let other = BINDER
        .replace("demo.config-dir-variable", "other.config")
        .replace("harness = \"demo\"", "harness = \"other\"");
    KnowledgeFile::parse(&file(&[BINDER, &later, &other, SOURCE]))
        .expect("disjoint ranges and other harnesses do not clash");
}

#[test]
fn a_source_that_uses_the_config_dir_token_needs_a_binder_for_the_same_versions() {
    rejects(&file(&[SOURCE]), &["demo.user-settings", "${CONFIG_DIR}"]);
    let other_versions = BINDER.replace(">=1.0.0, <2.0.0", ">=1.0.0, <3.0.0");
    rejects(
        &file(&[&other_versions, SOURCE]),
        &["demo.user-settings", "${CONFIG_DIR}", "versions"],
    );
    let other_harness = BINDER.replace("harness = \"demo\"", "harness = \"other\"");
    rejects(
        &file(&[&other_harness, SOURCE]),
        &["demo.user-settings", "${CONFIG_DIR}"],
    );
    KnowledgeFile::parse(&file(&[BINDER, SOURCE])).expect("a binder for the same versions");
}

/// The key's variable list does not move with the harness version: every range counts,
/// the names are sorted, and a name that two ranges share appears once.
#[test]
fn launch_variables_lists_every_range_sorted_and_deduplicated() {
    // Every variable the key is expected to hold binds a token; one variable that binds
    // nothing sits beside them and stays out. The flag is bound in the later range only,
    // so a lookup that reads the first range alone misses it, and it binds the other
    // token, since two binders of one token may not overlap in versions.
    let flag = BINDER
        .replace("demo.config-dir-variable", "demo.flag")
        .replace(">=1.0.0, <2.0.0", ">=2.0.0, <3.0.0")
        .replace(
            VARIABLE,
            r#"variable = { name = "DEMO_A_FLAG", binds = "${GLOBAL_STATE_DIR}", default = "${HOME}" }"#,
        );
    let plain = BINDER
        .replace("demo.config-dir-variable", "demo.plain")
        .replace(VARIABLE, r#"variable = { name = "DEMO_B_PLAIN" }"#);
    let later = BINDER
        .replace("demo.config-dir-variable", "demo.later")
        .replace(">=1.0.0, <2.0.0", ">=2.0.0, <3.0.0");
    let other = BINDER
        .replace("demo.config-dir-variable", "other.config")
        .replace("harness = \"demo\"", "harness = \"other\"")
        .replace(
            VARIABLE,
            r#"variable = { name = "OTHER_DIR", binds = "${CONFIG_DIR}", default = "${HOME}/.other" }"#,
        );
    let parsed = KnowledgeFile::parse(&file(&[BINDER, &flag, &plain, &later, &other, SOURCE]))
        .expect("valid");
    assert_eq!(
        parsed.launch_variables("demo"),
        ["DEMO_A_FLAG", "DEMO_CONFIG_DIR"],
        "both ranges name DEMO_CONFIG_DIR and it is listed once; DEMO_B_PLAIN binds nothing"
    );
    assert_eq!(parsed.launch_variables("other"), ["OTHER_DIR"]);
    assert!(parsed.launch_variables("nothing").is_empty());
}

/// Token binding stays version-scoped and is never part of the key.
#[test]
fn binding_names_the_variable_and_default_for_a_token_at_a_version() {
    let parsed = KnowledgeFile::parse(&file(&[BINDER, SOURCE])).expect("valid");
    let binding = parsed
        .binding("demo", &version("1.2.0"), "${CONFIG_DIR}")
        .expect("the token is bound at 1.2.0");
    assert_eq!(binding.variable(), "DEMO_CONFIG_DIR");
    assert_eq!(binding.default(), "${HOME}/.demo");
    assert!(
        parsed
            .binding("demo", &version("2.0.0"), "${CONFIG_DIR}")
            .is_none()
    );
    assert!(
        parsed
            .binding("demo", &version("1.2.0"), "${HOME}")
            .is_none()
    );
    assert!(
        parsed
            .binding("other", &version("1.2.0"), "${CONFIG_DIR}")
            .is_none()
    );
}

#[test]
fn the_embedded_seed_names_claude_config_dir_and_binds_the_config_directory() {
    let seed = KnowledgeFile::embedded().expect("the embedded file parses");
    let item = seed
        .items()
        .iter()
        .find(|item| item.id() == "claude-code.config-dir-variable")
        .expect("the config directory item is in the seed");
    let variable = item.variable().expect("it names the variable");
    assert_eq!(variable.name(), "CLAUDE_CONFIG_DIR");
    assert_eq!(variable.binds(), Some("${CONFIG_DIR}"));
    assert_eq!(variable.default(), Some("${HOME}/.claude"));
    assert_eq!(item.last_verified().to_string(), "2026-11-20");
    assert_eq!(seed.launch_variables("claude-code"), ["CLAUDE_CONFIG_DIR"]);
    let binding = seed
        .binding("claude-code", &version("2.1.317"), "${CONFIG_DIR}")
        .expect("the config directory is bound");
    assert_eq!(binding.variable(), "CLAUDE_CONFIG_DIR");
    assert_eq!(binding.default(), "${HOME}/.claude");
    for harness in ["codex", "goose"] {
        assert!(
            seed.items()
                .iter()
                .filter(|item| item.harness() == harness)
                .all(|item| item.variable().is_none()),
            "{harness} stays without the field until its own story"
        );
    }
}
