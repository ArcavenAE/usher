//! Story 1.5, PR C: managed first-wins sources. The fixture knowledge file is signed by
//! the test kit's signer and taken through the update path, not the real seed, so the
//! ranks and platforms below are the test's own.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use usher_adapter_claude_code::ClaudeCodeAdapter;
use usher_core::{
    KeyNames, LaunchContext, SourceLister, SourceListing, SourceStatus, SystemRoot, environment_key,
};
use usher_knowledge::{Knowledge, KnowledgeFile, Platform, TrustedKeys};
use usher_testkit::{ScratchHome, Signer};

const TOP: &str = "etc/claude-code/managed-top.json";
const MIDDLE: &str = "etc/claude-code/managed-middle.json";
const LOW: &str = "etc/claude-code/managed-low.json";
const EVERY_USER: &str = "every/user-settings.json";
const MAC_USER: &str = "mac/user-settings.json";
const MAC_ONLY: &str = "Library/Application Support/ClaudeCode/managed-settings.json";

/// A user-layer source that always exists, limited or not to some platforms.
fn user_item(id: &str, path: &str, platforms: Option<&str>) -> String {
    let platforms = platforms
        .map(|list| format!(", platforms = {list}"))
        .unwrap_or_default();
    format!(
        r#"
[[item]]
id = "{id}"
harness = "claude-code"
layer = "slow"
class = "sources"
versions = ">=2.1.317, <3.0.0"
summary = "A user settings file."
reference = {{ kind = "vendor-docs", locator = "https://example.test/docs/settings" }}
last_verified = 2026-11-20
source = {{ path = "{path}", layer = "user", read = "read", rank = 1, always_present = true{platforms} }}
"#
    )
}

fn item(id: &str, path: &str, rank: u32, platforms: Option<&str>) -> String {
    let platforms = platforms
        .map(|list| format!(", platforms = {list}"))
        .unwrap_or_default();
    format!(
        r#"
[[item]]
id = "{id}"
harness = "claude-code"
layer = "slow"
class = "sources"
versions = ">=2.1.317, <3.0.0"
summary = "A managed settings file."
reference = {{ kind = "vendor-docs", locator = "https://example.test/docs/managed" }}
last_verified = 2026-11-20
source = {{ path = "{path}", layer = "managed", read = "first-wins", rank = {rank}, always_present = false{platforms} }}
"#
    )
}

