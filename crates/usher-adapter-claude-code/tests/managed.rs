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

/// A knowledge file with three Linux managed ranks (2 top, 3 middle, 4 low), a macOS-only
/// one at 1, and the control-key list. A smaller number ranks higher; 1 is the macOS-only
/// source, absent from this platform, and nothing on this platform sits at 1, as ranks
/// also count vendor deliveries knowledge does not list.
fn fixture() -> KnowledgeFile {
    fixture_with("")
}

/// The same file with `extra` items added.
fn fixture_with(extra: &str) -> KnowledgeFile {
    let text = format!(
        r#"
schema_version = 3
sequence = 99
{}{}{}{}{}{}{}
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
        item("demo.top", &format!("/{TOP}"), 2, Some(r#"["linux"]"#)),
        item(
            "demo.middle",
            &format!("/{MIDDLE}"),
            3,
            Some(r#"["linux"]"#)
        ),
        item("demo.low", &format!("/{LOW}"), 4, Some(r#"["linux"]"#)),
        item("demo.mac", &format!("/{MAC_ONLY}"), 1, Some(r#"["macos"]"#)),
        user_item("demo.every-user", &format!("/{EVERY_USER}"), None),
        user_item(
            "demo.mac-user",
            &format!("/{MAC_USER}"),
            Some(r#"["macos"]"#)
        ),
        extra,
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
        self.list_with(&fixture())
    }

    fn list_with(&self, knowledge: &KnowledgeFile) -> SourceListing {
        let home = self.scratch.home().to_path_buf();
        let key = environment_key("host", "alex", "claude-code", &home, &[]).expect("key");
        let context = LaunchContext {
            key,
            config_dir: home.join(".claude"),
            global_state_dir: home.clone(),
            set_variables: Default::default(),
            home,
            repo: None,
            user: "alex".to_string(),
            harness_version: Some("2.1.317".to_string()),
        };
        ClaudeCodeAdapter::new(
            knowledge.clone(),
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
    let paths: Vec<&Path> = user.checked.iter().map(|c| c.path.as_path()).collect();
    assert_eq!(paths, [rig.system.join(EVERY_USER)]);
    let named = format!("{:?}", listing.absent_layers);
    assert!(!named.contains(MAC_USER), "{named}");
}

#[test]
fn an_unreadable_top_file_is_unknown_and_leaves_the_lower_ones_undecided() {
    let rig = Rig::new();
    let top = rig.file(TOP, POLICY);
    fs::set_permissions(&top, fs::Permissions::from_mode(0o000)).expect("mode");
    // Probe the effect, not the uid: root, root squash and ACLs all read a mode 000 file.
    if fs::File::open(&top).is_ok() {
        usher_testkit::skip(
            "an_unreadable_top_file_is_unknown_and_leaves_the_lower_ones_undecided",
            "this process can read a mode 000 file, so there is no unreadable file to test",
        );
        return;
    }
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

const TWIN: &str = "etc/claude-code/managed-twin.json";

/// A knowledge file with a second Linux managed source at `rank`.
fn with_twin(rank: u32) -> KnowledgeFile {
    fixture_with(&item(
        "demo.twin",
        &format!("/{TWIN}"),
        rank,
        Some(r#"["linux"]"#),
    ))
}

fn undecided_by_rank(rank: u32) -> SourceStatus {
    SourceStatus::Unknown {
        reason: format!("first-wins not decided: two managed sources share rank {rank}"),
    }
}

/// Two deliverers at one rank have no stated winner, so neither is in effect and neither
/// is guessed; everything ranked below them is not decided either, and says why.
#[test]
fn two_managed_sources_with_one_rank_on_a_platform_read_unknown_naming_the_rank() {
    let rig = Rig::new();
    let top = rig.file(TOP, POLICY);
    let twin = rig.file(TWIN, POLICY);
    let middle = rig.file(MIDDLE, POLICY);
    let low = rig.file(LOW, POLICY);
    let listing = rig.list_with(&with_twin(2));
    assert_eq!(listing.sources.len(), 4, "{:?}", listing.sources);
    let tied = SourceStatus::Unknown {
        reason: "two managed sources share rank 2".to_string(),
    };
    assert_eq!(status_of(&listing, &top), &tied);
    assert_eq!(status_of(&listing, &twin), &tied);
    assert_eq!(status_of(&listing, &middle), &undecided_by_rank(2));
    assert_eq!(status_of(&listing, &low), &undecided_by_rank(2));
}

/// A tie below a delivering winner decides nothing, so it is never reported as a tie.
#[test]
fn a_tie_below_a_delivering_winner_reads_shadowed_for_both_members() {
    let rig = Rig::new();
    let top = rig.file(TOP, POLICY);
    let middle = rig.file(MIDDLE, POLICY);
    let twin = rig.file(TWIN, POLICY);
    let listing = rig.list_with(&with_twin(3));
    assert_eq!(listing.sources.len(), 3, "{:?}", listing.sources);
    assert_eq!(status_of(&listing, &top), &SourceStatus::InEffect);
    let shadowed = SourceStatus::Shadowed { by: top };
    assert_eq!(status_of(&listing, &middle), &shadowed);
    assert_eq!(status_of(&listing, &twin), &shadowed);
}

/// Only sources that deliver can tie: an empty member drops out, and the other member
/// wins alone.
#[test]
fn a_tie_where_one_member_delivers_nothing_is_won_by_the_other() {
    let rig = Rig::new();
    let top = rig.file(TOP, "{}");
    let twin = rig.file(TWIN, POLICY);
    let low = rig.file(LOW, POLICY);
    let listing = rig.list_with(&with_twin(2));
    assert_eq!(listing.sources.len(), 3, "{:?}", listing.sources);
    assert_eq!(
        status_of(&listing, &top),
        &unread("delivers no policy key: empty")
    );
    assert_eq!(status_of(&listing, &twin), &SourceStatus::InEffect);
    assert_eq!(
        status_of(&listing, &low),
        &SourceStatus::Shadowed { by: twin }
    );
}

/// An unreadable member might deliver, so paired with a deliverer nothing is decided, and
/// the lower sources name the bad file as the cause.
#[test]
fn a_tie_of_an_unreadable_source_and_a_deliverer_leaves_both_unknown_and_names_the_bad_file() {
    let rig = Rig::new();
    let top = rig.file(TOP, "{ not json");
    let twin = rig.file(TWIN, POLICY);
    let middle = rig.file(MIDDLE, POLICY);
    let listing = rig.list_with(&with_twin(2));
    assert_eq!(listing.sources.len(), 3, "{:?}", listing.sources);
    match status_of(&listing, &top) {
        SourceStatus::Unknown { reason } => assert!(
            reason.contains("not valid JSON at line 1, column "),
            "{reason}"
        ),
        other => panic!("expected unknown, got {other:?}"),
    }
    assert_eq!(
        status_of(&listing, &twin),
        &SourceStatus::Unknown {
            reason: format!(
                "first-wins not decided: {} could not be read",
                top.display()
            ),
        }
    );
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

/// Only sources that deliver can tie: two that deliver nothing drop out together, and the
/// decision moves to the next rank down.
#[test]
fn a_tie_where_neither_member_delivers_moves_the_decision_down() {
    let rig = Rig::new();
    let top = rig.file(TOP, "{}");
    let twin = rig.file(TWIN, r#"{"policyHelper":"h"}"#);
    let low = rig.file(LOW, POLICY);
    let listing = rig.list_with(&with_twin(2));
    assert_eq!(listing.sources.len(), 3, "{:?}", listing.sources);
    assert_eq!(
        status_of(&listing, &top),
        &unread("delivers no policy key: empty")
    );
    assert_eq!(
        status_of(&listing, &twin),
        &unread("delivers no policy key: only control keys")
    );
    assert_eq!(status_of(&listing, &low), &SourceStatus::InEffect);
}

/// An unreadable member that ties with one that delivers nothing is alone in the
/// decision, so it follows the single-file rule and not the tie rule.
#[test]
fn a_tie_of_an_unreadable_source_and_an_empty_one_is_not_a_rank_tie() {
    let rig = Rig::new();
    let top = rig.file(TOP, "{ not json");
    let twin = rig.file(TWIN, "{}");
    let middle = rig.file(MIDDLE, POLICY);
    let listing = rig.list_with(&with_twin(2));
    assert_eq!(listing.sources.len(), 3, "{:?}", listing.sources);
    match status_of(&listing, &top) {
        SourceStatus::Unknown { reason } => assert!(
            reason.contains("not valid JSON at line 1, column "),
            "{reason}"
        ),
        other => panic!("expected unknown, got {other:?}"),
    }
    assert_eq!(
        status_of(&listing, &twin),
        &unread("delivers no policy key: empty")
    );
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

/// A layer with two always-present sources lists both paths, each with the id of the item
/// that states it and in knowledge order, so an id cannot drift from its path.
#[test]
fn two_absent_sources_in_one_layer_keep_each_id_with_its_own_path() {
    let rig = Rig::new();
    let second = "second/user-settings.json";
    let knowledge = fixture_with(&user_item("demo.second-user", &format!("/{second}"), None));
    let listing = rig.list_with(&knowledge);
    let user = listing
        .absent_layers
        .iter()
        .find(|a| a.layer == "user")
        .unwrap_or_else(|| panic!("the user layer is named: {listing:?}"));
    let pairs: Vec<(&Path, &str)> = user
        .checked
        .iter()
        .map(|c| (c.path.as_path(), c.item_id.as_str()))
        .collect();
    assert_eq!(
        pairs,
        [
            (rig.system.join(EVERY_USER).as_path(), "demo.every-user"),
            (rig.system.join(second).as_path(), "demo.second-user"),
        ]
    );
}

/// Puts a directory at mode 000 and restores it on drop, so the scratch home can be
/// removed even when an assertion fails.
struct Locked(PathBuf);

impl Locked {
    fn new(dir: PathBuf) -> Self {
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o000)).expect("mode");
        Locked(dir)
    }
}

impl Drop for Locked {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
    }
}

const SECRET: &str = "FILE-CONTENT-NEVER-LISTED";

#[test]
fn a_candidate_that_cannot_be_checked_is_never_named_as_no_file() {
    let rig = Rig::new();
    let file = rig.file(EVERY_USER, SECRET);
    let _locked = Locked::new(file.parent().expect("a parent").to_path_buf());
    // Probe the effect, not the uid: root and ACLs can look through a mode 000 directory.
    if fs::metadata(&file).is_ok() {
        usher_testkit::skip(
            "a_candidate_that_cannot_be_checked_is_never_named_as_no_file",
            "this process can look through a mode 000 directory, so there is nothing to test",
        );
        return;
    }
    let listing = rig.list();
    assert!(
        listing.absent_layers.iter().all(|a| a.layer != "user"),
        "an always-present layer that could not be checked is not absent: {:?}",
        listing.absent_layers
    );
    let checked = listing
        .not_checked
        .iter()
        .find(|n| n.layer == "user")
        .unwrap_or_else(|| panic!("the user layer is named as not checked: {listing:?}"));
    assert!(
        checked.reason.contains("PermissionDenied"),
        "{}",
        checked.reason
    );
    assert!(
        checked.reason.contains(&file.display().to_string()),
        "{}",
        checked.reason
    );
    assert!(!format!("{listing:?}").contains(SECRET));
}

#[test]
fn a_layer_that_is_not_always_present_does_not_drop_out_when_it_cannot_be_checked() {
    let rig = Rig::new();
    let knowledge = fixture_with(&item(
        "demo.locked",
        "/locked/managed.json",
        5,
        Some(r#"["linux"]"#),
    ));
    let file = rig.file("locked/managed.json", SECRET);
    let _locked = Locked::new(file.parent().expect("a parent").to_path_buf());
    if fs::metadata(&file).is_ok() {
        usher_testkit::skip(
            "a_layer_that_is_not_always_present_does_not_drop_out_when_it_cannot_be_checked",
            "this process can look through a mode 000 directory, so there is nothing to test",
        );
        return;
    }
    let listing = rig.list_with(&knowledge);
    let checked = listing
        .not_checked
        .iter()
        .find(|n| n.layer == "managed")
        .unwrap_or_else(|| panic!("the managed layer is named as not checked: {listing:?}"));
    assert!(
        checked.reason.contains("PermissionDenied"),
        "{}",
        checked.reason
    );
    assert!(
        checked.reason.contains(&file.display().to_string()),
        "{}",
        checked.reason
    );
    assert!(listing.sources.is_empty(), "{:?}", listing.sources);
    assert!(!format!("{listing:?}").contains(SECRET));
}

/// A managed first-wins candidate that cannot be checked might deliver, so it leaves its
/// rank and everything below undecided, as an unreadable file does. It is listed only as
/// not checked.
#[test]
fn a_first_wins_candidate_that_cannot_be_checked_leaves_the_lower_ranks_undecided() {
    let rig = Rig::new();
    let knowledge = fixture_with(&item(
        "demo.locked-top",
        "/locked/top.json",
        1,
        Some(r#"["linux"]"#),
    ));
    let hidden = rig.file("locked/top.json", SECRET);
    let _locked = Locked::new(hidden.parent().expect("a parent").to_path_buf());
    if fs::metadata(&hidden).is_ok() {
        usher_testkit::skip(
            "a_first_wins_candidate_that_cannot_be_checked_leaves_the_lower_ranks_undecided",
            "this process can look through a mode 000 directory, so there is nothing to test",
        );
        return;
    }
    let top = rig.file(TOP, POLICY);
    let middle = rig.file(MIDDLE, POLICY);
    let listing = rig.list_with(&knowledge);
    assert_eq!(listing.sources.len(), 2, "{:?}", listing.sources);
    let cause = format!(
        "first-wins not decided: {} could not be checked",
        hidden.display()
    );
    for path in [&top, &middle] {
        assert_eq!(
            status_of(&listing, path),
            &SourceStatus::Unknown {
                reason: cause.clone()
            },
            "{}",
            path.display()
        );
    }
    assert!(
        listing.not_checked.iter().any(|n| n.layer == "managed"),
        "{:?}",
        listing.not_checked
    );
    assert!(!format!("{listing:?}").contains(SECRET));
}

#[test]
fn a_first_wins_candidate_that_cannot_be_checked_below_a_winner_changes_nothing_above_it() {
    let rig = Rig::new();
    let knowledge = fixture_with(&item(
        "demo.locked-below",
        "/locked/below.json",
        5,
        Some(r#"["linux"]"#),
    ));
    let hidden = rig.file("locked/below.json", SECRET);
    let _locked = Locked::new(hidden.parent().expect("a parent").to_path_buf());
    if fs::metadata(&hidden).is_ok() {
        usher_testkit::skip(
            "a_first_wins_candidate_that_cannot_be_checked_below_a_winner_changes_nothing_above_it",
            "this process can look through a mode 000 directory, so there is nothing to test",
        );
        return;
    }
    let top = rig.file(TOP, POLICY);
    let low = rig.file(LOW, POLICY);
    let listing = rig.list_with(&knowledge);
    assert_eq!(status_of(&listing, &top), &SourceStatus::InEffect);
    assert_eq!(
        status_of(&listing, &low),
        &SourceStatus::Shadowed { by: top }
    );
    assert!(
        listing.not_checked.iter().any(|n| n.layer == "managed"),
        "{:?}",
        listing.not_checked
    );
}
