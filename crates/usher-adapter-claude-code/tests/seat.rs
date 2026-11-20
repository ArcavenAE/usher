//! Story 1.5, PR D: what the Claude Code adapter reads of the seat around the harness.
//! Each item's read rule says how much: presence, one directory listing, key names at
//! listed key paths, whether named variables are set, or whether top-level keys exist.
//! Nothing beyond the rule is read, and no value or credential-key content is held.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use usher_adapter_claude_code::ClaudeCodeAdapter;
use usher_core::{
    LaunchContext, SeatLine, SourceLister, SourceListing, SystemRoot, environment_key,
};
use usher_knowledge::{KnowledgeFile, Platform};
use usher_testkit::ScratchHome;

const VERSION: &str = "2.1.317";
const NOT_STATED: &str =
    "which of the two Claude Code loads from a subdirectory launch is not stated";

/// One item of each read rule, over files the tests write under the scratch home.
const FIXTURE: &str = r#"
schema_version = 4
sequence = 20

[[item]]
id = "claude-code.fx-client"
harness = "claude-code"
layer = "slow"
class = "seat"
versions = ">=2.1.317, <3.0.0"
summary = "A client file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/client" }
last_verified = 2026-11-20
seat = { part = "fleet-client", path = "${HOME}/.fleet/client-env.sh", read = "presence" }

[[item]]
id = "claude-code.fx-packs"
harness = "claude-code"
layer = "slow"
class = "seat"
versions = ">=2.1.317, <3.0.0"
summary = "A pack directory."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/packs" }
last_verified = 2026-11-20
seat = { part = "packs", path = "${HOME}/.packs", read = "listing" }

[[item]]
id = "claude-code.fx-servers"
harness = "claude-code"
layer = "slow"
class = "seat"
versions = ">=2.1.317, <3.0.0"
summary = "Server names."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/mcp" }
last_verified = 2026-11-20
seat = { part = "mcp", path = "${HOME}/.state.json", read = "names", key_paths = ["mcpServers.*", "mcpServers.*.env", "mcpServers.*.headers"] }

[[item]]
id = "claude-code.fx-wide"
harness = "claude-code"
layer = "slow"
class = "seat"
versions = ">=2.1.317, <3.0.0"
summary = "A wildcard that can meet a credential key."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/wide" }
last_verified = 2026-11-20
seat = { part = "toolchain", path = "${HOME}/.wide.json", read = "names", key_paths = ["servers.*", "servers.*.env"] }

[[item]]
id = "claude-code.fx-env"
harness = "claude-code"
layer = "slow"
class = "seat"
versions = ">=2.1.317, <3.0.0"
summary = "Two variables."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/env" }
last_verified = 2026-11-20
seat = { part = "auth-route", read = "env", variables = ["FLEET_API_KEY", "FLEET_TOKEN"] }

[[item]]
id = "claude-code.fx-keys"
harness = "claude-code"
layer = "slow"
class = "seat"
versions = ">=2.1.317, <3.0.0"
summary = "A login marker."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/keys" }
last_verified = 2026-11-20
seat = { part = "auth-route", path = "${HOME}/.state.json", read = "keys-present", keys = ["oauthAccount"] }

[[item]]
id = "claude-code.fx-deep"
harness = "claude-code"
layer = "slow"
class = "seat"
versions = ">=2.1.317, <3.0.0"
summary = "A wildcard over a nested key."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/deep" }
last_verified = 2026-11-20
seat = { part = "toolchain", path = "${HOME}/.deep.json", read = "names", key_paths = ["servers.*.env"] }

[[item]]
id = "claude-code.fx-plain"
harness = "claude-code"
layer = "slow"
class = "seat"
versions = ">=2.1.317, <3.0.0"
summary = "A plain wildcard."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/plain" }
last_verified = 2026-11-20
seat = { part = "toolchain", path = "${HOME}/.plain.json", read = "names", key_paths = ["servers.*"] }
"#;

struct Rig {
    scratch: ScratchHome,
    system: PathBuf,
    repo: PathBuf,
}

