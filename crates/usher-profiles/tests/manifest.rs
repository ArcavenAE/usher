//! The `profile.toml` manifest: closed fields with name patterns, a free-text
//! version, four-field errors that name the profile and the field, values
//! stored as written, and the tools table parsed and kept.

use usher_core::Version;
use usher_profiles::Manifest;

const ORIGIN: &str = "team/profiles/setup";

/// The smallest valid manifest. Tests append tables after it.
const VALID: &str = r#"name = "team-setup"
version = "2026.11"

[harnesses.claude-code]
versions = ">=2.1.317, <3.0.0"
"#;

fn parse(text: &str) -> Result<Manifest, usher_core::UsherError> {
    Manifest::parse(text, ORIGIN)
}

fn must_parse(text: &str) -> Manifest {
    parse(text).unwrap_or_else(|err| panic!("a valid manifest was refused: {}", err.status()))
}

/// The error for `text`, which must be refused; and the control that `VALID`
/// itself parses, so a parser that refuses everything cannot pass.
fn refusal(text: &str) -> usher_core::UsherError {
    must_parse(VALID);
    parse(text).expect_err("this manifest must be refused")
}

fn names_field(err: &usher_core::UsherError, field: &str) {
    assert!(
        err.check().contains(field) || err.status().contains(field),
        "the error does not name `{field}`: check `{}`, status `{}`",
        err.check(),
        err.status()
    );
}

#[test]
fn a_full_manifest_parses_into_its_parts() {
    let text = format!(
        r#"{VALID}
[harnesses.codex]
versions = ">=0.160.0"

[tools]
node = ">=22.0.0, <23.0.0"
ripgrep = ">=14.0.0"

[mcp_servers.github]
command = "github-mcp"
args = ["--stdio"]

[notes]
"claude-code:permissions.allow" = "Why the allow list is short."
"mcp:github" = "Read only."
"#
    );
    let manifest = must_parse(&text);
    assert_eq!(manifest.name(), "team-setup");
    assert_eq!(manifest.version(), "2026.11");

    let ids: Vec<&str> = manifest.harnesses().iter().map(|h| h.id()).collect();
    assert_eq!(ids, ["claude-code", "codex"]);
    let claude = &manifest.harnesses()[0];
    assert!(claude.versions().contains(&Version::new(2, 1, 329)));
    assert!(!claude.versions().contains(&Version::new(3, 0, 0)));

    let tools: Vec<&str> = manifest.tools().iter().map(|t| t.name()).collect();
    assert_eq!(tools, ["node", "ripgrep"]);
    assert!(
        manifest.tools()[0]
            .versions()
            .contains(&Version::new(22, 4, 0))
    );
    assert!(
        !manifest.tools()[0]
            .versions()
            .contains(&Version::new(23, 0, 0))
    );

    assert_eq!(manifest.mcp_servers().len(), 1);
    assert_eq!(manifest.mcp_servers()[0].name(), "github");
    assert_eq!(
        manifest.mcp_servers()[0].field_str("command"),
        Some("github-mcp")
    );

    let notes: Vec<(&str, &str)> = manifest
        .notes()
        .iter()
        .map(|n| (n.key(), n.text()))
        .collect();
    assert_eq!(
        notes,
        [
            (
                "claude-code:permissions.allow",
                "Why the allow list is short."
            ),
            ("mcp:github", "Read only."),
        ]
    );
}

#[test]
fn the_optional_tables_may_be_absent() {
    let manifest = must_parse(VALID);
    assert!(manifest.tools().is_empty());
    assert!(manifest.mcp_servers().is_empty());
    assert!(manifest.notes().is_empty());
}

#[test]
fn a_name_is_one_to_sixty_four_of_lowercase_digits_and_hyphens_starting_with_a_letter() {
    let long_ok = format!("a{}", "b".repeat(63));
    let long_bad = format!("a{}", "b".repeat(64));
    for good in ["a", "team-setup-2", long_ok.as_str()] {
        let text = VALID.replace("team-setup", good);
        assert_eq!(must_parse(&text).name(), good);
    }
    for bad in [
        "",
        "Upper",
        "1abc",
        "has space",
        "a_b",
        "-a",
        long_bad.as_str(),
    ] {
        let text = VALID.replace("team-setup", bad);
        names_field(&refusal(&text), "name");
    }
}

