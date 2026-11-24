//! Story 1.6, PR A: schema 6, the `settings-keys` class and the cross-layer stack.
//! A settings-key item names one key a harness reads, how it merges across layers, whether
//! its value is never read, and the environment variable that overrides it. A `layers`
//! item carries the vendor's cross-layer order, highest first, so precedence is data and
//! never a constant in code. Public API only.
//!
//! The default readable-key list and the handling of permission rule text are decided.
//! The mechanism tested here does not depend on either.

use usher_knowledge::{Class, KnowledgeFile, Layer, MergeKind, ReferenceKind, Scopes, SourceLayer};

const FILE: &str = r#"
schema_version = 6
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
settings_key = { key = "model", merge = "scalar", sensitive = false, scopes = ["any"], overrides = { variable = "DEMO_MODEL" } }

[[item]]
id = "demo.key-allow"
harness = "demo"
layer = "fast"
class = "settings-keys"
versions = ">=1.0.0, <2.0.0"
summary = "Allow rules; every layer adds entries."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/permissions" }
last_verified = 2026-11-20
settings_key = { key = "permissions.allow", merge = "list-merge", sensitive = false, scopes = ["any"] }

[[item]]
id = "demo.key-env"
harness = "demo"
layer = "fast"
class = "settings-keys"
versions = ">=1.0.0, <2.0.0"
summary = "The env table; objects combine, values are never read."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/env" }
last_verified = 2026-11-20
settings_key = { key = "env", merge = "object-merge", sensitive = true, scopes = ["any"] }

[[item]]
id = "demo.key-picker"
harness = "demo"
layer = "fast"
class = "settings-keys"
versions = ">=1.0.0, <2.0.0"
summary = "A key the harness merges by its own rule."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/picker" }
last_verified = 2026-11-20
settings_key = { key = "modelPicker", merge = "own-rule", sensitive = false, scopes = ["any"] }

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
const MODEL: &str = r#"settings_key = { key = "model", merge = "scalar", sensitive = false, scopes = ["any"], overrides = { variable = "DEMO_MODEL" } }"#;
const ALLOW: &str = r#"settings_key = { key = "permissions.allow", merge = "list-merge", sensitive = false, scopes = ["any"] }"#;

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
fn a_schema_6_file_carries_settings_keys_with_merge_kind_sensitivity_and_override() {
    let file = KnowledgeFile::parse(FILE).expect("a valid schema 6 file");
    assert_eq!(file.version().schema_version(), 6);
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
    assert_eq!(fact.scopes(), &Scopes::Any);
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
        &Scopes::Layers(vec![SourceLayer::User, SourceLayer::Managed])
    );
    assert!(items[0].settings_key().is_none());
}

#[test]
fn a_layers_item_carries_the_cross_layer_stack_highest_first() {
    let file = KnowledgeFile::parse(FILE).expect("a valid schema 6 file");
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
        &with("schema_version = 6", "schema_version = 4"),
        &["schema 4", "schema 6"],
    );
}

#[test]
fn the_embedded_file_parses_with_literal_key_paths_only() {
    let file = KnowledgeFile::embedded().expect("the seed parses under schema 6");
    for item in file.items() {
        if let Some(fact) = item.settings_key() {
            assert!(!fact.key().contains('*'), "{}", item.id());
        }
    }
    assert!(
        file.items()
            .iter()
            .any(|item| item.settings_key().is_some()),
        "the seed lists settings keys, so this check is not empty"
    );
}

#[test]
fn the_embedded_file_is_schema_5() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    assert_eq!(file.version().schema_version(), 6);
}

