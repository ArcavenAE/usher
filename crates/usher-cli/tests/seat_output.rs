//! Story 1.5, PR D: what `usher inventory` prints and records for the seat. The block
//! follows the sources, one line per seat item, each ending with its knowledge item.
//! Output and store agree, and no value, no credential-key content and no environment
//! value appears in either. Scratch home, scratch system root.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use usher_cli::{InventoryEnv, LaunchEnvironment, inventory, inventory_with};
use usher_core::SystemRoot;
use usher_knowledge::{Knowledge, Platform, TrustedKeys};
use usher_store::{SeatRow, Store};
use usher_testkit::{ScratchHome, Signer};

const FIXTURE: &str = r#"
schema_version = 4
sequence = 99

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
"#;
const LAUNCHER: &str = r#"
[[item]]
id = "claude-code.native-launcher"
harness = "claude-code"
layer = "slow"
class = "paths"
versions = ">=2.1.317, <3.0.0"
summary = "The native installer's launcher is a symlink to a file named by its version."
reference = { kind = "vendor-docs", locator = "https://code.claude.com/docs/en/setup" }
last_verified = 2026-11-20
version_from = { launcher = "${HOME}/.local/bin/claude", rule = "symlink-target-last-component", layout = "${HOME}/.local/share/claude/versions/${VERSION}" }
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
        fs::create_dir_all(repo.join(".git")).expect("repo");
        let target = scratch.home().join(".local/share/claude/versions/2.1.317");
        fs::create_dir_all(target.parent().expect("a parent")).expect("dirs");
        fs::write(&target, "binary").expect("binary");
        fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).expect("mode");
        let launcher = scratch.home().join(".local/bin/claude");
        fs::create_dir_all(launcher.parent().expect("a parent")).expect("dirs");
        symlink(&target, launcher).expect("symlink");
        Rig {
            scratch,
            system,
            repo,
        }
    }

    fn home(&self) -> PathBuf {
        self.scratch.home().to_path_buf()
    }

    fn env(&self, pairs: &[(&str, &str)]) -> InventoryEnv {
        InventoryEnv {
            host: "host-1".to_string(),
            user: "alex".to_string(),
            home: self.home(),
            launch_dir: self.repo.clone(),
            launch: LaunchEnvironment::from_pairs(pairs),
            path: Some(self.home().join(".local/bin").into_os_string()),
            state_dir: self.home().join(".local/state/usher"),
            now: "2026-11-20T12:00:00Z".to_string(),
            platform: Platform::Linux,
        }
    }

    fn write(&self, path: &Path, text: &str) -> PathBuf {
        fs::create_dir_all(path.parent().expect("a parent")).expect("dirs");
        fs::write(path, text).expect("file");
        path.to_path_buf()
    }

    fn fixture_run(&self, env: &InventoryEnv) -> String {
        let signer = Signer::new();
        let trusted = TrustedKeys::for_tests(&signer.public_key_base64(), None).expect("keys");
        let mut knowledge = Knowledge::embedded().expect("embedded");
        let text = format!("{FIXTURE}{LAUNCHER}");
        knowledge
            .update(
                text.as_bytes(),
                &signer.sign(text.as_bytes(), "cli fixture"),
                &trusted,
            )
            .expect("a signed update");
        inventory_with(env, SystemRoot::at(self.system.clone()), &knowledge).expect("an inventory")
    }

    fn rows(&self) -> Vec<SeatRow> {
        let store = Store::open(&self.env(&[]).state_dir).expect("the store opens");
        let runs = store.runs().expect("runs");
        store
            .seat_of(runs.last().expect("a run").id)
            .expect("seat rows")
    }
}

fn line_for<'a>(out: &'a str, id: &str) -> &'a str {
    let tail = format!("(knowledge item {id})");
    out.lines()
        .find(|line| line.ends_with(&tail))
        .unwrap_or_else(|| panic!("no seat line ends with `{tail}` in:\n{out}"))
}

fn row<'a>(rows: &'a [SeatRow], id: &str) -> &'a SeatRow {
    rows.iter()
        .find(|row| row.item_id == id)
        .unwrap_or_else(|| panic!("no stored seat row for {id}: {rows:?}"))
}

fn seg(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| (*p).to_string()).collect()
}

#[test]
fn a_presence_item_prints_and_stores_present_or_absent() {
    let rig = Rig::new();
    let path = rig.home().join(".fleet/client-env.sh");
    let out = rig.fixture_run(&rig.env(&[]));
    assert_eq!(
        line_for(&out, "claude-code.fx-client"),
        format!(
            "fleet-client  {}  absent (knowledge item claude-code.fx-client)",
            path.display()
        )
    );
    assert_eq!(
        row(&rig.rows(), "claude-code.fx-client").present,
        Some(false)
    );
    rig.write(&path, "SECRET_TOKEN=abc123\n");
    let out = rig.fixture_run(&rig.env(&[]));
    assert_eq!(
        line_for(&out, "claude-code.fx-client"),
        format!(
            "fleet-client  {}  present (knowledge item claude-code.fx-client)",
            path.display()
        )
    );
    assert_eq!(
        row(&rig.rows(), "claude-code.fx-client").present,
        Some(true)
    );
    assert!(!out.contains("abc123"));
}