impl Rig {
    fn new() -> Self {
        let scratch = ScratchHome::new().expect("scratch");
        let system = scratch.root().join("system");
        let repo = scratch.home().join("work/repo");
        fs::create_dir_all(&system).expect("system root");
        fs::create_dir_all(repo.join("pkg")).expect("repo");
        Rig {
            scratch,
            system,
            repo,
        }
    }

    fn home(&self) -> PathBuf {
        self.scratch.home().to_path_buf()
    }

    fn context(&self, launch_dir: &Path, set: &[&str]) -> LaunchContext {
        let home = self.home();
        let key = environment_key("host", "alex", "claude-code", launch_dir, &[])
            .expect("an absolute directory");
        LaunchContext {
            key,
            config_dir: home.join(".claude"),
            global_state_dir: home.clone(),
            set_variables: set
                .iter()
                .map(|s| (*s).to_string())
                .collect::<BTreeSet<_>>(),
            home,
            repo: Some(self.repo.clone()),
            user: "alex".to_string(),
            harness_version: Some(VERSION.to_string()),
        }
    }

    fn list_with(&self, knowledge: KnowledgeFile, context: &LaunchContext) -> SourceListing {
        ClaudeCodeAdapter::new(
            knowledge,
            SystemRoot::at(self.system.clone()),
            Platform::Linux,
        )
        .list_sources(context)
        .expect("a listing")
    }

    fn fixture(&self, context: &LaunchContext) -> SourceListing {
        self.list_with(KnowledgeFile::parse(FIXTURE).expect("fixture"), context)
    }

    fn seed(&self, context: &LaunchContext) -> SourceListing {
        self.list_with(KnowledgeFile::embedded().expect("embedded"), context)
    }

    fn write(&self, path: &Path, text: &str) -> PathBuf {
        fs::create_dir_all(path.parent().expect("a parent")).expect("dirs");
        fs::write(path, text).expect("file");
        path.to_path_buf()
    }
}

fn line<'a>(listing: &'a SourceListing, id: &str) -> &'a SeatLine {
    listing
        .seat
        .iter()
        .find(|line| line.item_id == id)
        .unwrap_or_else(|| panic!("no seat line for {id}: {:?}", listing.seat))
}

fn seg(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| (*p).to_string()).collect()
}

#[test]
fn a_presence_item_says_present_or_absent_and_reads_nothing_else() {
    let rig = Rig::new();
    let context = rig.context(&rig.repo, &[]);
    let absent = rig.fixture(&context);
    let l = line(&absent, "claude-code.fx-client");
    assert_eq!(
        (l.part.as_str(), l.read.as_str()),
        ("fleet-client", "presence")
    );
    assert_eq!(l.present, Some(false));
    assert_eq!(
        l.path.as_deref(),
        Some(rig.home().join(".fleet/client-env.sh").as_path())
    );
    rig.write(
        &rig.home().join(".fleet/client-env.sh"),
        "SECRET_TOKEN=abc123\n",
    );
    let present = rig.fixture(&context);
    let l = line(&present, "claude-code.fx-client");
    assert_eq!(l.present, Some(true));
    assert!(l.entries.is_empty() && l.names.is_empty() && l.members.is_empty());
    assert!(
        !format!("{present:?}").contains("abc123"),
        "the file is not read"
    );
}

#[test]
fn a_listing_item_names_the_entries_one_level_deep_and_no_more() {
    let rig = Rig::new();
    rig.write(&rig.home().join(".packs/zeta/inner-secret.txt"), "x");
    rig.write(&rig.home().join(".packs/alpha/readme.md"), "x");
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&listing, "claude-code.fx-packs");
    assert_eq!(l.present, Some(true));
    assert_eq!(l.entries, ["alpha", "zeta"]);
    assert!(
        !format!("{listing:?}").contains("inner-secret"),
        "no descent"
    );
}

#[test]
fn a_listing_item_over_a_missing_directory_is_absent() {
    let rig = Rig::new();
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&listing, "claude-code.fx-packs");
    assert_eq!(l.present, Some(false));
    assert!(l.entries.is_empty());
}