#[test]
fn a_harness_id_has_the_same_pattern_as_a_name() {
    for bad in ["Claude", "1x", "a b", "a_b"] {
        let text = VALID.replace("claude-code", bad);
        names_field(&refusal(&text), "harnesses");
    }
    let text = VALID.replace("claude-code", "goose");
    assert_eq!(must_parse(&text).harnesses()[0].id(), "goose");
}

// The version field is free text for now; the operator may want semver, which
// would change this one validator.
#[test]
fn a_version_is_free_text_on_one_line_of_at_most_sixty_four_characters() {
    for good in ["2026.11", "abc123", "draft for review", &"v".repeat(64)] {
        let text = VALID.replace("2026.11", good);
        assert_eq!(must_parse(&text).version(), good);
    }
    for bad in ["", "two\\nlines", &"v".repeat(65)] {
        let text = VALID.replace("2026.11", bad);
        names_field(&refusal(&text), "version");
    }
}

#[test]
fn a_harness_needs_a_valid_satisfiable_version_range() {
    let missing = VALID.replace("versions = \">=2.1.317, <3.0.0\"\n", "");
    names_field(&refusal(&missing), "harnesses.claude-code.versions");
    for bad in ["not a range", "1.0.0", ">=2.0.0, <1.0.0", ""] {
        let text = VALID.replace(">=2.1.317, <3.0.0", bad);
        names_field(&refusal(&text), "harnesses.claude-code.versions");
    }
}

#[test]
fn an_unknown_key_is_refused_and_named() {
    let top = format!("colour = \"blue\"\n{VALID}");
    names_field(&refusal(&top), "colour");

    let in_harness = format!("{VALID}flavor = \"x\"\n");
    names_field(&refusal(&in_harness), "flavor");

    let in_table = format!("{VALID}\n[extras]\nx = 1\n");
    names_field(&refusal(&in_table), "extras");
}

#[test]
fn every_refusal_is_a_four_field_error_naming_the_profile_and_the_field() {
    let text = VALID.replace("team-setup", "Not Valid");
    let err = refusal(&text);
    assert!(err.step().contains(ORIGIN), "step: {}", err.step());
    assert!(err.check().contains("name"), "check: {}", err.check());
    assert!(!err.status().is_empty(), "the status is empty");
    assert!(!err.next_action().is_empty(), "the next action is empty");
}

#[test]
fn text_that_is_not_toml_is_refused_naming_the_profile() {
    let err = refusal("name = \n= nope [");
    assert!(err.step().contains(ORIGIN), "step: {}", err.step());
    assert!(
        err.check().contains("profile.toml"),
        "check: {}",
        err.check()
    );
    assert!(!err.next_action().is_empty());
}

#[test]
fn a_required_field_that_is_missing_is_named() {
    let no_name = VALID.replace("name = \"team-setup\"\n", "");
    names_field(&refusal(&no_name), "name");
    let no_version = VALID.replace("version = \"2026.11\"\n", "");
    names_field(&refusal(&no_version), "version");
}

#[test]
fn the_tools_table_holds_name_to_range_pairs_only() {
    let ok = format!("{VALID}\n[tools]\nnode = \">=22.0.0\"\n");
    assert_eq!(must_parse(&ok).tools().len(), 1);

    let not_text = format!("{VALID}\n[tools]\nnode = 22\n");
    names_field(&refusal(&not_text), "tools.node");

    let table = format!("{VALID}\n[tools.node]\nversion = \">=22.0.0\"\n");
    names_field(&refusal(&table), "tools.node");

    let bad_range = format!("{VALID}\n[tools]\nnode = \"latest\"\n");
    names_field(&refusal(&bad_range), "tools.node");
}

#[test]
fn shared_mcp_servers_are_kept_as_written_with_path_tokens_unresolved() {
    let text = format!(
        r#"{VALID}
[mcp_servers.files]
command = "${{HOME}}/bin/files-mcp"
cwd = "${{REPO}}"
config = "${{CONFIG_DIR}}/files.json"
user = "${{USER}}"
args = ["--root", "${{REPO}}/docs"]
"#
    );
    let manifest = must_parse(&text);
    let server = &manifest.mcp_servers()[0];
    assert_eq!(server.field_str("command"), Some("${HOME}/bin/files-mcp"));
    assert_eq!(server.field_str("cwd"), Some("${REPO}"));
    assert_eq!(server.field_str("config"), Some("${CONFIG_DIR}/files.json"));
    assert_eq!(server.field_str("user"), Some("${USER}"));
    assert_eq!(
        server.field_names(),
        ["command", "cwd", "config", "user", "args"]
    );
}

