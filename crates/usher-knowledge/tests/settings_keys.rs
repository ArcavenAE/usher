//! Story 1.6, PR A: schema 5, the `settings-keys` class and the cross-layer stack.
//! A settings-key item names one key a harness reads, how it merges across layers, whether
//! its value is never read, and the environment variable that overrides it. A `layers`
//! item carries the vendor's cross-layer order, highest first, so precedence is data and
//! never a constant in code. Public API only.
//!
//! Reds written to the architect's defaults for the readable-key list and for permission
//! rule text; those two are provisional until the operator rules. The mechanism tested
//! here does not depend on either.

use usher_knowledge::{Class, KnowledgeFile, Layer, MergeKind, SourceLayer};

const FILE: &str = r#"
schema_version = 5
sequence = 12

[[item]]
id = "demo.stack"
harness = "demo"
layer = "slow"
class = "layers"
versions = ">=1.0.0, <2.0.0"
summary = "Settings layers, highest first."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/precedence" }
last_verified = 2026-11-20
stack = ["managed", "command-line", "local", "project", "user"]

[[item]]
id = "demo.key-mode"
harness = "demo"
layer = "fast"
class = "settings-keys"
versions = ">=1.0.0, <2.0.0"
summary = "The model; one layer wins."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/model" }
last_verified = 2026-11-20
settings_key = { key = "model", merge = "scalar", sensitive = false, overrides = { variable = "DEMO_MODEL" } }

[[item]]
id = "demo.key-allow"
harness = "demo"
layer = "fast"
class = "settings-keys"
versions = ">=1.0.0, <2.0.0"
summary = "Allow rules; every layer adds entries."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/permissions" }
last_verified = 2026-11-20
settings_key = { key = "permissions.allow", merge = "list-merge", sensitive = false }

[[item]]
id = "demo.key-env"
harness = "demo"
layer = "fast"
class = "settings-keys"
versions = ">=1.0.0, <2.0.0"
summary = "The env table; objects combine, values are never read."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/env" }
last_verified = 2026-11-20
settings_key = { key = "env", merge = "object-merge", sensitive = true }

[[item]]
id = "demo.key-picker"
harness = "demo"
layer = "fast"
class = "settings-keys"
versions = ">=1.0.0, <2.0.0"
summary = "A key the harness merges by its own rule."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/picker" }
last_verified = 2026-11-20
settings_key = { key = "modelPicker", merge = "own-rule", sensitive = false }

[[item]]
id = "demo.key-auto"
harness = "demo"
layer = "fast"
class = "settings-keys"
versions = ">=1.0.0, <2.0.0"
summary = "An autoMode array; only user and managed files set it."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/auto" }
last_verified = 2026-11-20
settings_key = { key = "autoMode.allow", merge = "list-merge", sensitive = false, scopes = ["user", "managed"] }
"#;

const STACK: &str = r#"stack = ["managed", "command-line", "local", "project", "user"]"#;
const MODEL: &str = r#"settings_key = { key = "model", merge = "scalar", sensitive = false, overrides = { variable = "DEMO_MODEL" } }"#;
const ALLOW: &str =
    r#"settings_key = { key = "permissions.allow", merge = "list-merge", sensitive = false }"#;

fn rejects(text: &str, needles: &[&str]) {
    let err = KnowledgeFile::parse(text).expect_err("the check must fail");
    let shown = err.to_string();
    for needle in needles {
        assert!(shown.contains(needle), "`{needle}` not in `{shown}`");
    }
    assert!(!err.step().is_empty() && !err.check().is_empty());
    assert!(!err.status().is_empty() && !err.next_action().is_empty());
}

fn with(old: &str, new: &str) -> String {
    assert_eq!(FILE.matches(old).count(), 1, "`{old}` must occur once");
    FILE.replace(old, new)
}

fn accepts(text: &str) {
    if let Err(err) = KnowledgeFile::parse(text) {
        panic!("expected a valid file: {err}");
    }
}