#[test]
fn a_names_item_lists_concrete_paths_sorted_by_segments_and_never_a_value() {
    let rig = Rig::new();
    rig.write(
        &rig.home().join(".state.json"),
        r#"{"mcpServers":{"z":{},"a.b":{"command":"run-secret","env":{"K":"env-secret"},"headers":{"H":"hdr-secret"}}},"other":1}"#,
    );
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&listing, "claude-code.fx-servers");
    assert_eq!(l.present, Some(true));
    assert_eq!(
        l.names,
        vec![
            seg(&["mcpServers", "a.b"]),
            seg(&["mcpServers", "a.b", "env", "K"]),
            seg(&["mcpServers", "a.b", "headers", "H"]),
            seg(&["mcpServers", "z"]),
        ]
    );
    let text = format!("{listing:?}");
    for value in ["run-secret", "env-secret", "hdr-secret"] {
        assert!(!text.contains(value), "{value} is held nowhere");
    }
}

#[test]
fn a_wildcard_never_descends_into_a_credential_key() {
    let rig = Rig::new();
    rig.write(
        &rig.home().join(".wide.json"),
        r#"{"servers":{"ok":{"env":{"FINE":"v1"}},"oauthAccount":{"env":{"SECRET_NAME":"v2"},"emailAddress":"e@x.test"}}}"#,
    );
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&listing, "claude-code.fx-wide");
    assert!(
        l.names.contains(&seg(&["servers", "ok", "env", "FINE"])),
        "{:?}",
        l.names
    );
    for name in &l.names {
        assert!(
            !(name.len() > 2 && name[1] == "oauthAccount"),
            "nothing is listed under a credential key: {name:?}"
        );
    }
    let text = format!("{listing:?}");
    for leaked in ["SECRET_NAME", "emailAddress", "e@x.test", "v2"] {
        assert!(!text.contains(leaked), "{leaked} appears nowhere");
    }
}

#[test]
fn a_names_item_over_a_file_that_does_not_parse_lists_no_names_and_no_content() {
    let rig = Rig::new();
    rig.write(
        &rig.home().join(".state.json"),
        r#"{"mcpServers": oops-secret"#,
    );
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&listing, "claude-code.fx-servers");
    assert_eq!(l.present, Some(true));
    assert!(l.names.is_empty());
    assert!(!format!("{listing:?}").contains("oops-secret"));
}

#[test]
fn an_env_item_reports_which_listed_variables_are_set_in_listed_order() {
    let rig = Rig::new();
    let none = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&none, "claude-code.fx-env");
    assert_eq!((l.part.as_str(), l.read.as_str()), ("auth-route", "env"));
    assert!(l.path.is_none());
    assert_eq!(l.present, None, "an environment read has no file");
    assert_eq!(
        l.members,
        [
            ("FLEET_API_KEY".to_string(), false),
            ("FLEET_TOKEN".to_string(), false)
        ]
    );
    let one = rig.fixture(&rig.context(&rig.repo, &["FLEET_TOKEN", "UNLISTED_VARIABLE"]));
    let l = line(&one, "claude-code.fx-env");
    assert_eq!(l.present, None, "an environment read has no file");
    assert_eq!(
        l.members,
        [
            ("FLEET_API_KEY".to_string(), false),
            ("FLEET_TOKEN".to_string(), true)
        ]
    );
    assert!(
        !format!("{one:?}").contains("UNLISTED_VARIABLE"),
        "only listed names"
    );
}

#[test]
fn a_keys_present_item_gives_membership_only_and_nothing_under_the_key() {
    let rig = Rig::new();
    rig.write(
        &rig.home().join(".state.json"),
        r#"{"oauthAccount":{"emailAddress":"alex@example.test","organizationName":"Org"},"x":1}"#,
    );
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&listing, "claude-code.fx-keys");
    assert_eq!(l.present, Some(true));
    assert_eq!(l.members, [("oauthAccount".to_string(), true)]);
    assert!(l.names.is_empty());
    let text = format!("{listing:?}");
    for leaked in [
        "emailAddress",
        "alex@example.test",
        "organizationName",
        "Org\"",
    ] {
        assert!(!text.contains(leaked), "{leaked} appears nowhere");
    }
}

#[test]
fn a_keys_present_item_over_a_missing_file_is_absent_with_its_key_absent() {
    let rig = Rig::new();
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&listing, "claude-code.fx-keys");
    assert_eq!(l.present, Some(false));
    assert_eq!(l.members, [("oauthAccount".to_string(), false)]);
}