/// A control, not a red for new code: the class-to-layer mapping is the existing layer
/// check, so once schema 6 is accepted this passes against the stub too.
#[test]
fn a_settings_key_item_of_the_slow_layer_is_refused_control() {
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
            r#"merge = "list-merge", sensitive = false, scopes = ["any"] }"#,
            r#"merge = "union", sensitive = false, scopes = ["any"] }"#,
        ),
        &["demo.key-allow", "merge", "union"],
    );
    rejects(
        &with(
            r#"merge = "list-merge", sensitive = false, scopes = ["any"] }"#,
            r#"sensitive = false, scopes = ["any"] }"#,
        ),
        &["demo.key-allow", "merge"],
    );
    rejects(
        &with(
            r#"merge = "list-merge", sensitive = false, scopes = ["any"] }"#,
            r#"merge = "list-merge", scopes = ["any"] }"#,
        ),
        &["demo.key-allow", "sensitive"],
    );
    rejects(
        &with(
            r#"merge = "list-merge", sensitive = false, scopes = ["any"] }"#,
            r#"merge = "list-merge", sensitive = "no", scopes = ["any"] }"#,
        ),
        &["demo.key-allow", "sensitive"],
    );
    rejects(
        &with(
            r#"merge = "list-merge", sensitive = false, scopes = ["any"] }"#,
            r#"merge = "list-merge", sensitive = false, scopes = ["any"], colour = "red" }"#,
        ),
        &["demo.key-allow", "colour"],
    );
}

/// A settings key is a literal path: a name, or names joined by single dots. No `*`
/// appears anywhere in schema 6, so a child the seed does not list is never matched by
/// a pattern. The seat key-path wildcard rule is separate.
#[test]
fn a_key_path_is_names_joined_by_dots_and_no_star_appears() {
    for bad in [
        "", "a..b", ".a", "a.", "mod*", "a.*x", "**", "a.**.b", "a.?", "*", "*.x", "a.*", "a.*.b",
    ] {
        let text = with(
            ALLOW,
            &format!(
                r#"settings_key = {{ key = "{bad}", merge = "list-merge", sensitive = false, scopes = ["any"] }}"#
            ),
        );
        if bad.contains('*') {
            rejects(&text, &["demo.key-allow", "key", bad, "wildcard"]);
        } else {
            rejects(&text, &["demo.key-allow", "key", bad]);
        }
    }
    accepts(&with(
        ALLOW,
        r#"settings_key = { key = "permissions.additionalDirectories", merge = "list-merge", sensitive = false, scopes = ["any"] }"#,
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
            &format!(
                r#"settings_key = {{ key = "{bad}", merge = "scalar", sensitive = false, scopes = ["any"] }}"#
            ),
        );
        rejects(&text, &["demo.key-allow", "key", bad]);
    }
    accepts(&with(
        ALLOW,
        r#"settings_key = { key = "oauthAccountNote", merge = "scalar", sensitive = false, scopes = ["any"] }"#,
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
                r#"settings_key = {{ key = "model", merge = "scalar", sensitive = false, scopes = ["any"], overrides = {{ variable = "{bad}" }} }}"#
            ),
        );
        rejects(&text, &["demo.key-mode", "overrides", "variable"]);
    }
    let missing_variable = with(
        MODEL,
        r#"settings_key = { key = "model", merge = "scalar", sensitive = false, scopes = ["any"], overrides = {} }"#,
    );
    rejects(
        &missing_variable,
        &["demo.key-mode", "overrides", "variable"],
    );
    let extra_key = with(
        MODEL,
        r#"settings_key = { key = "model", merge = "scalar", sensitive = false, scopes = ["any"], overrides = { variable = "DEMO_MODEL", key = "model" } }"#,
    );
    rejects(&extra_key, &["demo.key-mode", "overrides", "key"]);
}

