//! Story 1.5, PR C0: `always_present` on `sources` items (which layers the listing
//! names when they have no file) and `version_from` on `paths` items (how the harness
//! version is read from disk, never by running the harness). Public API only.

use usher_knowledge::{Class, KnowledgeFile, ReferenceKind, SourceLayer, VersionRule};

const FILE: &str = r#"
schema_version = 5
sequence = 9

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

[[item]]
id = "demo.launcher"
harness = "demo"
layer = "slow"
class = "paths"
versions = ">=1.0.0, <2.0.0"
summary = "The launcher link whose target names the installed version."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/install" }
last_verified = 2026-11-20
version_from = { launcher = "${HOME}/.local/bin/demo", rule = "symlink-target-last-component", layout = "${HOME}/.local/share/demo/versions/${VERSION}" }

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

const SOURCE: &str = r#"source = { path = "${CONFIG_DIR}/settings.json", layer = "user", read = "read", rank = 1, always_present = true }"#;
const LINK: &str = r#"version_from = { launcher = "${HOME}/.local/bin/demo", rule = "symlink-target-last-component", layout = "${HOME}/.local/share/demo/versions/${VERSION}" }"#;

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
    FILE.replace(SOURCE, source)
}

fn with_link(link: &str) -> String {
    FILE.replace(LINK, link)
}

#[test]
fn a_source_says_whether_its_layer_is_always_present() {
    let file = KnowledgeFile::parse(FILE).expect("a valid schema 4 file");
    let source = file.items()[0].source().expect("a source fact");
    assert!(source.always_present());

    let text = with_source(&SOURCE.replace("always_present = true", "always_present = false"));
    let file = KnowledgeFile::parse(&text).expect("false is a value too");
    let source = file.items()[0].source().expect("a source fact");
    assert!(!source.always_present());
}

#[test]
fn a_source_without_always_present_is_refused_naming_the_item() {
    let text = with_source(&SOURCE.replace(", always_present = true", ""));
    rejects(&text, &["demo.user-settings", "always_present"]);
}

#[test]
fn always_present_must_be_a_boolean() {
    let text = with_source(&SOURCE.replace("always_present = true", "always_present = \"yes\""));
    rejects(&text, &["demo.user-settings", "always_present", "boolean"]);
}

#[test]
fn a_paths_item_carries_the_launcher_rule_and_layout_for_the_version() {
    let file = KnowledgeFile::parse(FILE).expect("a valid schema 4 file");
    let item = &file.items()[1];
    assert_eq!(item.class(), Class::Paths);
    let from = item
        .version_from()
        .expect("a paths item may say how to read the version");
    assert_eq!(from.launcher(), "${HOME}/.local/bin/demo");
    assert_eq!(from.rule(), VersionRule::SymlinkTargetLastComponent);
    assert_eq!(
        from.layout(),
        "${HOME}/.local/share/demo/versions/${VERSION}"
    );
    assert!(
        file.items()[0].version_from().is_none(),
        "an item that does not say has none"
    );
}

#[test]
fn only_a_paths_item_may_carry_version_from() {
    let text = FILE.replace(SOURCE, &format!("{SOURCE}\n{LINK}"));
    rejects(&text, &["demo.user-settings", "version_from", "paths"]);
}

#[test]
fn an_unknown_rule_is_refused_naming_it() {
    let text = with_link(&LINK.replace("symlink-target-last-component", "run-the-harness"));
    rejects(&text, &["demo.launcher", "run-the-harness"]);
}

#[test]
fn the_launcher_must_be_an_absolute_path_in_the_known_tokens() {
    rejects(
        &with_link(&LINK.replace("${HOME}/.local/bin/demo", "bin/demo")),
        &["demo.launcher", "absolute"],
    );
    rejects(
        &with_link(&LINK.replace("${HOME}/.local/bin/demo", "${ELSEWHERE}/demo")),
        &["demo.launcher", "${ELSEWHERE}"],
    );
}

