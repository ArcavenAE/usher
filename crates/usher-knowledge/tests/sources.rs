//! Story 1.5, PR A: the `sources` knowledge class (AD-12, AD-6), schema 2, and the
//! Claude Code seed entries. Runs against the public API only.

use usher_knowledge::{Class, KnowledgeFile, Layer, ReadRule, SourceLayer, Version};

const FILE: &str = r#"
schema_version = 2
sequence = 9

[[item]]
id = "demo.source.user-settings"
harness = "demo"
layer = "slow"
class = "sources"
versions = ">=1.0.0, <2.0.0"
summary = "The user settings file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/settings" }
last_verified = 2026-11-02
source = { path = "${CONFIG_DIR}/settings.json", layer = "user", read = "read", rank = 1 }
"#;

fn rejects(text: &str, needles: &[&str]) {
    let err = KnowledgeFile::parse(text).expect_err("the check must fail");
    let shown = err.to_string();
    for needle in needles {
        assert!(shown.contains(needle), "`{needle}` not in `{shown}`");
    }
    assert!(!err.step().is_empty() && !err.check().is_empty());
    assert!(!err.status().is_empty() && !err.next_action().is_empty());
}

fn with_source(source: &str) -> String {
    FILE.replace(
        r#"source = { path = "${CONFIG_DIR}/settings.json", layer = "user", read = "read", rank = 1 }"#,
        source,
    )
}

#[test]
fn a_sources_item_parses_with_path_layer_read_rule_and_rank() {
    let file = KnowledgeFile::parse(FILE).expect("a valid schema 2 file");
    assert_eq!(file.version().schema_version(), 2);
    let item = &file.items()[0];
    assert_eq!(item.class(), Class::Sources);
    assert_eq!(item.layer(), Layer::Slow);
    let source = item.source().expect("a sources item carries its source fact");
    assert_eq!(source.path(), "${CONFIG_DIR}/settings.json");
    assert_eq!(source.layer(), SourceLayer::User);
    assert_eq!(source.read(), ReadRule::Read);
    assert_eq!(source.rank(), 1);
}

#[test]
fn every_layer_and_read_rule_name_is_accepted() {
    for (name, layer) in [
        ("managed", SourceLayer::Managed),
        ("user", SourceLayer::User),
        ("project", SourceLayer::Project),
        ("local", SourceLayer::Local),
        ("global-state", SourceLayer::GlobalState),
        ("server-managed-cache", SourceLayer::ServerManagedCache),
    ] {
        let text = with_source(&format!(
            r#"source = {{ path = "${{HOME}}/x", layer = "{name}", read = "read", rank = 1 }}"#
        ));
        let file = KnowledgeFile::parse(&text).unwrap_or_else(|e| panic!("layer {name}: {e}"));
        assert_eq!(file.items()[0].source().map(|s| s.layer()), Some(layer));
    }
    for (name, rule) in [
        ("read", ReadRule::Read),
        ("not-read", ReadRule::NotRead),
        ("first-wins", ReadRule::FirstWins),
    ] {
        let text = with_source(&format!(
            r#"source = {{ path = "${{HOME}}/x", layer = "managed", read = "{name}", rank = 2 }}"#
        ));
        let file = KnowledgeFile::parse(&text).unwrap_or_else(|e| panic!("rule {name}: {e}"));
        assert_eq!(file.items()[0].source().map(|s| s.read()), Some(rule));
    }
}

#[test]
fn schema_one_is_refused_and_schema_two_is_read() {
    let one = FILE.replace("schema_version = 2", "schema_version = 1");
    rejects(&one, &["schema 1 is not supported", "reads schema 2"]);
    let two_without_sources = r#"
schema_version = 2
sequence = 1
"#;
    KnowledgeFile::parse(two_without_sources).expect("schema 2 with no items is valid");
}

#[test]
fn all_four_path_tokens_are_accepted() {
    let text = with_source(
        r#"source = { path = "${HOME}/a/${USER}/${CONFIG_DIR}/b/${REPO}/c", layer = "project", read = "read", rank = 1 }"#,
    );
    KnowledgeFile::parse(&text).expect("the four tokens of the path convention");
}

#[test]
fn an_absolute_path_with_no_token_is_accepted() {
    let text = with_source(
        r#"source = { path = "/etc/demo/managed.json", layer = "managed", read = "first-wins", rank = 1 }"#,
    );
    KnowledgeFile::parse(&text).expect("an absolute path");
}

#[test]
fn a_token_outside_the_four_is_refused_naming_the_item_and_the_token() {
    let text = with_source(
        r#"source = { path = "${TMPDIR}/settings.json", layer = "user", read = "read", rank = 1 }"#,
    );
    rejects(&text, &["demo.source.user-settings", "${TMPDIR}"]);
}