#[test]
fn a_file_without_the_key_reads_the_key_absent_with_the_file_present() {
    let rig = Rig::new();
    rig.write(&rig.home().join(".state.json"), r#"{"projects":{}}"#);
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&listing, "claude-code.fx-keys");
    assert_eq!(l.present, Some(true));
    assert_eq!(l.members, [("oauthAccount".to_string(), false)]);
}

#[test]
fn an_item_for_another_version_range_or_harness_adds_no_line() {
    let rig = Rig::new();
    let mut context = rig.context(&rig.repo, &[]);
    context.harness_version = Some("3.0.0".to_string());
    let listing = rig.fixture(&context);
    assert!(listing.seat.is_empty(), "{:?}", listing.seat);
}

fn mcp_lines(listing: &SourceListing) -> Vec<&SeatLine> {
    listing
        .seat
        .iter()
        .filter(|l| l.part == "mcp" && l.path.as_deref().is_some_and(|p| p.ends_with(".mcp.json")))
        .collect()
}

#[test]
fn the_seed_lists_both_mcp_json_files_with_a_blind_spot_stamp_when_both_exist_and_differ() {
    let rig = Rig::new();
    let launch = rig.repo.join("pkg");
    let root = rig.write(&rig.repo.join(".mcp.json"), "{}");
    let start = rig.write(&launch.join(".mcp.json"), "{}");
    let listing = rig.seed(&rig.context(&launch, &[]));
    let lines = mcp_lines(&listing);
    assert_eq!(lines.len(), 2, "{:?}", listing.seat);
    let paths: Vec<&Path> = lines.iter().filter_map(|l| l.path.as_deref()).collect();
    assert!(paths.contains(&root.as_path()) && paths.contains(&start.as_path()));
    for l in lines {
        assert_eq!(l.present, Some(true));
        assert_eq!(l.note.as_deref(), Some(NOT_STATED), "{l:?}");
    }
}

#[test]
fn the_seed_lists_one_mcp_json_line_when_the_two_paths_are_equal() {
    let rig = Rig::new();
    let root = rig.write(&rig.repo.join(".mcp.json"), "{}");
    let listing = rig.seed(&rig.context(&rig.repo, &[]));
    let lines = mcp_lines(&listing);
    assert_eq!(lines.len(), 1, "{:?}", listing.seat);
    assert_eq!(lines[0].path.as_deref(), Some(root.as_path()));
    assert_eq!(lines[0].present, Some(true));
    assert!(lines[0].note.is_none());
}

#[test]
fn the_seed_carries_no_stamp_when_only_one_of_two_differing_mcp_json_files_exists() {
    let rig = Rig::new();
    let launch = rig.repo.join("pkg");
    rig.write(&launch.join(".mcp.json"), "{}");
    let listing = rig.seed(&rig.context(&launch, &[]));
    let lines = mcp_lines(&listing);
    assert_eq!(lines.len(), 2, "{:?}", listing.seat);
    assert_eq!(lines.iter().filter(|l| l.present == Some(true)).count(), 1);
    assert!(lines.iter().all(|l| l.note.is_none()), "{lines:?}");
}

#[test]
fn the_seed_reads_the_auth_route_by_variable_presence_and_the_login_key_by_membership() {
    let rig = Rig::new();
    let global = rig.write(
        &rig.home().join(".claude.json"),
        r#"{"oauthAccount":{"emailAddress":"alex@example.test"}}"#,
    );
    let listing = rig.seed(&rig.context(&rig.repo, &["ANTHROPIC_API_KEY"]));
    let env = line(&listing, "claude-code.seat-auth-env");
    assert_eq!(env.present, None);
    assert!(
        env.members
            .contains(&("ANTHROPIC_API_KEY".to_string(), true))
    );
    assert!(
        env.members
            .contains(&("ANTHROPIC_AUTH_TOKEN".to_string(), false))
    );
    let account = line(&listing, "claude-code.seat-auth-account");
    assert_eq!(account.path.as_deref(), Some(global.as_path()));
    assert_eq!(account.members, [("oauthAccount".to_string(), true)]);
    assert!(!format!("{listing:?}").contains("alex@example.test"));
}