#[test]
fn a_schema_5_file_carries_settings_keys_with_merge_kind_sensitivity_and_override() {
    let file = KnowledgeFile::parse(FILE).expect("a valid schema 5 file");
    assert_eq!(file.version().schema_version(), 5);
    let items = file.items();
    assert_eq!(items.len(), 6);
    let model = &items[1];
    assert_eq!(model.class(), Class::SettingsKeys);
    assert_eq!(model.layer(), Layer::Fast);
    let fact = model.settings_key().expect("a settings-key fact");
    assert_eq!(fact.key(), "model");
    assert_eq!(fact.merge(), MergeKind::Scalar);
    assert!(!fact.sensitive());
    let over = fact.overrides().expect("an override");
    assert_eq!(over.variable(), "DEMO_MODEL");
    assert_eq!(fact.scopes(), None);
    let allow = items[2].settings_key().expect("allow");
    assert_eq!(
        (allow.key(), allow.merge()),
        ("permissions.allow", MergeKind::ListMerge)
    );
    assert!(allow.overrides().is_none());
    let env = items[3].settings_key().expect("env");
    assert_eq!(env.merge(), MergeKind::ObjectMerge);
    assert!(env.sensitive());
    let picker = items[4].settings_key().expect("picker");
    assert_eq!(picker.merge(), MergeKind::OwnRule);
    let auto = items[5].settings_key().expect("auto");
    assert_eq!(auto.key(), "autoMode.allow");
    assert_eq!(
        auto.scopes(),
        Some(&[SourceLayer::User, SourceLayer::Managed][..])
    );
    assert!(items[0].settings_key().is_none());
}

#[test]
fn a_layers_item_carries_the_cross_layer_stack_highest_first() {
    let file = KnowledgeFile::parse(FILE).expect("a valid schema 5 file");
    let stack = &file.items()[0];
    assert_eq!(stack.class(), Class::Layers);
    assert_eq!(
        stack.stack(),
        ["managed", "command-line", "local", "project", "user"]
    );
    assert!(file.items()[1].stack().is_empty());
}

#[test]
fn a_schema_4_file_is_refused_naming_both_schemas() {
    rejects(
        &with("schema_version = 5", "schema_version = 4"),
        &["schema 4", "schema 5"],
    );
}

#[test]
fn the_embedded_file_is_schema_5() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    assert_eq!(file.version().schema_version(), 5);
}

#[test]
fn a_settings_key_item_of_the_slow_layer_is_refused() {
    let text = with(
        "layer = \"fast\"\nclass = \"settings-keys\"\nversions = \">=1.0.0, <2.0.0\"\nsummary = \"The model;",
        "layer = \"slow\"\nclass = \"settings-keys\"\nversions = \">=1.0.0, <2.0.0\"\nsummary = \"The model;",
    );
    rejects(
        &text,
        &[
            "demo.key-mode",
            "class `settings-keys`",
            "other layer",
            "`slow`",
        ],
    );
}

#[test]
fn a_settings_key_table_on_another_class_is_refused_naming_the_item() {
    let text = with(
        "class = \"settings-keys\"\nversions = \">=1.0.0, <2.0.0\"\nsummary = \"Allow rules;",
        "class = \"keys\"\nversions = \">=1.0.0, <2.0.0\"\nsummary = \"Allow rules;",
    );
    rejects(&text, &["demo.key-allow", "settings_key"]);
}

#[test]
fn a_settings_key_item_without_its_table_is_refused() {
    rejects(&with(ALLOW, ""), &["demo.key-allow", "settings_key"]);
}