#[test]
fn a_listing_item_prints_and_stores_entry_names_only() {
    let rig = Rig::new();
    rig.write(&rig.home().join(".packs/zeta/inner-secret.txt"), "x");
    rig.write(&rig.home().join(".packs/alpha/readme.md"), "x");
    let out = rig.fixture_run(&rig.env(&[]));
    let path = rig.home().join(".packs");
    assert_eq!(
        line_for(&out, "claude-code.fx-packs"),
        format!(
            "packs  {}  present (entries: alpha, zeta) (knowledge item claude-code.fx-packs)",
            path.display()
        )
    );
    let rows = rig.rows();
    let stored = row(&rows, "claude-code.fx-packs");
    assert_eq!(stored.present, Some(true));
    assert_eq!(stored.names, vec![seg(&["alpha"]), seg(&["zeta"])]);
    assert!(!out.contains("inner-secret"));
}

#[test]
fn a_names_item_prints_quoted_dotted_paths_and_stores_the_segments() {
    let rig = Rig::new();
    let path = rig.write(
        &rig.home().join(".state.json"),
        r#"{"mcpServers":{"z":{},"a.b":{"env":{"K":"env-secret"}}}}"#,
    );
    let out = rig.fixture_run(&rig.env(&[]));
    assert_eq!(
        line_for(&out, "claude-code.fx-servers"),
        format!(
            "mcp  {}  present (names: mcpServers.\"a.b\", mcpServers.\"a.b\".env.K, mcpServers.z) (knowledge item claude-code.fx-servers)",
            path.display()
        )
    );
    let rows = rig.rows();
    assert_eq!(
        row(&rows, "claude-code.fx-servers").names,
        vec![
            seg(&["mcpServers", "a.b"]),
            seg(&["mcpServers", "a.b", "env", "K"]),
            seg(&["mcpServers", "z"]),
        ]
    );
    assert!(!out.contains("env-secret") && !format!("{rows:?}").contains("env-secret"));
}

#[test]
fn an_env_item_prints_each_variable_present_or_absent_and_never_a_value() {
    let rig = Rig::new();
    let out = rig.fixture_run(&rig.env(&[("FLEET_TOKEN", "sk-live-secret")]));
    assert_eq!(
        line_for(&out, "claude-code.fx-env"),
        "auth-route  env  FLEET_API_KEY absent, FLEET_TOKEN present (knowledge item claude-code.fx-env)"
    );
    let rows = rig.rows();
    let stored = row(&rows, "claude-code.fx-env");
    assert_eq!(stored.present, None);
    assert!(stored.names.is_empty());
    assert_eq!(
        stored.members,
        [
            ("FLEET_API_KEY".to_string(), false),
            ("FLEET_TOKEN".to_string(), true)
        ]
    );
    assert!(!out.contains("sk-live-secret") && !format!("{rows:?}").contains("sk-live-secret"));
}

#[test]
fn a_keys_present_item_prints_membership_and_stores_nothing_under_the_key() {
    let rig = Rig::new();
    let path = rig.write(
        &rig.home().join(".state.json"),
        r#"{"oauthAccount":{"emailAddress":"alex@example.test"}}"#,
    );
    let out = rig.fixture_run(&rig.env(&[]));
    assert_eq!(
        line_for(&out, "claude-code.fx-keys"),
        format!(
            "auth-route  {}  oauthAccount present (knowledge item claude-code.fx-keys)",
            path.display()
        )
    );
    let rows = rig.rows();
    let stored = row(&rows, "claude-code.fx-keys");
    assert_eq!(stored.present, Some(true));
    assert!(stored.names.is_empty());
    assert_eq!(stored.members, [("oauthAccount".to_string(), true)]);
    let all = format!("{out}{rows:?}");
    for leaked in ["emailAddress", "alex@example.test"] {
        assert!(!all.contains(leaked), "{leaked}");
    }
}

#[test]
fn a_keys_present_item_over_a_missing_file_prints_absent_for_the_whole_line() {
    let rig = Rig::new();
    let path = rig.home().join(".state.json");
    let out = rig.fixture_run(&rig.env(&[]));
    assert_eq!(
        line_for(&out, "claude-code.fx-keys"),
        format!(
            "auth-route  {}  absent (knowledge item claude-code.fx-keys)",
            path.display()
        )
    );
    let rows = rig.rows();
    let stored = row(&rows, "claude-code.fx-keys");
    assert_eq!(stored.present, Some(false));
    assert_eq!(stored.members, [("oauthAccount".to_string(), false)]);
}

