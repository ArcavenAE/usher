//! Story 1.5, PR D: the file a `keys-present` item reads must be named by an anchor over
//! the same versions and on every platform the item reads on. Overlap alone would let the
//! item read a file at a version or on a platform where nothing makes it a known JSON file.
//! Public API only.

use usher_knowledge::KnowledgeFile;

const HEADER: &str = "schema_version = 4\nsequence = 11\n";

fn names(versions: &str, platforms: &str) -> String {
    ITEM_NAMES
        .replace("__V__", versions)
        .replace("__P__", platforms)
}

fn keys(versions: &str, platforms: &str) -> String {
    ITEM_KEYS
        .replace("__V__", versions)
        .replace("__P__", platforms)
}

const ITEM_NAMES: &str = r#"
[[item]]
id = "demo.seat-anchor"
harness = "demo"
layer = "slow"
class = "seat"
versions = "__V__"
summary = "Server names."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/x" }
last_verified = 2026-11-20
seat = { part = "mcp", path = "${HOME}/.x.json", read = "names", key_paths = ["mcpServers.*"]__P__ }
"#;

const ITEM_KEYS: &str = r#"
[[item]]
id = "demo.seat-ask"
harness = "demo"
layer = "slow"
class = "seat"
versions = "__V__"
summary = "A login marker."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/y" }
last_verified = 2026-11-20
seat = { part = "auth-route", path = "${HOME}/.x.json", read = "keys-present", keys = ["oauthAccount"]__P__ }
"#;

fn parse(anchor: &str, ask: &str) -> Result<KnowledgeFile, usher_core::UsherError> {
    KnowledgeFile::parse(&format!("{HEADER}{anchor}{ask}"))
}

#[test]
fn an_anchor_over_a_narrower_range_is_refused_naming_versions() {
    let anchor = names(">=1.0.0, <1.5.0", "");
    let ask = keys(">=1.0.0, <2.0.0", "");
    let err = parse(&anchor, &ask).expect_err("a narrower anchor is refused");
    let shown = err.to_string();
    assert!(shown.contains("versions"), "{shown}");
    assert!(shown.contains("demo.seat-ask"), "{shown}");
    assert!(!err.step().is_empty() && !err.check().is_empty());
    assert!(!err.status().is_empty() && !err.next_action().is_empty());
}

#[test]
fn an_anchor_on_fewer_platforms_is_refused_naming_platforms() {
    let anchor = names(">=1.0.0, <2.0.0", r#", platforms = ["macos"]"#);
    let ask = keys(">=1.0.0, <2.0.0", "");
    let err = parse(&anchor, &ask).expect_err("a narrower platform is refused");
    let shown = err.to_string();
    assert!(shown.contains("platforms"), "{shown}");
    assert!(shown.contains("demo.seat-ask"), "{shown}");
}

#[test]
fn an_anchor_on_other_platforms_than_the_item_is_refused_naming_platforms() {
    let anchor = names(">=1.0.0, <2.0.0", r#", platforms = ["macos"]"#);
    let ask = keys(">=1.0.0, <2.0.0", r#", platforms = ["linux"]"#);
    let err =
        parse(&anchor, &ask).expect_err("an anchor that omits the item's platform is refused");
    let shown = err.to_string();
    assert!(shown.contains("platforms"), "{shown}");
    assert!(shown.contains("demo.seat-ask"), "{shown}");
}

#[test]
fn an_anchor_that_covers_only_one_of_the_items_platforms_is_refused_naming_platforms() {
    let anchor = names(">=1.0.0, <2.0.0", r#", platforms = ["macos"]"#);
    let ask = keys(">=1.0.0, <2.0.0", r#", platforms = ["macos", "linux"]"#);
    let err = parse(&anchor, &ask).expect_err("an anchor that omits one platform is refused");
    let shown = err.to_string();
    assert!(shown.contains("platforms"), "{shown}");
    assert!(shown.contains("demo.seat-ask"), "{shown}");
}

#[test]
fn an_anchor_over_the_same_versions_in_another_term_order_is_accepted() {
    let anchor = names("<2.0.0, >=1.0.0", "");
    let ask = keys(">=1.0.0, <2.0.0", "");
    if let Err(err) = parse(&anchor, &ask) {
        panic!("expected a valid file: {err}");
    }
}

#[test]
fn an_anchor_with_equal_versions_and_no_platforms_is_accepted() {
    let anchor = names(">=1.0.0, <2.0.0", "");
    let ask = keys(">=1.0.0, <2.0.0", "");
    if let Err(err) = parse(&anchor, &ask) {
        panic!("expected a valid file: {err}");
    }
}

#[test]
fn an_anchor_whose_platforms_include_every_platform_of_the_item_is_accepted() {
    let anchor = names(">=1.0.0, <2.0.0", r#", platforms = ["macos", "linux"]"#);
    let ask = keys(">=1.0.0, <2.0.0", r#", platforms = ["linux"]"#);
    if let Err(err) = parse(&anchor, &ask) {
        panic!("expected a valid file: {err}");
    }
}

#[test]
fn the_embedded_seed_parses_under_the_rule() {
    KnowledgeFile::embedded().expect("the embedded file parses");
}