#[test]
fn an_unterminated_token_is_refused() {
    let text = with_source(
        r#"source = { path = "${HOME/settings.json", layer = "user", read = "read", rank = 1 }"#,
    );
    rejects(&text, &["demo.source.user-settings", "${HOME"]);
}

#[test]
fn a_relative_path_is_refused() {
    let text = with_source(
        r#"source = { path = "settings.json", layer = "user", read = "read", rank = 1 }"#,
    );
    rejects(&text, &["demo.source.user-settings", "settings.json"]);
}

#[test]
fn a_sources_item_without_a_source_table_is_refused_naming_the_item() {
    let text = FILE.replace(
        r#"source = { path = "${CONFIG_DIR}/settings.json", layer = "user", read = "read", rank = 1 }"#,
        "",
    );
    rejects(&text, &["demo.source.user-settings", "source"]);
}

#[test]
fn a_source_table_on_an_item_of_another_class_is_refused() {
    let text = FILE.replace(r#"class = "sources""#, r#"class = "paths""#);
    rejects(&text, &["demo.source.user-settings", "source"]);
}

#[test]
fn the_sources_class_belongs_to_the_slow_layer() {
    let text = FILE.replace(r#"layer = "slow""#, r#"layer = "fast""#);
    rejects(&text, &["demo.source.user-settings", "other layer"]);
}

#[test]
fn an_unknown_layer_read_rule_or_key_is_refused() {
    rejects(
        &with_source(r#"source = { path = "${HOME}/x", layer = "enterprise", read = "read", rank = 1 }"#),
        &["demo.source.user-settings", "enterprise"],
    );
    rejects(
        &with_source(r#"source = { path = "${HOME}/x", layer = "user", read = "sometimes", rank = 1 }"#),
        &["demo.source.user-settings", "sometimes"],
    );
    rejects(
        &with_source(r#"source = { path = "${HOME}/x", layer = "user", read = "read", rank = 1, mode = "0600" }"#),
        &["demo.source.user-settings", "mode"],
    );
}

#[test]
fn a_rank_below_one_or_missing_is_refused() {
    rejects(
        &with_source(r#"source = { path = "${HOME}/x", layer = "user", read = "read", rank = 0 }"#),
        &["demo.source.user-settings", "rank"],
    );
    rejects(
        &with_source(r#"source = { path = "${HOME}/x", layer = "user", read = "read" }"#),
        &["demo.source.user-settings", "rank"],
    );
}

#[test]
fn the_embedded_file_is_schema_two_and_lists_every_claude_code_source() {
    let file = KnowledgeFile::embedded().expect("the embedded file passes the check");
    assert_eq!(file.version().schema_version(), 2);
    assert!(file.version().sequence() >= 2, "the new items raise the sequence");
    let installed = Version::parse("2.1.317").expect("a version");
    let sources: Vec<_> = file
        .items_for("claude-code", &installed)
        .filter(|i| i.class() == Class::Sources)
        .collect();
    for layer in [
        SourceLayer::Managed,
        SourceLayer::User,
        SourceLayer::Project,
        SourceLayer::Local,
        SourceLayer::GlobalState,
        SourceLayer::ServerManagedCache,
    ] {
        assert!(
            sources
                .iter()
                .any(|i| i.source().is_some_and(|s| s.layer() == layer)),
            "no Claude Code source item for layer {layer:?}"
        );
    }
    for item in &sources {
        let source = item.source().expect("a sources item carries its source fact");
        assert!(
            source.path().starts_with("${") || source.path().starts_with('/'),
            "{}: {}",
            item.id(),
            source.path()
        );
        assert!(!item.reference().locator().is_empty());
    }
}

#[test]
fn the_embedded_managed_sources_say_which_wins() {
    let file = KnowledgeFile::embedded().expect("the embedded file passes the check");
    let installed = Version::parse("2.1.317").expect("a version");
    let managed: Vec<_> = file
        .items_for("claude-code", &installed)
        .filter_map(|i| i.source())
        .filter(|s| s.layer() == SourceLayer::Managed)
        .collect();
    assert!(!managed.is_empty(), "no managed Claude Code source");
    assert!(
        managed.iter().any(|s| s.read() == ReadRule::FirstWins),
        "the managed tier is first-wins in the research map"
    );
}

#[test]
fn the_embedded_user_config_directory_source_moves_with_claude_config_dir() {
    let file = KnowledgeFile::embedded().expect("the embedded file passes the check");
    let installed = Version::parse("2.1.317").expect("a version");
    let user = file
        .items_for("claude-code", &installed)
        .filter_map(|i| i.source())
        .find(|s| s.layer() == SourceLayer::User)
        .expect("a user source");
    assert!(
        user.path().contains("${CONFIG_DIR}"),
        "CLAUDE_CONFIG_DIR relocates the user home, so the path must use the token: {}",
        user.path()
    );
}
