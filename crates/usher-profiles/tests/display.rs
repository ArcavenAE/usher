//! Names a refusal echoes are shown safely: each control character is written
//! as \uXXXX in uppercase hex and everything else stays as it is, so an error
//! never carries a raw control character, an escape sequence included.

use usher_profiles::{Manifest, display_name};

const VALID: &str = r#"name = "team-setup"
version = "2026.11"

[harnesses.claude-code]
versions = ">=2.1.317, <3.0.0"
"#;

#[test]
fn every_control_character_is_written_as_an_uppercase_u_escape() {
    let controls: Vec<char> = (0u32..=0x1F)
        .chain(std::iter::once(0x7F))
        .map(|n| char::from_u32(n).expect("a char"))
        .collect();
    assert_eq!(controls.len(), 33);
    for c in controls {
        let shown = display_name(&c.to_string());
        assert_eq!(
            shown,
            format!("\\u{:04X}", u32::from(c)),
            "for U+{:04X}",
            u32::from(c)
        );
    }
}

#[test]
fn everything_else_stays_as_written() {
    for text in [
        "o'brien",
        "node js",
        "a\"b",
        "github.com_mcp",
        "caf\u{e9}",
        "npm:@scope/pkg",
        "",
    ] {
        assert_eq!(display_name(text), text);
    }
    // Control: a control character in the same text is the only thing that changes.
    assert_eq!(display_name("o'brien\u{1}"), "o'brien\\u0001");
}

#[test]
fn a_name_with_a_control_character_and_a_del_show_their_escapes_and_no_raw_character() {
    let shown = display_name("a\u{1}b");
    assert_eq!(shown, "a\\u0001b");
    assert!(!shown.contains('\u{1}'));
    assert_eq!(display_name("\u{7F}"), "\\u007F");
    assert_eq!(display_name("x\u{1B}y"), "x\\u001By");
}

fn refusal(text: &str) -> usher_core::UsherError {
    Manifest::parse(VALID, "team/profiles/setup").expect("control: a valid manifest parses");
    Manifest::parse(text, "team/profiles/setup").expect_err("this manifest must be refused")
}

fn assert_no_raw_escape(err: &usher_core::UsherError) {
    for (field, value) in [
        ("step", err.step()),
        ("check", err.check()),
        ("status", err.status()),
        ("next action", err.next_action()),
    ] {
        assert!(
            !value.chars().any(|c| c.is_control()),
            "the {field} holds a raw control character: {value:?}"
        );
    }
}

#[test]
fn a_refusal_shows_an_escape_in_a_tool_name_as_text() {
    let err = refusal(&format!("{VALID}\n[tools]\n\"a\\u001Bb\" = \">=1.0.0\"\n"));
    assert_no_raw_escape(&err);
    assert!(
        err.check().contains("a\\u001Bb") || err.status().contains("a\\u001Bb"),
        "{} / {}",
        err.check(),
        err.status()
    );
    let control = refusal(&format!("{VALID}\n[tools]\n\"a\\u0001b\" = \">=1.0.0\"\n"));
    assert_no_raw_escape(&control);
    assert!(
        control.check().contains("a\\u0001b") || control.status().contains("a\\u0001b"),
        "{} / {}",
        control.check(),
        control.status()
    );
}

#[test]
fn a_refusal_shows_an_escape_in_an_mcp_server_name_as_text() {
    let err = refusal(&format!(
        "{VALID}\n[mcp_servers.\"a\\u001Bb\"]\ncommand = \"x\"\n"
    ));
    assert_no_raw_escape(&err);
    assert!(
        err.check().contains("a\\u001Bb") || err.status().contains("a\\u001Bb"),
        "{} / {}",
        err.check(),
        err.status()
    );
}

#[test]
fn a_refusal_shows_an_escape_in_a_harness_id_as_text() {
    let text = VALID.replace("[harnesses.claude-code]", "[harnesses.\"a\\u001Bb\"]");
    let err = refusal(&text);
    assert_no_raw_escape(&err);
    assert!(
        err.check().contains("a\\u001Bb") || err.status().contains("a\\u001Bb"),
        "{} / {}",
        err.check(),
        err.status()
    );
}

#[test]
fn a_refusal_shows_an_escape_in_a_note_segment_as_text() {
    let err = refusal(&format!(
        "{VALID}\n[notes]\n\"claude-code:a.b\\u001Bc\" = \"why\"\n"
    ));
    assert_no_raw_escape(&err);
    assert!(
        err.check().contains("b\\u001Bc") || err.status().contains("b\\u001Bc"),
        "{} / {}",
        err.check(),
        err.status()
    );
}

#[test]
fn a_refusal_shows_an_escape_in_an_unknown_key_as_text() {
    // Control: the same keys without the escape are refused too, by name.
    let plain = refusal(&format!(
        "{VALID}\n[tools]\nnode = \">=1.0.0\"\n\n[extras]\nx = 1\n"
    ));
    assert!(plain.check().contains("extras"), "{}", plain.check());

    let top = refusal(&format!("\"a\\u001Bb\" = 1\n{VALID}"));
    assert_no_raw_escape(&top);
    assert!(
        top.check().contains("a\\u001Bb") || top.status().contains("a\\u001Bb"),
        "{} / {}",
        top.check(),
        top.status()
    );

    let under_harness = refusal(&format!("{VALID}\"a\\u001Bb\" = 1\n"));
    assert_no_raw_escape(&under_harness);
    assert!(
        under_harness.check().contains("a\\u001Bb") || under_harness.status().contains("a\\u001Bb"),
        "{} / {}",
        under_harness.check(),
        under_harness.status()
    );
}

#[test]
fn a_refusal_holds_no_raw_escape_when_a_version_range_holds_one() {
    let harness = VALID.replace(">=2.1.317, <3.0.0", "\\u001B[31m");
    let err = refusal(&harness);
    assert_no_raw_escape(&err);
    assert!(err.check().contains("harnesses.claude-code.versions"));

    let tool = refusal(&format!("{VALID}\n[tools]\nnode = \"\\u001B[31m\"\n"));
    assert_no_raw_escape(&tool);
    assert!(tool.check().contains("tools.node"));
}

// Ruling 7.1 PR A, Q7: the escape set is exactly char::is_control(), which
// takes in the C1 controls U+0080 to U+009F as well as C0 and DEL.
#[test]
fn a_c1_control_character_is_written_as_an_uppercase_u_escape() {
    let shown = display_name("a\u{9b}b");
    assert_eq!(shown, "a\\u009Bb");
    assert!(!shown.contains('\u{9b}'));
    for n in 0x80u32..=0x9F {
        let c = char::from_u32(n).expect("a char");
        assert_eq!(display_name(&c.to_string()), format!("\\u{n:04X}"));
    }
    // Control: the characters either side of the C1 block stay as written.
    assert_eq!(display_name("\u{7e}\u{a0}\u{a1}"), "\u{7e}\u{a0}\u{a1}");
    assert_eq!(display_name("o'brien"), "o'brien");
}

#[test]
fn a_refusal_shows_a_c1_control_in_a_tool_name_as_text_and_holds_no_raw_one() {
    let err = refusal(&format!("{VALID}\n[tools]\n\"a\\u009Bb\" = \">=1.0.0\"\n"));
    assert_no_raw_escape(&err);
    assert!(
        err.check().contains("a\\u009Bb") || err.status().contains("a\\u009Bb"),
        "{} / {}",
        err.check(),
        err.status()
    );
}