#[test]
fn a_wildcard_lists_a_credential_childs_own_name_and_reads_nothing_under_it() {
    let rig = Rig::new();
    rig.write(
        &rig.home().join(".deep.json"),
        r#"{"servers":{"oauthAccount":{"env":{"K":"v"}},"ok":{"env":{"K":"v2"}}}}"#,
    );
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&listing, "claude-code.fx-deep");
    assert!(
        l.names.contains(&seg(&["servers", "oauthAccount"])),
        "{:?}",
        l.names
    );
    assert!(
        l.names.contains(&seg(&["servers", "ok", "env", "K"])),
        "{:?}",
        l.names
    );
    assert!(
        !l.names
            .iter()
            .any(|n| n.len() > 2 && n[0] == "servers" && n[1] == "oauthAccount"),
        "no env and no K under the credential child: {:?}",
        l.names
    );
    assert!(!format!("{listing:?}").contains("\"v\""), "no value");
}

#[test]
fn a_plain_wildcard_lists_the_credential_childs_name() {
    let rig = Rig::new();
    rig.write(
        &rig.home().join(".plain.json"),
        r#"{"servers":{"oauthAccount":{"secret-inner":"v"},"ok":{}}}"#,
    );
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&listing, "claude-code.fx-plain");
    assert_eq!(
        l.names,
        vec![seg(&["servers", "oauthAccount"]), seg(&["servers", "ok"])]
    );
    assert!(!format!("{listing:?}").contains("secret-inner"));
}

#[test]
fn a_credential_key_at_any_depth_is_listed_by_name_only() {
    let rig = Rig::new();
    rig.write(
        &rig.home().join(".deep.json"),
        r#"{"servers":{"a":{"env":{"oauthAccount":{"INNER":"v"}}}}}"#,
    );
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let l = line(&listing, "claude-code.fx-deep");
    assert!(
        l.names
            .contains(&seg(&["servers", "a", "env", "oauthAccount"])),
        "{:?}",
        l.names
    );
    assert!(!format!("{listing:?}").contains("INNER"));
}

#[test]
fn seat_lines_follow_the_part_declaration_order_then_the_item_id() {
    let rig = Rig::new();
    let listing = rig.fixture(&rig.context(&rig.repo, &[]));
    let ids: Vec<&str> = listing
        .seat
        .iter()
        .map(|l| l.item_id.as_str())
        .filter(|id| id.starts_with("claude-code.fx-"))
        .collect();
    assert_eq!(
        ids,
        [
            "claude-code.fx-servers",
            "claude-code.fx-packs",
            "claude-code.fx-client",
            "claude-code.fx-env",
            "claude-code.fx-keys",
            "claude-code.fx-deep",
            "claude-code.fx-plain",
            "claude-code.fx-wide",
        ]
    );
}

#[test]
fn the_seed_names_its_two_mcp_json_items_by_where_they_look() {
    let rig = Rig::new();
    let launch = rig.repo.join("pkg");
    rig.write(
        &rig.repo.join(".mcp.json"),
        r#"{"mcpServers":{"repo-server":{}}}"#,
    );
    rig.write(
        &launch.join(".mcp.json"),
        r#"{"mcpServers":{"launch-server":{}}}"#,
    );
    let listing = rig.seed(&rig.context(&launch, &[]));
    let by_launch = line(&listing, "claude-code.seat-mcp-launch-dir");
    let by_repo = line(&listing, "claude-code.seat-mcp-repo-root");
    assert_eq!(
        by_launch.path.as_deref(),
        Some(launch.join(".mcp.json").as_path())
    );
    assert_eq!(
        by_repo.path.as_deref(),
        Some(rig.repo.join(".mcp.json").as_path())
    );
    assert_eq!(by_launch.names, vec![seg(&["mcpServers", "launch-server"])]);
    assert_eq!(by_repo.names, vec![seg(&["mcpServers", "repo-server"])]);
    assert_eq!(by_launch.note.as_deref(), Some(NOT_STATED));
    assert_eq!(by_repo.note.as_deref(), Some(NOT_STATED));
}
