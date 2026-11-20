//! Story 1.5, PR C0: `platforms` on a source (which operating systems read it). A
//! missing field means every platform. Public API only.

use usher_knowledge::{KnowledgeFile, Platform};

const SOURCE: &str = r#"source = { path = "/etc/demo/managed.json", layer = "managed", read = "first-wins", rank = 3, always_present = false }"#;

fn file(source: &str) -> String {
    format!(
        r#"
schema_version = 3
sequence = 9

[[item]]
id = "demo.managed"
harness = "demo"
layer = "slow"
class = "sources"
versions = ">=1.0.0, <2.0.0"
summary = "The managed settings file."
reference = {{ kind = "vendor-docs", locator = "https://example.test/docs/managed" }}
last_verified = 2026-11-20
{source}
"#
    )
}

fn with_platforms(list: &str) -> String {
    file(&SOURCE.replace(
        "always_present = false }",
        &format!("always_present = false, platforms = {list} }}"),
    ))
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

#[test]
fn a_source_names_the_platforms_that_read_it() {
    let parsed = KnowledgeFile::parse(&with_platforms(r#"["macos", "linux"]"#)).expect("valid");
    let source = parsed.items()[0].source().expect("a source");
    assert_eq!(
        source.platforms(),
        Some(&[Platform::Macos, Platform::Linux][..])
    );
    let all = KnowledgeFile::parse(&with_platforms(r#"["windows"]"#)).expect("valid");
    assert_eq!(
        all.items()[0].source().expect("a source").platforms(),
        Some(&[Platform::Windows][..])
    );
}

#[test]
fn a_missing_platforms_field_means_every_platform() {
    let parsed = KnowledgeFile::parse(&file(SOURCE)).expect("valid");
    assert_eq!(
        parsed.items()[0].source().expect("a source").platforms(),
        None
    );
}

#[test]
fn an_empty_platforms_list_is_refused_naming_the_item() {
    rejects(
        &with_platforms("[]"),
        &["demo.managed", "platforms", "empty"],
    );
}

#[test]
fn a_repeated_platform_is_refused_naming_it() {
    rejects(
        &with_platforms(r#"["linux", "macos", "linux"]"#),
        &["demo.managed", "platforms", "linux"],
    );
}

#[test]
fn a_platform_outside_the_three_is_refused_naming_it() {
    rejects(
        &with_platforms(r#"["macos", "freebsd"]"#),
        &["demo.managed", "freebsd"],
    );
    rejects(
        &with_platforms(r#"["macos", 3]"#),
        &["demo.managed", "platforms", "string"],
    );
}

#[test]
fn the_embedded_seed_reads_each_managed_file_on_its_own_platform_only() {
    let seed = KnowledgeFile::embedded().expect("the embedded file parses");
    let platforms = |id: &str| {
        seed.items()
            .iter()
            .find(|item| item.id() == id)
            .and_then(|item| item.source())
            .unwrap_or_else(|| panic!("{id} is in the seed"))
            .platforms()
            .map(<[Platform]>::to_vec)
    };
    assert_eq!(
        platforms("claude-code.source-managed-file"),
        Some(vec![Platform::Macos])
    );
    assert_eq!(
        platforms("claude-code.source-managed-file-linux"),
        Some(vec![Platform::Linux])
    );
    for id in [
        "claude-code.source-user-settings",
        "claude-code.source-project-settings",
        "claude-code.source-local-settings",
        "claude-code.source-global-state",
        "claude-code.source-server-managed-cache",
    ] {
        assert_eq!(platforms(id), None, "{id} is read on every platform");
    }
}