#[test]
fn a_shared_mcp_server_must_be_a_table() {
    let text = format!("{VALID}\n[mcp_servers]\ngithub = \"github-mcp\"\n");
    names_field(&refusal(&text), "mcp_servers.github");
}

// Meaning change: this red also refused the note key "mcp:" with no server
// name. The manifest reader does not decide that case, so it is no longer
// pinned here; the note key for a server is read by the next change.
#[test]
fn a_note_is_keyed_by_a_covered_harness_and_a_key_path_or_by_an_mcp_server() {
    let good = [
        "claude-code:permissions.allow",
        "claude-code:env.\\\"a.b\\\"",
        "mcp:github",
    ];
    for key in good {
        let text = format!("{VALID}\n[notes]\n\"{key}\" = \"why\"\n");
        let manifest = must_parse(&text);
        assert_eq!(manifest.notes()[0].key(), key.replace("\\\"", "\""));
    }
    let bad = [
        "permissions.allow",
        "claude-code:",
        "claude-code:permissions..allow",
        "codex:model",
        ":model",
    ];
    for key in bad {
        let text = format!("{VALID}\n[notes]\n\"{key}\" = \"why\"\n");
        let err = refusal(&text);
        assert!(
            err.check().contains(key) || err.status().contains(key),
            "the error does not name the note `{key}`: {} / {}",
            err.check(),
            err.status()
        );
    }
}

#[test]
fn a_note_needs_text() {
    let empty = format!("{VALID}\n[notes]\n\"claude-code:model\" = \"\"\n");
    names_field(&refusal(&empty), "claude-code:model");
    let multi = format!("{VALID}\n[notes]\n\"claude-code:model\" = \"line one\\nline two\"\n");
    assert_eq!(must_parse(&multi).notes()[0].text(), "line one\nline two");
}

// Provisional: a profile must cover at least one harness. A tools-only profile
// may be wanted later, which would remove this one check.
#[test]
fn a_profile_that_covers_no_harness_is_refused() {
    let no_table = "name = \"team-setup\"\nversion = \"2026.11\"\n";
    let empty_table = format!("{no_table}\n[harnesses]\n");
    for text in [no_table.to_string(), empty_table] {
        let err = refusal(&text);
        assert!(
            err.check().contains("profile.toml") && err.check().contains("harnesses"),
            "check: {}",
            err.check()
        );
        assert!(
            err.status().contains("the profile covers no harness"),
            "status: {}",
            err.status()
        );
        assert!(err.step().contains(ORIGIN), "step: {}", err.step());
    }
}

#[test]
fn an_mcp_server_name_is_one_to_sixty_four_of_letters_digits_dot_underscore_and_hyphen_not_starting_with_a_dot()
 {
    let long_ok = "s".repeat(64);
    let long_bad = "s".repeat(65);
    for good in ["github.com_mcp", "fs-tools", "A_b.9", long_ok.as_str()] {
        let text = format!("{VALID}\n[mcp_servers.\"{good}\"]\ncommand = \"x\"\n");
        assert_eq!(must_parse(&text).mcp_servers()[0].name(), good);
    }
    for bad in ["my server", "a:b", ".hidden", "", long_bad.as_str()] {
        let text = format!("{VALID}\n[mcp_servers.\"{bad}\"]\ncommand = \"x\"\n");
        let err = refusal(&text);
        assert!(
            err.check().contains("mcp_servers")
                && (bad.is_empty() || err.check().contains(bad) || err.status().contains(bad)),
            "the error does not name the server `{bad}`: {} / {}",
            err.check(),
            err.status()
        );
    }
}

#[test]
fn a_tool_name_is_one_to_one_hundred_twenty_eight_printable_characters_without_space_or_quote() {
    let long_ok = "t".repeat(128);
    let long_bad = "t".repeat(129);
    for good in ["node", "cargo:ripgrep", "npm:@scope/pkg", long_ok.as_str()] {
        let text = format!("{VALID}\n[tools]\n\"{good}\" = \">=1.0.0\"\n");
        assert_eq!(must_parse(&text).tools()[0].name(), good);
    }
    for bad in [
        "node js",
        "a'b",
        "a\\\"b",
        "a\\u0001b",
        "",
        long_bad.as_str(),
    ] {
        let text = format!("{VALID}\n[tools]\n\"{bad}\" = \">=1.0.0\"\n");
        let err = refusal(&text);
        assert!(
            err.check().contains("tools"),
            "the error does not name the tools table: {} / {}",
            err.check(),
            err.status()
        );
    }
}