#[test]
fn key_merge_and_sensitive_must_be_named_and_known() {
    rejects(
        &with(r#"key = "permissions.allow", "#, ""),
        &["demo.key-allow", "key"],
    );
    rejects(
        &with(
            r#"merge = "list-merge", sensitive = false }"#,
            r#"merge = "union", sensitive = false }"#,
        ),
        &["demo.key-allow", "merge", "union"],
    );
    rejects(
        &with(
            r#"merge = "list-merge", sensitive = false }"#,
            r#"sensitive = false }"#,
        ),
        &["demo.key-allow", "merge"],
    );
    rejects(
        &with(
            r#"merge = "list-merge", sensitive = false }"#,
            r#"merge = "list-merge" }"#,
        ),
        &["demo.key-allow", "sensitive"],
    );
    rejects(
        &with(
            r#"merge = "list-merge", sensitive = false }"#,
            r#"merge = "list-merge", sensitive = "no" }"#,
        ),
        &["demo.key-allow", "sensitive"],
    );
    rejects(
        &with(
            r#"merge = "list-merge", sensitive = false }"#,
            r#"merge = "list-merge", sensitive = false, colour = "red" }"#,
        ),
        &["demo.key-allow", "colour"],
    );
}

#[test]
fn a_key_path_is_names_joined_by_dots_and_a_star_is_one_whole_level() {
    for bad in [
        "", "a..b", ".a", "a.", "mod*", "a.*x", "**", "a.**.b", "a.?",
    ] {
        let text = with(
            ALLOW,
            &format!(
                r#"settings_key = {{ key = "{bad}", merge = "list-merge", sensitive = false }}"#
            ),
        );
        rejects(&text, &["demo.key-allow", "key", bad]);
    }
    accepts(&with(
        ALLOW,
        r#"settings_key = { key = "permissions.additionalDirectories", merge = "list-merge", sensitive = false }"#,
    ));
}

#[test]
fn no_settings_key_reaches_the_account_key() {
    for bad in [
        "oauthAccount",
        "oauthAccount.emailAddress",
        "projects.oauthAccount",
    ] {
        let text = with(
            ALLOW,
            &format!(r#"settings_key = {{ key = "{bad}", merge = "scalar", sensitive = false }}"#),
        );
        rejects(&text, &["demo.key-allow", "key", bad]);
    }
    accepts(&with(
        ALLOW,
        r#"settings_key = { key = "oauthAccountNote", merge = "scalar", sensitive = false }"#,
    ));
}

/// Red repair of the first version: an override names only its variable, because the item
/// is already one key.
#[test]
fn an_override_names_an_upper_case_variable_and_nothing_else() {
    for bad in ["demo_model", "DEMO-MODEL", "1DEMO", "", "DEMO=1", "_DEMO"] {
        let text = with(
            MODEL,
            &format!(
                r#"settings_key = {{ key = "model", merge = "scalar", sensitive = false, overrides = {{ variable = "{bad}" }} }}"#
            ),
        );
        rejects(&text, &["demo.key-mode", "overrides", "variable"]);
    }
    let missing_variable = with(
        MODEL,
        r#"settings_key = { key = "model", merge = "scalar", sensitive = false, overrides = {} }"#,
    );
    rejects(
        &missing_variable,
        &["demo.key-mode", "overrides", "variable"],
    );
    let extra_key = with(
        MODEL,
        r#"settings_key = { key = "model", merge = "scalar", sensitive = false, overrides = { variable = "DEMO_MODEL", key = "model" } }"#,
    );
    rejects(&extra_key, &["demo.key-mode", "overrides", "key"]);
}

#[test]
fn scopes_name_known_layers_once_each_and_are_not_empty() {
    let base = r#"settings_key = { key = "permissions.allow", merge = "list-merge", sensitive = false, scopes = SCOPES }"#;
    let put = |scopes: &str| with(ALLOW, &base.replace("SCOPES", scopes));
    accepts(&put(r#"["user", "managed"]"#));
    accepts(&put(r#"["user", "project", "local", "managed"]"#));
    for bad in [
        r#"[]"#,
        r#"["user", "user"]"#,
        r#"["user", "plugin"]"#,
        r#""user""#,
    ] {
        rejects(&put(bad), &["demo.key-allow", "scopes"]);
    }
    rejects(
        &put(r#"["user", "plugin"]"#),
        &["demo.key-allow", "scopes", "plugin"],
    );
}

#[test]
fn two_items_for_one_key_over_overlapping_versions_are_refused_naming_both() {
    let dup = format!(
        "{FILE}\n[[item]]\nid = \"demo.key-allow-again\"\nharness = \"demo\"\nlayer = \"fast\"\nclass = \"settings-keys\"\nversions = \">=1.5.0, <2.0.0\"\nsummary = \"Again.\"\nreference = {{ kind = \"vendor-docs\", locator = \"https://example.test/docs/again\" }}\nlast_verified = 2026-11-20\nsettings_key = {{ key = \"permissions.allow\", merge = \"list-merge\", sensitive = false }}\n"
    );
    rejects(
        &dup,
        &[
            "demo.key-allow",
            "demo.key-allow-again",
            "permissions.allow",
        ],
    );
    let apart = format!(
        "{}\n[[item]]\nid = \"demo.key-allow-v2\"\nharness = \"demo\"\nlayer = \"fast\"\nclass = \"settings-keys\"\nversions = \">=2.0.0, <3.0.0\"\nsummary = \"Next major.\"\nreference = {{ kind = \"vendor-docs\", locator = \"https://example.test/docs/v2\" }}\nlast_verified = 2026-11-20\nsettings_key = {{ key = \"permissions.allow\", merge = \"list-merge\", sensitive = false }}\n",
        FILE
    );
    accepts(&apart);
    let other_harness = format!(
        "{FILE}\n[[item]]\nid = \"other.key-allow\"\nharness = \"other\"\nlayer = \"fast\"\nclass = \"settings-keys\"\nversions = \">=1.0.0, <2.0.0\"\nsummary = \"Another harness.\"\nreference = {{ kind = \"vendor-docs\", locator = \"https://example.test/docs/other\" }}\nlast_verified = 2026-11-20\nsettings_key = {{ key = \"permissions.allow\", merge = \"list-merge\", sensitive = false }}\n"
    );
    accepts(&other_harness);
}

#[test]
fn the_stack_belongs_to_layers_and_layers_needs_it() {
    let on_keys = with(ALLOW, &format!("{ALLOW}\n{STACK}"));
    rejects(&on_keys, &["demo.key-allow", "stack"]);
    rejects(&with(STACK, ""), &["demo.stack", "stack"]);
    rejects(&with(STACK, "stack = []"), &["demo.stack", "stack"]);
    rejects(
        &with(STACK, r#"stack = "managed""#),
        &["demo.stack", "stack"],
    );
}

#[test]
fn a_stack_names_known_layers_once_each() {
    rejects(
        &with(STACK, r#"stack = ["managed", "plugin", "user"]"#),
        &["demo.stack", "stack", "plugin"],
    );
    rejects(
        &with(STACK, r#"stack = ["managed", "user", "managed"]"#),
        &["demo.stack", "stack", "managed"],
    );
    accepts(&with(STACK, r#"stack = ["managed", "user"]"#));
}

#[test]
fn two_stacks_for_one_harness_over_overlapping_versions_are_refused() {
    let dup = format!(
        "{FILE}\n[[item]]\nid = \"demo.stack-again\"\nharness = \"demo\"\nlayer = \"slow\"\nclass = \"layers\"\nversions = \">=1.5.0, <2.0.0\"\nsummary = \"Again.\"\nreference = {{ kind = \"vendor-docs\", locator = \"https://example.test/docs/again\" }}\nlast_verified = 2026-11-20\nstack = [\"managed\", \"user\"]\n"
    );
    rejects(&dup, &["demo.stack", "demo.stack-again", "stack"]);
}

#[test]
fn the_embedded_file_carries_the_vendors_stack_with_a_citation() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    let item = file
        .items()
        .iter()
        .find(|item| item.harness() == "claude-code" && item.class() == Class::Layers)
        .expect("a claude-code layers item");
    assert_eq!(
        item.stack(),
        ["managed", "command-line", "local", "project", "user"]
    );
    assert!(item.reference().locator().contains("settings"));
}

fn seeded(key: &str) -> usher_knowledge::SettingsKeyFact {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    file.items()
        .iter()
        .filter(|item| item.harness() == "claude-code")
        .filter_map(|item| item.settings_key())
        .find(|fact| fact.key() == key)
        .unwrap_or_else(|| panic!("the seed has a settings key `{key}`"))
        .clone()
}

#[test]
fn the_seed_lists_the_default_readable_keys_with_their_merge_kinds() {
    for (key, merge) in [
        ("permissions.allow", MergeKind::ListMerge),
        ("permissions.deny", MergeKind::ListMerge),
        ("permissions.ask", MergeKind::ListMerge),
        ("permissions.defaultMode", MergeKind::Scalar),
        ("autoMode.environment", MergeKind::ListMerge),
        ("autoMode.allow", MergeKind::ListMerge),
        ("autoMode.soft_deny", MergeKind::ListMerge),
        ("autoMode.hard_deny", MergeKind::ListMerge),
        ("model", MergeKind::Scalar),
        ("policyHelper", MergeKind::Scalar),
        ("wslInheritsWindowsSettings", MergeKind::Scalar),
        ("managedSourcesBehavior", MergeKind::Scalar),
    ] {
        let fact = seeded(key);
        assert_eq!(fact.merge(), merge, "{key}");
        assert!(!fact.sensitive(), "{key}");
    }
}

#[test]
fn the_seed_gives_the_keys_with_their_own_merge_rule_no_decision() {
    for key in [
        "fallbackModel",
        "modelPicker",
        "availableModels",
        "modelSettings",
    ] {
        assert_eq!(seeded(key).merge(), MergeKind::OwnRule, "{key}");
    }
}

#[test]
fn the_seed_marks_env_sensitive() {
    let env = seeded("env");
    assert_eq!(env.merge(), MergeKind::ObjectMerge);
    assert!(env.sensitive());
}

#[test]
fn every_seeded_settings_key_is_cited_and_appears_once_per_version() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    let keyed: Vec<_> = file
        .items()
        .iter()
        .filter(|item| item.class() == Class::SettingsKeys)
        .collect();
    assert!(keyed.len() >= 18, "{} settings-key items", keyed.len());
    for item in &keyed {
        assert!(!item.reference().locator().is_empty(), "{}", item.id());
        assert_eq!(item.layer(), Layer::Fast, "{}", item.id());
    }
    let mut keys: Vec<&str> = keyed
        .iter()
        .filter_map(|item| item.settings_key())
        .map(|fact| fact.key())
        .collect();
    keys.sort_unstable();
    let before = keys.len();
    keys.dedup();
    assert_eq!(before, keys.len(), "a key is listed twice");
}

#[test]
fn the_seed_names_no_credential_key() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    assert!(
        file.items()
            .iter()
            .any(|item| item.settings_key().is_some()),
        "the seed lists settings keys, so this check is not empty"
    );
    for item in file.items() {
        if let Some(fact) = item.settings_key() {
            assert!(
                fact.key().split('.').all(|part| part != "oauthAccount"),
                "{}",
                item.id()
            );
        }
    }
}

#[test]
fn the_seed_gives_each_auto_mode_array_the_user_and_managed_scopes() {
    for key in [
        "autoMode.environment",
        "autoMode.allow",
        "autoMode.soft_deny",
        "autoMode.hard_deny",
        "autoMode.classifyAllShell",
    ] {
        assert_eq!(
            seeded(key).scopes(),
            Some(&[SourceLayer::User, SourceLayer::Managed][..]),
            "{key}"
        );
    }
}

/// A `true` in the user file or on the command line wins even over a managed `false`, so
/// the boolean is not plain scalar precedence.
#[test]
fn the_seed_gives_classify_all_shell_its_own_rule() {
    assert_eq!(
        seeded("autoMode.classifyAllShell").merge(),
        MergeKind::OwnRule
    );
}

#[test]
fn the_seed_has_no_whole_auto_mode_item() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    let wild = file
        .items()
        .iter()
        .filter_map(|item| item.settings_key())
        .any(|fact| fact.key() == "autoMode" || fact.key() == "autoMode.*");
    assert!(!wild, "autoMode is five items, one per member");
    assert_eq!(seeded("autoMode.allow").key(), "autoMode.allow");
}

#[test]
fn the_seed_says_anthropic_model_overrides_model_in_any_file() {
    let model = seeded("model");
    assert_eq!(model.scopes(), None);
    let over = model.overrides().expect("model is overridden");
    assert_eq!(over.variable(), "ANTHROPIC_MODEL");
}

/// `ANTHROPIC_DEFAULT_MODEL` applies only when no file sets `model`, so it overrides
/// nothing.
#[test]
fn the_seed_names_no_default_model_variable_as_an_override() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    for item in file.items() {
        if let Some(over) = item.settings_key().and_then(|fact| fact.overrides()) {
            assert_ne!(over.variable(), "ANTHROPIC_DEFAULT_MODEL", "{}", item.id());
        }
    }
    let seen = file
        .items()
        .iter()
        .filter_map(|item| item.settings_key())
        .any(|fact| fact.overrides().is_some());
    assert!(seen, "the seed names at least one override");
}
