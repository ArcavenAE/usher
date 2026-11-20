//! Story 1.5, PR C0: the `keys` field of class `keys` items, schema 3 (AD-12, AD-6),
//! and the managed-tier control keys that first-wins needs. Runs against the public
//! API only.

use usher_knowledge::{Class, KnowledgeFile, Layer};

const FILE: &str = r#"
schema_version = 3
sequence = 9

[[item]]
id = "demo.control-keys"
harness = "demo"
layer = "fast"
class = "keys"
versions = ">=1.0.0, <2.0.0"
summary = "Keys that control how managed sources combine, so a file that holds only these delivers no policy."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/managed" }
last_verified = 2026-11-20
keys = ["beta", "alpha"]
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

#[test]
fn a_keys_item_parses_with_its_key_names_in_file_order() {
    let file = KnowledgeFile::parse(FILE).expect("a valid schema 3 file");
    assert_eq!(file.version().schema_version(), 3);
    let item = &file.items()[0];
    assert_eq!(item.class(), Class::Keys);
    assert_eq!(item.layer(), Layer::Fast);
    // Not sorted in the file: an implementation that sorts would fail here.
    assert_eq!(item.keys(), ["beta", "alpha"]);
}

#[test]
fn schema_two_is_refused_and_schema_three_is_read() {
    let two = FILE.replace("schema_version = 3", "schema_version = 2");
    rejects(&two, &["schema 2 is not supported", "reads schema 3"]);
    KnowledgeFile::parse("schema_version = 3\nsequence = 1\n")
        .expect("schema 3 with no items is valid");
}

#[test]
fn a_keys_item_without_a_keys_list_is_refused_naming_the_item() {
    let text = FILE.replace("keys = [\"beta\", \"alpha\"]\n", "");
    rejects(&text, &["demo.control-keys", "keys"]);
}

#[test]
fn an_empty_keys_list_is_refused_naming_the_item() {
    let text = FILE.replace("[\"beta\", \"alpha\"]", "[]");
    rejects(&text, &["demo.control-keys", "empty"]);
}

#[test]
fn a_key_name_must_be_a_non_empty_string() {
    rejects(
        &FILE.replace("[\"beta\", \"alpha\"]", "[\"beta\", \"\"]"),
        &["demo.control-keys", "empty"],
    );
    rejects(
        &FILE.replace("[\"beta\", \"alpha\"]", "[\"beta\", 3]"),
        &["demo.control-keys", "string"],
    );
}

#[test]
fn a_key_name_padded_with_spaces_is_refused_naming_the_item() {
    for padded in ["[\"beta\", \" alpha\"]", "[\"beta\", \"alpha \"]"] {
        rejects(
            &FILE.replace("[\"beta\", \"alpha\"]", padded),
            &["demo.control-keys", "padded"],
        );
    }
}

#[test]
fn a_repeated_key_name_is_refused_naming_it() {
    rejects(
        &FILE.replace("[\"beta\", \"alpha\"]", "[\"beta\", \"alpha\", \"beta\"]"),
        &["demo.control-keys", "beta"],
    );
}

#[test]
fn only_a_keys_item_may_carry_a_keys_list() {
    let text = FILE.replace(
        "layer = \"fast\"\nclass = \"keys\"",
        "layer = \"fast\"\nclass = \"defaults\"",
    );
    rejects(&text, &["demo.control-keys", "defaults", "keys"]);
}

#[test]
fn the_embedded_seed_names_the_claude_code_managed_control_keys() {
    let file = KnowledgeFile::embedded().expect("the embedded file parses");
    assert_eq!(file.version().schema_version(), 3);
    let item = file
        .items()
        .iter()
        .find(|item| item.id() == "claude-code.managed-control-keys")
        .expect("the control-key item is in the seed");
    assert_eq!(item.class(), Class::Keys);
    assert_eq!(item.harness(), "claude-code");
    let mut names: Vec<&str> = item.keys().iter().map(String::as_str).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "managedSourcesBehavior",
            "policyHelper",
            "wslInheritsWindowsSettings"
        ],
        "exactly the three delivery controls, no more"
    );
    assert!(item.reference().locator().contains("managed-settings"));
}
