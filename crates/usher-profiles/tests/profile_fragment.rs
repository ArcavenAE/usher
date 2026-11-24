//! I-673 U2: a profile fragment for a harness that names the keys a profile may own is
//! refused when it sets any other key (docs/design/settings-renderer.md, D2). The caller
//! passes the keys, read from harness knowledge; this crate does not know the harness.
//! Fixtures are generated in a scratch directory per test and removed afterwards.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use usher_core::UsherError;
use usher_profiles::{FragmentFormat, HarnessRule, LoadRules, Profile, Source};

const MANIFEST: &str = r#"name = "team-setup"
version = "2026.11"

[harnesses.claude-code]
versions = ">=2.1.317, <3.0.0"
"#;

/// The keys knowledge marks profile-ownable for Claude Code (U1).
const OWNED: [&str; 7] = [
    "permissions.allow",
    "permissions.ask",
    "permissions.deny",
    "autoMode.environment",
    "autoMode.allow",
    "autoMode.soft_deny",
    "autoMode.hard_deny",
];

static NEXT: AtomicU32 = AtomicU32::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        let dir = std::env::temp_dir().join(format!(
            "usher-frag-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).expect("create scratch");
        Scratch(dir)
    }

    fn profile(&self, fragment: &str) -> PathBuf {
        let dir = self.0.join("p");
        fs::create_dir_all(dir.join("claude-code")).expect("create profile dir");
        fs::write(dir.join("profile.toml"), MANIFEST).expect("write manifest");
        fs::write(dir.join("claude-code/settings.json"), fragment).expect("write fragment");
        dir
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn rules(owned: bool) -> LoadRules {
    let rule = HarnessRule::new("claude-code", "settings.json", FragmentFormat::Json);
    LoadRules::new(vec![if owned {
        rule.with_owned_keys(&OWNED)
    } else {
        rule
    }])
}

fn load(fragment: &str, owned: bool) -> Result<Vec<Profile>, UsherError> {
    let scratch = Scratch::new();
    let dir: &Path = &scratch.profile(fragment);
    Profile::load(&Source::Directory(dir.to_path_buf()), &rules(owned))
}

fn shown(err: &UsherError) -> String {
    format!(
        "{} | {} | {} | {}",
        err.step(),
        err.check(),
        err.status(),
        err.next_action()
    )
}

/// Every row of D2's table, with the fragment that sets it, and the path the refusal names.
const BARRED: [(&str, &str); 9] = [
    (
        r#"{"env": {"ANTHROPIC_API_KEY": "canary"}}"#,
        "env.ANTHROPIC_API_KEY",
    ),
    (
        r#"{"env": {"CLAUDE_CODE_AUTO_MODE_SERVER": "1"}}"#,
        "env.CLAUDE_CODE_AUTO_MODE_SERVER",
    ),
    (r#"{"apiKeyHelper": "get-key.sh"}"#, "apiKeyHelper"),
    (r#"{"model": "sonnet"}"#, "model"),
    (r#"{"advisorModel": "opus"}"#, "advisorModel"),
    (
        r#"{"permissions": {"defaultMode": "auto"}}"#,
        "permissions.defaultMode",
    ),
    (
        r#"{"permissions": {"blockReadsOutsideWorkingDirectories": true}}"#,
        "permissions.blockReadsOutsideWorkingDirectories",
    ),
    (r#"{"hooks": {"PreToolUse": []}}"#, "hooks.PreToolUse"),
    (
        r#"{"statusLine": {"type": "command", "command": "x"}}"#,
        "statusLine.type",
    ),
];

#[test]
fn fragment_with_only_list_entries_loads() {
    let fragment = r#"{"permissions": {"allow": ["Bash(ls)"], "deny": []}, "autoMode": {"allow": ["$defaults", "x"]}}"#;
    let profiles = load(fragment, true).unwrap_or_else(|e| panic!("refused: {}", shown(&e)));
    let leaves = profiles[0]
        .fragment("claude-code")
        .expect("the fragment")
        .leaves()
        .to_vec();
    assert!(
        leaves.contains(&"permissions.allow".to_string()),
        "{leaves:?}"
    );
    assert!(leaves.contains(&"autoMode.allow".to_string()), "{leaves:?}");
}

#[test]
fn fragment_refuses_every_barred_path() {
    // Positive control, in the same run: the same loader with the keys named loads a
    // list-only fragment, so a loader that refuses everything cannot pass.
    load(r#"{"permissions": {"allow": ["Bash(ls)"]}}"#, true)
        .unwrap_or_else(|e| panic!("the control was refused: {}", shown(&e)));
    for (fragment, path) in BARRED {
        let err = load(fragment, true).expect_err(&format!("`{path}` must be refused"));
        let text = shown(&err);
        assert!(
            text.contains(path),
            "the refusal does not name `{path}`: {text}"
        );
        assert!(
            text.contains("may own"),
            "the refusal does not say what a profile may own: {text}"
        );
        assert!(
            !text.contains("canary"),
            "a value reached the refusal: {text}"
        );
    }
}

#[test]
fn a_key_nobody_listed_is_refused_too() {
    let err = load(r#"{"somethingNew": ["x"]}"#, true).expect_err("an unlisted key is refused");
    assert!(shown(&err).contains("somethingNew"), "{}", shown(&err));
}

/// Control, not a red for new code: a harness rule that names no owned keys keeps loading
/// every fragment, so existing callers do not change.
#[test]
fn a_rule_without_owned_keys_loads_any_fragment_control() {
    load(r#"{"model": "sonnet", "env": {"A": "b"}}"#, false)
        .unwrap_or_else(|e| panic!("refused: {}", shown(&e)));
}