#[test]
fn a_missing_launcher_rule_or_layout_is_refused_naming_the_field() {
    for (cut, field) in [
        (r#"launcher = "${HOME}/.local/bin/demo", "#, "launcher"),
        (r#"rule = "symlink-target-last-component", "#, "rule"),
        (
            r#", layout = "${HOME}/.local/share/demo/versions/${VERSION}""#,
            "layout",
        ),
    ] {
        rejects(
            &with_link(&LINK.replace(cut, "")),
            &["demo.launcher", field],
        );
    }
}

/// `${VERSION}` is a capture placeholder, not a sixth path token: it stands for the
/// last component of `layout` and nowhere else.
#[test]
fn version_is_a_placeholder_held_once_as_the_last_component_of_the_layout() {
    let layout = "${HOME}/.local/share/demo/versions/${VERSION}";
    for (bad, why) in [
        ("${HOME}/.local/share/demo/versions", "no placeholder"),
        (
            "${HOME}/.local/share/${VERSION}/versions/${VERSION}",
            "twice",
        ),
        ("${HOME}/.local/share/demo/${VERSION}/bin", "not last"),
        (
            "${HOME}/.local/share/demo/versions/v${VERSION}",
            "part of a component",
        ),
    ] {
        let text = with_link(&LINK.replace(layout, bad));
        let err =
            KnowledgeFile::parse(&text).expect_err(&format!("a layout with {why} must be refused"));
        let shown = err.to_string();
        assert!(
            shown.contains("demo.launcher") && shown.contains("${VERSION}"),
            "{why}: `{shown}`"
        );
    }
}

#[test]
fn an_unknown_token_in_the_layout_is_refused_naming_it() {
    let text = with_link(&LINK.replace("${HOME}/.local/share/demo", "${ELSEWHERE}/demo"));
    rejects(&text, &["demo.launcher", "${ELSEWHERE}"]);
}

#[test]
fn a_relative_layout_is_refused() {
    let text = with_link(&LINK.replace("${HOME}/.local/share/demo/versions/", "demo/versions/"));
    rejects(&text, &["demo.launcher", "layout", "absolute"]);
}

/// A layout must start at a root: `/` or one of the five path tokens. `${VERSION}` is a
/// capture placeholder, never a root, so a layout that is only the placeholder is relative.
#[test]
fn a_layout_that_is_only_the_version_placeholder_is_refused_as_relative() {
    let text = with_link(&LINK.replace(
        "${HOME}/.local/share/demo/versions/${VERSION}",
        "${VERSION}",
    ));
    rejects(&text, &["demo.launcher", "layout", "absolute"]);
}

#[test]
fn version_outside_the_layout_is_refused() {
    rejects(
        &with_link(&LINK.replace("${HOME}/.local/bin/demo", "${HOME}/${VERSION}/demo")),
        &["demo.launcher", "${VERSION}"],
    );
    rejects(
        &with_source(&SOURCE.replace(
            "${CONFIG_DIR}/settings.json",
            "${CONFIG_DIR}/${VERSION}.json",
        )),
        &["demo.user-settings", "${VERSION}"],
    );
}

#[test]
fn an_empty_layout_is_refused() {
    let text = with_link(&LINK.replace("${HOME}/.local/share/demo/versions/${VERSION}", ""));
    rejects(&text, &["demo.launcher", "layout", "empty"]);
}

#[test]
fn the_embedded_seed_marks_user_project_and_local_always_present_and_the_rest_not() {
    let file = KnowledgeFile::embedded().expect("the embedded file parses");
    let mut seen = Vec::new();
    for item in file.items().iter().filter(|i| i.harness() == "claude-code") {
        let Some(source) = item.source() else {
            continue;
        };
        let expected = matches!(
            source.layer(),
            SourceLayer::User | SourceLayer::Project | SourceLayer::Local
        );
        assert_eq!(
            source.always_present(),
            expected,
            "{} ({:?})",
            item.id(),
            source.layer()
        );
        seen.push(source.layer());
    }
    for layer in [
        SourceLayer::Managed,
        SourceLayer::User,
        SourceLayer::Project,
        SourceLayer::Local,
        SourceLayer::GlobalState,
        SourceLayer::ServerManagedCache,
    ] {
        assert!(seen.contains(&layer), "no Claude Code source for {layer:?}");
    }
}

#[test]
fn the_embedded_seed_reads_the_claude_code_version_from_the_native_launcher_link() {
    let file = KnowledgeFile::embedded().expect("the embedded file parses");
    let item = file
        .items()
        .iter()
        .find(|i| i.id() == "claude-code.native-launcher")
        .expect("the native launcher item is in the seed");
    assert_eq!(item.class(), Class::Paths);
    let from = item.version_from().expect("checked above");
    assert_eq!(item.harness(), "claude-code");
    assert_eq!(from.launcher(), "${HOME}/.local/bin/claude");
    assert_eq!(from.rule(), VersionRule::SymlinkTargetLastComponent);
    assert_eq!(
        from.layout(),
        "${HOME}/.local/share/claude/versions/${VERSION}"
    );
    assert_eq!(item.reference().kind(), ReferenceKind::VendorDocs);
    assert_eq!(
        item.reference().locator(),
        "https://code.claude.com/docs/en/setup"
    );
}
