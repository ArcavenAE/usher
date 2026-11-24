//! aae-orc-7jm7u: `{:?}` of an MCP entry or of a profile that holds MCP servers prints names
//! and never values, because an entry's fields and a server's env and headers can carry a
//! token. Every value here is a synthetic canary. Fixtures live in a scratch directory per
//! test and are removed afterwards.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use usher_profiles::{FragmentFormat, HarnessRule, LoadRules, McpOrigin, Profile, Source};

const MANIFEST: &str = r#"name = "team-setup"
version = "2026.11"

[harnesses.claude-code]
versions = ">=2.1.317, <3.0.0"

[mcp_servers.shared]
command = "shared-mcp"
url = "https://example.test/mcp?token=canary-shared-url"
authorization = "Bearer canary-shared-header"

[mcp_servers.shared.env]
SHARED_TOKEN = "canary-shared-env"
"#;

const FRAGMENT: &str = r#"{"mcpServers": {"own": {"command": "own-mcp", "url": "https://example.test/own?token=canary-own-url", "authorization": "Bearer canary-own-header", "env": {"OWN_TOKEN": "canary-own-env"}}}}"#;

const CANARIES: [&str; 5] = [
    "canary-shared-url",
    "canary-shared-header",
    "canary-shared-env",
    "canary-own-url",
    "canary-own-header",
];

static NEXT: AtomicU32 = AtomicU32::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        let dir = std::env::temp_dir().join(format!(
            "usher-mcpdbg-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(dir.join("p/claude-code")).expect("create scratch");
        fs::write(dir.join("p/profile.toml"), MANIFEST).expect("write manifest");
        fs::write(dir.join("p/claude-code/settings.json"), FRAGMENT).expect("write fragment");
        Scratch(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn load() -> (Scratch, Vec<Profile>) {
    let scratch = Scratch::new();
    let rules = LoadRules::new(vec![
        HarnessRule::new("claude-code", "settings.json", FragmentFormat::Json)
            .with_mcp_path("mcpServers"),
    ]);
    let profiles = match Profile::load(&Source::Directory(scratch.0.join("p")), &rules) {
        Ok(profiles) => profiles,
        Err(err) => panic!("the fixture profile was refused: {}", err.status()),
    };
    (scratch, profiles)
}

fn holds_no_canary(text: &str, what: &str) {
    for canary in CANARIES {
        assert!(!text.contains(canary), "{what} prints a value (`{canary}`)");
    }
}

#[test]
fn debug_of_a_shared_mcp_entry_prints_names_and_no_values() {
    let (_scratch, profiles) = load();
    let entry = profiles[0]
        .mcp_entry("claude-code", "shared")
        .expect("entry");
    // Controls: the entry is the shared one and it holds the fields the test is about.
    assert_eq!(entry.origin(), McpOrigin::Shared);
    assert!(
        entry
            .field_str("url")
            .is_some_and(|v| v.contains("canary-shared-url"))
    );
    let text = format!("{entry:?}");
    holds_no_canary(&text, "Debug of a shared entry");
    assert!(
        text.contains("url") && text.contains("authorization"),
        "field names are printed: {}",
        text.len()
    );
}

#[test]
fn debug_of_a_fragment_mcp_entry_prints_names_and_no_values() {
    let (_scratch, profiles) = load();
    let entry = profiles[0].mcp_entry("claude-code", "own").expect("entry");
    assert_eq!(entry.origin(), McpOrigin::Fragment);
    assert!(
        entry
            .field_str("url")
            .is_some_and(|v| v.contains("canary-own-url"))
    );
    let text = format!("{entry:?}");
    holds_no_canary(&text, "Debug of a fragment entry");
    assert!(
        text.contains("url") && text.contains("authorization"),
        "field names are printed: {}",
        text.len()
    );
}

#[test]
fn debug_of_a_profile_holding_shared_mcp_servers_prints_no_values() {
    let (_scratch, profiles) = load();
    let text = format!("{:?}", profiles[0]);
    for canary in [
        "canary-shared-url",
        "canary-shared-header",
        "canary-shared-env",
    ] {
        assert!(
            !text.contains(canary),
            "Debug of a profile prints a shared server value (`{canary}`)"
        );
    }
    assert!(
        text.contains("shared"),
        "the server name is printed: {}",
        text.len()
    );
}