#[test]
fn the_seat_block_follows_the_sources_and_the_seed_lines_end_with_their_items() {
    let rig = Rig::new();
    rig.write(&rig.home().join(".claude/settings.json"), "{}");
    rig.write(
        &rig.home().join(".claude.json"),
        r#"{"oauthAccount":{"emailAddress":"alex@example.test"}}"#,
    );
    let out = inventory(
        &rig.env(&[("ANTHROPIC_API_KEY", "sk-ant-secret")]),
        SystemRoot::at(rig.system.clone()),
    )
    .expect("an inventory");
    let lines: Vec<&str> = out.lines().collect();
    let source = lines
        .iter()
        .position(|l| l.contains(".claude/settings.json"))
        .expect("a source line");
    let env = lines
        .iter()
        .position(|l| l.ends_with("(knowledge item claude-code.seat-auth-env)"))
        .expect("the auth env line");
    assert!(source < env, "{out}");
    assert!(
        env > 0 && lines[env].contains("ANTHROPIC_API_KEY present"),
        "{out}"
    );
    assert!(lines[env].contains("ANTHROPIC_AUTH_TOKEN absent"), "{out}");
    let account = line_for(&out, "claude-code.seat-auth-account");
    assert!(account.contains("oauthAccount present"), "{account}");
    for leaked in ["sk-ant-secret", "emailAddress", "alex@example.test"] {
        assert!(!out.contains(leaked), "{leaked}");
    }
}

#[test]
fn an_empty_environment_value_counts_as_not_set() {
    let rig = Rig::new();
    let out = rig.fixture_run(&rig.env(&[("FLEET_TOKEN", ""), ("FLEET_API_KEY", "k")]));
    assert_eq!(
        line_for(&out, "claude-code.fx-env"),
        "auth-route  env  FLEET_API_KEY present, FLEET_TOKEN absent (knowledge item claude-code.fx-env)"
    );
    let rows = rig.rows();
    assert_eq!(
        row(&rows, "claude-code.fx-env").members,
        [
            ("FLEET_API_KEY".to_string(), true),
            ("FLEET_TOKEN".to_string(), false)
        ]
    );
}

#[test]
fn two_differing_mcp_json_files_print_two_lines_with_names_first_then_the_stamp() {
    let rig = Rig::new();
    let launch = rig.repo.join("pkg");
    fs::create_dir_all(&launch).expect("dirs");
    let repo_file = rig.write(&rig.repo.join(".mcp.json"), r#"{"mcpServers":{"s2":{}}}"#);
    let launch_file = rig.write(&launch.join(".mcp.json"), r#"{"mcpServers":{"s1":{}}}"#);
    let mut env = rig.env(&[]);
    env.launch_dir = launch;
    let out = inventory(&env, SystemRoot::at(rig.system.clone())).expect("an inventory");
    let stamp = "which of the two Claude Code loads from a subdirectory launch is not stated";
    assert_eq!(
        line_for(&out, "claude-code.seat-mcp-launch-dir"),
        format!(
            "mcp  {}  present (names: mcpServers.s1; {stamp}) (knowledge item claude-code.seat-mcp-launch-dir)",
            launch_file.display()
        )
    );
    assert_eq!(
        line_for(&out, "claude-code.seat-mcp-repo-root"),
        format!(
            "mcp  {}  present (names: mcpServers.s2; {stamp}) (knowledge item claude-code.seat-mcp-repo-root)",
            repo_file.display()
        )
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

#[test]
fn a_seat_path_under_an_unreadable_directory_prints_not_checked_in_plain_words() {
    let rig = Rig::new();
    let file = rig.write(&rig.home().join(".fleet/client-env.sh"), "SECRET=x");
    let _locked = Locked::new(file.parent().expect("a parent").to_path_buf());
    if fs::metadata(&file).is_ok() {
        usher_testkit::skip(
            "a_seat_path_under_an_unreadable_directory_prints_not_checked_in_plain_words",
            "this process can look through a mode 000 directory, so there is nothing to test",
        );
        return;
    }
    let out = rig.fixture_run(&rig.env(&[]));
    assert_eq!(
        line_for(&out, "claude-code.fx-client"),
        format!(
            "fleet-client  {}  not checked (permission denied) (knowledge item claude-code.fx-client)",
            file.display()
        )
    );
    assert!(!out.contains("PermissionDenied"), "{out}");
    let stored = rig.rows();
    let row = row(&stored, "claude-code.fx-client");
    assert_eq!(row.present, None);
    assert_eq!(row.note.as_deref(), Some("permission denied"));
}