/// Red repair of the first version, where scopes were optional and unset meant any file.
/// A key with no cited scope is not decided, so the seed cannot claim the widest scope
/// quietly: `scopes` is required, either exactly ["any"] or a set of file layers.
#[test]
fn scopes_are_required_and_are_any_alone_or_a_set_of_file_layers() {
    let base = r#"settings_key = { key = "permissions.allow", merge = "list-merge", sensitive = false, scopes = SCOPES }"#;
    let put = |scopes: &str| with(ALLOW, &base.replace("SCOPES", scopes));
    accepts(&put(r#"["any"]"#));
    accepts(&put(r#"["user", "managed"]"#));
    accepts(&put(r#"["user", "local", "managed"]"#));
    accepts(&put(r#"["managed", "user", "project", "local"]"#));
    let without = with(
        ALLOW,
        r#"settings_key = { key = "permissions.allow", merge = "list-merge", sensitive = false }"#,
    );
    rejects(&without, &["demo.key-allow", "scopes"]);
    for bad in [
        r#"[]"#,
        r#"["user", "user"]"#,
        r#"["any", "any"]"#,
        r#"["user", "plugin"]"#,
        r#"["global-state"]"#,
        r#"["server-managed-cache"]"#,
        r#"["any", "user"]"#,
        r#"["user", "any"]"#,
        r#""user""#,
        r#""any""#,
    ] {
        rejects(&put(bad), &["demo.key-allow", "scopes"]);
    }
    rejects(
        &put(r#"["user", "plugin"]"#),
        &["demo.key-allow", "scopes", "plugin"],
    );
    rejects(
        &put(r#"["any", "user"]"#),
        &["demo.key-allow", "scopes", "any"],
    );
}

#[test]
fn two_items_for_one_key_over_overlapping_versions_are_refused_naming_both() {
    let dup = format!(
        "{FILE}\n[[item]]\nid = \"demo.key-allow-again\"\nharness = \"demo\"\nlayer = \"fast\"\nclass = \"settings-keys\"\nversions = \">=1.5.0, <2.0.0\"\nsummary = \"Again.\"\nreference = {{ kind = \"vendor-docs\", locator = \"https://example.test/docs/again\" }}\nlast_verified = 2026-11-20\nsettings_key = {{ key = \"permissions.allow\", merge = \"list-merge\", sensitive = false, scopes = [\"any\"] }}\n"
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
        "{}\n[[item]]\nid = \"demo.key-allow-v2\"\nharness = \"demo\"\nlayer = \"fast\"\nclass = \"settings-keys\"\nversions = \">=2.0.0, <3.0.0\"\nsummary = \"Next major.\"\nreference = {{ kind = \"vendor-docs\", locator = \"https://example.test/docs/v2\" }}\nlast_verified = 2026-11-20\nsettings_key = {{ key = \"permissions.allow\", merge = \"list-merge\", sensitive = false, scopes = [\"any\"] }}\n",
        FILE
    );
    accepts(&apart);
    let other_harness = format!(
        "{FILE}\n[[item]]\nid = \"other.key-allow\"\nharness = \"other\"\nlayer = \"fast\"\nclass = \"settings-keys\"\nversions = \">=1.0.0, <2.0.0\"\nsummary = \"Another harness.\"\nreference = {{ kind = \"vendor-docs\", locator = \"https://example.test/docs/other\" }}\nlast_verified = 2026-11-20\nsettings_key = {{ key = \"permissions.allow\", merge = \"list-merge\", sensitive = false, scopes = [\"any\"] }}\n"
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
    assert_eq!(item.reference().kind(), ReferenceKind::VendorDocs);
    assert_eq!(
        item.reference().locator(),
        "https://code.claude.com/docs/en/settings"
    );
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
        ("permissions.defaultMode", MergeKind::OwnRule),
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

/// The default readable-key list, exactly. The permissions keys, `autoMode`, `model`, the
/// managed control keys, `env` (by name only), `apiKeyHelper` (sensitive) and the one env child
/// read by value (I-673 U1). The model-selection keys
/// `availableModels`, `fallbackModel`, `modelPicker` and `modelSettings` belong to `model`.
const SEEDED_KEYS: [&str; 20] = [
    "apiKeyHelper",
    "autoMode.allow",
    "autoMode.classifyAllShell",
    "autoMode.environment",
    "autoMode.hard_deny",
    "autoMode.soft_deny",
    "availableModels",
    "env",
    "env.CLAUDE_CODE_AUTO_MODE_SERVER",
    "fallbackModel",
    "managedSourcesBehavior",
    "model",
    "modelPicker",
    "modelSettings",
    "permissions.allow",
    "permissions.ask",
    "permissions.defaultMode",
    "permissions.deny",
    "policyHelper",
    "wslInheritsWindowsSettings",
];

#[test]
fn every_seeded_settings_key_is_cited_and_the_set_is_exactly_the_default_list() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    let keyed: Vec<_> = file
        .items()
        .iter()
        .filter(|item| item.class() == Class::SettingsKeys)
        .collect();
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
    assert_eq!(keys, SEEDED_KEYS);
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
            &Scopes::Layers(vec![SourceLayer::User, SourceLayer::Managed]),
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
    assert_eq!(model.scopes(), &Scopes::Any);
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

fn same_layers(scopes: &Scopes, want: &[SourceLayer]) -> bool {
    match scopes {
        Scopes::Layers(have) => {
            have.len() == want.len() && want.iter().all(|layer| have.contains(layer))
        }
        _ => false,
    }
}

/// Every seeded key's scope, as the vendor's settings reference gives it for that key
/// (its "Scope" line). The autoMode arrays take the scope of the autoMode entry, which
/// states one for the whole object. Compared as sets, so the written order is free.
#[test]
fn the_seed_gives_every_key_the_scope_the_settings_reference_states() {
    use SourceLayer::{Managed, User};
    let any = [
        "permissions.allow",
        "permissions.ask",
        "permissions.deny",
        "permissions.defaultMode",
        "model",
        "env",
        "env.CLAUDE_CODE_AUTO_MODE_SERVER",
        "apiKeyHelper",
        "fallbackModel",
        "availableModels",
        "modelSettings",
    ];
    for key in any {
        assert_eq!(seeded(key).scopes(), &Scopes::Any, "{key}");
    }
    for key in [
        "autoMode.environment",
        "autoMode.allow",
        "autoMode.soft_deny",
        "autoMode.hard_deny",
        "autoMode.classifyAllShell",
        "modelPicker",
    ] {
        assert!(
            same_layers(seeded(key).scopes(), &[User, Managed]),
            "{key}: {:?}",
            seeded(key).scopes()
        );
    }
    for key in [
        "policyHelper",
        "wslInheritsWindowsSettings",
        "managedSourcesBehavior",
    ] {
        assert!(
            same_layers(seeded(key).scopes(), &[Managed]),
            "{key}: {:?}",
            seeded(key).scopes()
        );
    }
    let mut covered: Vec<&str> = any
        .iter()
        .copied()
        .chain([
            "autoMode.environment",
            "autoMode.allow",
            "autoMode.soft_deny",
            "autoMode.hard_deny",
            "autoMode.classifyAllShell",
            "modelPicker",
            "policyHelper",
            "wslInheritsWindowsSettings",
            "managedSourcesBehavior",
        ])
        .collect();
    covered.sort_unstable();
    assert_eq!(
        covered, SEEDED_KEYS,
        "the table covers exactly the seeded keys"
    );
}

#[test]
fn the_seed_cites_the_model_entry_of_the_settings_reference() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    let item = file
        .items()
        .iter()
        .find(|item| {
            item.harness() == "claude-code"
                && item
                    .settings_key()
                    .is_some_and(|fact| fact.key() == "model")
        })
        .expect("the model item");
    assert_eq!(item.reference().kind(), ReferenceKind::VendorDocs);
    assert_eq!(
        item.reference().locator(),
        "https://code.claude.com/docs/en/settings-reference#model"
    );
}

/// The vendor says `auto` and `bypassPermissions` do not take effect from project or
/// local settings, and does not say what applies instead. Plain scalar precedence would
/// print a project `auto` as in effect when the harness ignores it, which is a guess. So
/// the key has its own rule, keeps the any-file scope, and its summary says why.
#[test]
fn the_seed_gives_default_mode_its_own_rule_with_the_vendors_sentence() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    let item = file
        .items()
        .iter()
        .find(|item| {
            item.harness() == "claude-code"
                && item
                    .settings_key()
                    .is_some_and(|fact| fact.key() == "permissions.defaultMode")
        })
        .expect("the defaultMode item");
    let fact = item.settings_key().expect("a settings-key fact");
    assert_eq!(fact.merge(), MergeKind::OwnRule);
    assert_eq!(fact.scopes(), &Scopes::Any);
    assert!(
        item.summary()
            .contains("don't take effect from project or local settings"),
        "{}",
        item.summary()
    );
}

// ---- I-673 U1: the marks the settings renderer design needs (docs/design/settings-renderer.md D1 to D4) ----

fn seed_fact(file: &KnowledgeFile, key: &str) -> usher_knowledge::SettingsKeyFact {
    file.items()
        .iter()
        .filter(|item| item.harness() == "claude-code")
        .filter_map(|item| item.settings_key())
        .find(|fact| fact.key() == key)
        .unwrap_or_else(|| panic!("the seed has no claude-code settings key `{key}`"))
        .clone()
}

#[test]
fn api_key_helper_is_sensitive() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    assert!(seed_fact(&file, "apiKeyHelper").sensitive());
}

#[test]
fn auto_mode_server_is_readable_by_value_with_reason() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    let fact = seed_fact(&file, "env.CLAUDE_CODE_AUTO_MODE_SERVER");
    assert!(
        !fact.sensitive(),
        "the one named exception is not sensitive"
    );
    let reason = fact
        .readable_reason()
        .expect("a readable key carries its reason");
    assert!(!reason.trim().is_empty());
    // The exception is one name: `env` itself stays sensitive and no other env.* child exists.
    assert!(seed_fact(&file, "env").sensitive());
    let children: Vec<String> = file
        .items()
        .iter()
        .filter(|i| i.harness() == "claude-code")
        .filter_map(|i| i.settings_key())
        .map(|f| f.key().to_string())
        .filter(|k| k.starts_with("env."))
        .collect();
    assert_eq!(children, ["env.CLAUDE_CODE_AUTO_MODE_SERVER"]);
    // Its versions are no wider than the parent's.
    let by_key = |key: &str| {
        file.items()
            .iter()
            .find(|i| i.settings_key().is_some_and(|f| f.key() == key))
            .unwrap_or_else(|| panic!("no item for `{key}`"))
    };
    assert_eq!(
        by_key("env.CLAUDE_CODE_AUTO_MODE_SERVER").versions(),
        by_key("env").versions()
    );
    // The parent summary names the exception (U1).
    let parent = file
        .items()
        .iter()
        .find(|i| i.id() == "claude-code.key-env")
        .expect("the env item");
    assert!(parent.summary().contains("CLAUDE_CODE_AUTO_MODE_SERVER"));
}

#[test]
fn profile_ownable_keys_are_marked() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    let ownable = [
        "permissions.allow",
        "permissions.ask",
        "permissions.deny",
        "autoMode.environment",
        "autoMode.allow",
        "autoMode.soft_deny",
        "autoMode.hard_deny",
    ];
    for key in ownable {
        assert!(seed_fact(&file, key).profile_ownable(), "{key}");
    }
    // Positive control for the negative: the bar is a mark, not the absence of items.
    for key in ["model", "env", "permissions.defaultMode"] {
        assert!(!seed_fact(&file, key).profile_ownable(), "{key}");
    }
}

#[test]
fn auto_mode_items_carry_the_defaults_rule() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    for key in [
        "autoMode.environment",
        "autoMode.allow",
        "autoMode.soft_deny",
        "autoMode.hard_deny",
    ] {
        assert!(seed_fact(&file, key).defaults_rule(), "{key}");
    }
    for key in ["permissions.allow", "autoMode.classifyAllShell"] {
        assert!(!seed_fact(&file, key).defaults_rule(), "{key}");
    }
}

// ---- I-673 U1: schema 6 and the invariants of the three marks ----

fn schema6(old: &str, new: &str) -> String {
    with(old, new)
}

#[test]
fn the_embedded_file_is_schema_6() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    assert_eq!(file.version().schema_version(), 6);
}

#[test]
fn a_schema_5_file_is_refused_naming_both_schemas() {
    rejects(
        &with("schema_version = 6", "schema_version = 5"),
        &["schema 5", "schema 6"],
    );
}

#[test]
fn the_three_marks_load_on_a_list_key_and_read_back() {
    let text = schema6(
        ALLOW,
        r#"settings_key = { key = "permissions.allow", merge = "list-merge", sensitive = false, scopes = ["any"], profile_ownable = true, defaults_rule = true }"#,
    );
    let file = KnowledgeFile::parse(&text).expect("a schema 6 file with the marks");
    let fact = file
        .items()
        .iter()
        .filter_map(|i| i.settings_key())
        .find(|f| f.key() == "permissions.allow")
        .expect("the allow key");
    assert!(fact.profile_ownable() && fact.defaults_rule());
    // Controls: a key without the marks reads false, and a sensitive env child is accepted.
    let plain = file
        .items()
        .iter()
        .filter_map(|i| i.settings_key())
        .find(|f| f.key() == "model")
        .expect("the model key");
    assert!(
        !plain.profile_ownable() && !plain.defaults_rule() && plain.readable_reason().is_none()
    );
    accepts(&schema6(
        ALLOW,
        r#"settings_key = { key = "env.DEMO_TOKEN", merge = "scalar", sensitive = true, scopes = ["any"] }"#,
    ));
}

#[test]
fn a_readable_env_child_loads_with_its_reason() {
    let text = schema6(
        ALLOW,
        r#"settings_key = { key = "env.DEMO_FLAG", merge = "scalar", sensitive = false, scopes = ["any"], readable_reason = "compared to 1, never stored" }"#,
    );
    let file = KnowledgeFile::parse(&text).expect("a readable env child");
    let reason = file
        .items()
        .iter()
        .filter_map(|i| i.settings_key())
        .find(|f| f.key() == "env.DEMO_FLAG")
        .and_then(|f| f.readable_reason().map(str::to_string));
    assert_eq!(reason.as_deref(), Some("compared to 1, never stored"));
}

#[test]
fn defaults_rule_on_a_key_that_is_not_list_merge_is_refused() {
    rejects(
        &schema6(
            MODEL,
            r#"settings_key = { key = "model", merge = "scalar", sensitive = false, scopes = ["any"], defaults_rule = true }"#,
        ),
        &["demo.key-mode", "defaults_rule", "list-merge"],
    );
}

#[test]
fn profile_ownable_on_a_key_that_is_not_list_merge_is_refused() {
    rejects(
        &schema6(
            MODEL,
            r#"settings_key = { key = "model", merge = "scalar", sensitive = false, scopes = ["any"], profile_ownable = true }"#,
        ),
        &["demo.key-mode", "profile_ownable", "list-merge"],
    );
}

#[test]
fn readable_reason_on_a_sensitive_key_is_refused() {
    rejects(
        &schema6(
            ALLOW,
            r#"settings_key = { key = "env.DEMO_FLAG", merge = "scalar", sensitive = true, scopes = ["any"], readable_reason = "why" }"#,
        ),
        &["readable_reason", "needs `sensitive = false`"],
    );
}

#[test]
fn an_env_child_that_is_not_sensitive_needs_a_reason() {
    rejects(
        &schema6(
            ALLOW,
            r#"settings_key = { key = "env.DEMO_FLAG", merge = "scalar", sensitive = false, scopes = ["any"] }"#,
        ),
        &["env.DEMO_FLAG", "readable_reason"],
    );
}