/// A knowledge file with three Linux managed ranks (4 top, 3 middle, 2 low), a higher
/// macOS-only one, and the control-key list.
fn fixture() -> KnowledgeFile {
    let text = format!(
        r#"
schema_version = 3
sequence = 99
{}{}{}{}{}{}
[[item]]
id = "claude-code.managed-control-keys"
harness = "claude-code"
layer = "fast"
class = "keys"
versions = ">=2.1.317, <3.0.0"
summary = "Keys that control delivery and carry no policy."
reference = {{ kind = "vendor-docs", locator = "https://example.test/docs/managed" }}
last_verified = 2026-11-20
keys = ["policyHelper", "wslInheritsWindowsSettings", "managedSourcesBehavior"]
"#,
        item("demo.top", &format!("/{TOP}"), 4, Some(r#"["linux"]"#)),
        item(
            "demo.middle",
            &format!("/{MIDDLE}"),
            3,
            Some(r#"["linux"]"#)
        ),
        item("demo.low", &format!("/{LOW}"), 2, Some(r#"["linux"]"#)),
        item("demo.mac", &format!("/{MAC_ONLY}"), 5, Some(r#"["macos"]"#)),
        user_item("demo.every-user", &format!("/{EVERY_USER}"), None),
        user_item(
            "demo.mac-user",
            &format!("/{MAC_USER}"),
            Some(r#"["macos"]"#)
        ),
    );
    let signer = Signer::new();
    let trusted = TrustedKeys::for_tests(&signer.public_key_base64(), None).expect("keys");
    let mut knowledge = Knowledge::embedded().expect("embedded");
    knowledge
        .update(
            text.as_bytes(),
            &signer.sign(text.as_bytes(), "managed fixture"),
            &trusted,
        )
        .expect("a signed update is taken");
    knowledge.current().clone()
}

struct Rig {
    scratch: ScratchHome,
    system: PathBuf,
}

impl Rig {
    fn new() -> Self {
        let scratch = ScratchHome::new().expect("scratch");
        let system = scratch.root().join("system");
        fs::create_dir_all(&system).expect("system root");
        Rig { scratch, system }
    }

    fn file(&self, relative: &str, text: &str) -> PathBuf {
        let path = self.system.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("dirs");
        fs::write(&path, text).expect("file");
        path
    }

    fn list(&self) -> SourceListing {
        let home = self.scratch.home().to_path_buf();
        let key = environment_key("host", "alex", "claude-code", &home, &[]).expect("key");
        let context = LaunchContext {
            key,
            config_dir: home.join(".claude"),
            home,
            repo: None,
            user: "alex".to_string(),
            harness_version: Some("2.1.317".to_string()),
        };
        ClaudeCodeAdapter::new(
            fixture(),
            SystemRoot::at(self.system.clone()),
            Platform::Linux,
        )
        .list_sources(&context)
        .expect("a listing")
    }
}

fn status_of<'a>(listing: &'a SourceListing, path: &Path) -> &'a SourceStatus {
    &listing
        .sources
        .iter()
        .find(|s| s.path == path)
        .unwrap_or_else(|| panic!("{} is not listed", path.display()))
        .status
}

fn unread(reason: &str) -> SourceStatus {
    SourceStatus::Unread {
        reason: reason.to_string(),
    }
}

const POLICY: &str = r#"{"permissions":{}}"#;

#[test]
fn an_empty_top_file_is_unread_and_the_next_ranked_that_delivers_wins() {
    let rig = Rig::new();
    let top = rig.file(TOP, "{}");
    let middle = rig.file(MIDDLE, POLICY);
    let low = rig.file(LOW, POLICY);
    let listing = rig.list();
    assert_eq!(listing.sources.len(), 3, "{:?}", listing.sources);
    assert_eq!(
        status_of(&listing, &top),
        &unread("delivers no policy key: empty")
    );
    assert_eq!(status_of(&listing, &middle), &SourceStatus::InEffect);
    assert_eq!(
        status_of(&listing, &low),
        &SourceStatus::Shadowed { by: middle }
    );
}

#[test]
fn a_control_only_top_file_is_unread_and_the_next_ranked_that_delivers_wins() {
    let rig = Rig::new();
    let top = rig.file(TOP, r#"{"policyHelper":"h","managedSourcesBehavior":"x"}"#);
    let middle = rig.file(MIDDLE, POLICY);
    let listing = rig.list();
    assert_eq!(listing.sources.len(), 2, "{:?}", listing.sources);
    assert_eq!(
        status_of(&listing, &top),
        &unread("delivers no policy key: only control keys")
    );
    assert_eq!(status_of(&listing, &middle), &SourceStatus::InEffect);
}

#[test]
fn when_no_managed_file_delivers_every_one_is_unread_with_its_reason() {
    let rig = Rig::new();
    let top = rig.file(TOP, "{}");
    let middle = rig.file(MIDDLE, r#"{"policyHelper":"h"}"#);
    let listing = rig.list();
    assert_eq!(listing.sources.len(), 2, "{:?}", listing.sources);
    assert_eq!(
        status_of(&listing, &top),
        &unread("delivers no policy key: empty")
    );
    assert_eq!(
        status_of(&listing, &middle),
        &unread("delivers no policy key: only control keys")
    );
}

#[test]
fn a_bad_file_on_top_leaves_the_lower_ones_undecided_and_says_which_file() {
    let rig = Rig::new();
    let top = rig.file(TOP, r#"{"token": "sk-live-secret-123" oops}"#);
    let middle = rig.file(MIDDLE, POLICY);
    let low = rig.file(LOW, POLICY);
    let listing = rig.list();
    assert_eq!(listing.sources.len(), 3, "{:?}", listing.sources);
    match status_of(&listing, &top) {
        SourceStatus::Unknown { reason } => {
            assert!(
                reason.contains("not valid JSON at line 1, column "),
                "{reason}"
            );
            assert!(!reason.contains("sk-live-secret-123"), "{reason}");
        }
        other => panic!("expected unknown, got {other:?}"),
    }
    let undecided = SourceStatus::Unknown {
        reason: format!(
            "first-wins not decided: {} could not be read",
            top.display()
        ),
    };
    assert_eq!(status_of(&listing, &middle), &undecided);
    assert_eq!(status_of(&listing, &low), &undecided);
}

#[test]
fn a_bad_file_below_a_winner_is_shadowed_and_keeps_its_parse_note() {
    let rig = Rig::new();
    let top = rig.file(TOP, POLICY);
    let middle = rig.file(MIDDLE, "{ not json");
    let listing = rig.list();
    assert_eq!(listing.sources.len(), 2, "{:?}", listing.sources);
    assert_eq!(status_of(&listing, &top), &SourceStatus::InEffect);
    assert_eq!(
        status_of(&listing, &middle),
        &SourceStatus::Shadowed { by: top }
    );
    let bad = listing
        .sources
        .iter()
        .find(|s| s.path == middle)
        .expect("listed");
    match &bad.keys {
        KeyNames::NotRead { reason } => {
            assert!(
                reason.contains("not valid JSON at line 1, column "),
                "{reason}"
            )
        }
        other => panic!("expected the parse note, got {other:?}"),
    }
}

#[test]
fn another_platforms_source_never_takes_part_in_first_wins() {
    let rig = Rig::new();
    let mac = rig.file(MAC_ONLY, POLICY);
    let top = rig.file(TOP, POLICY);
    let listing = rig.list();
    assert_eq!(listing.sources.len(), 2, "{:?}", listing.sources);
    assert_eq!(status_of(&listing, &top), &SourceStatus::InEffect);
    assert_eq!(
        status_of(&listing, &mac),
        &unread("knowledge item demo.mac; read on macos only")
    );
}

#[test]
fn another_platforms_absent_always_present_source_is_not_named_while_an_every_platform_one_is() {
    let rig = Rig::new();
    let listing = rig.list();
    assert!(listing.sources.is_empty());
    let user = listing
        .absent_layers
        .iter()
        .find(|a| a.layer == "user")
        .unwrap_or_else(|| panic!("the every-platform user source is named: {listing:?}"));
    assert_eq!(user.paths, [rig.system.join(EVERY_USER)]);
    let named = format!("{:?}", listing.absent_layers);
    assert!(!named.contains(MAC_USER), "{named}");
}

#[test]
fn an_unreadable_top_file_is_unknown_and_leaves_the_lower_ones_undecided() {
    let rig = Rig::new();
    let top = rig.file(TOP, POLICY);
    fs::set_permissions(&top, fs::Permissions::from_mode(0o000)).expect("mode");
    let middle = rig.file(MIDDLE, POLICY);
    let listing = rig.list();
    assert_eq!(listing.sources.len(), 2, "{:?}", listing.sources);
    match status_of(&listing, &top) {
        SourceStatus::Unknown { reason } => assert!(!reason.is_empty()),
        other => panic!("expected unknown, got {other:?}"),
    }
    assert_eq!(
        status_of(&listing, &middle),
        &SourceStatus::Unknown {
            reason: format!(
                "first-wins not decided: {} could not be read",
                top.display()
            ),
        }
    );
}

#[test]
fn a_non_delivering_top_then_a_bad_file_leaves_the_lowest_undecided() {
    let rig = Rig::new();
    let top = rig.file(TOP, "{}");
    let middle = rig.file(MIDDLE, "{ not json");
    let low = rig.file(LOW, POLICY);
    let listing = rig.list();
    assert_eq!(listing.sources.len(), 3, "{:?}", listing.sources);
    assert_eq!(
        status_of(&listing, &top),
        &unread("delivers no policy key: empty")
    );
    assert!(matches!(
        status_of(&listing, &middle),
        SourceStatus::Unknown { .. }
    ));
    assert_eq!(
        status_of(&listing, &low),
        &SourceStatus::Unknown {
            reason: format!(
                "first-wins not decided: {} could not be read",
                middle.display()
            ),
        }
    );
}
