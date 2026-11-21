//! Profile sources: a directory or a file read into profiles with their
//! fragments, under the reader's limits. Fixtures are generated in a scratch
//! directory per test and removed afterwards.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use usher_core::UsherError;
use usher_profiles::{FragmentFormat, HarnessRule, LoadRules, McpOrigin, Profile, Source};

const MANIFEST: &str = r#"name = "team-setup"
version = "2026.11"

[harnesses.claude-code]
versions = ">=2.1.317, <3.0.0"
"#;

const CLAUDE: &str = r#"{"permissions": {"allow": ["Bash(ls)"], "deny": []}, "model": "sonnet"}"#;

static NEXT: AtomicU32 = AtomicU32::new(0);

/// A scratch directory, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        let dir = std::env::temp_dir().join(format!(
            "usher-src-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).expect("create scratch");
        Scratch(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    /// Writes `text` at `rel` under the scratch directory, creating parents.
    fn write(&self, rel: &str, text: &str) -> PathBuf {
        let path = self.0.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parents");
        fs::write(&path, text).expect("write fixture");
        path
    }

    /// The smallest valid profile in `rel` (a directory relative to scratch).
    fn profile(&self, rel: &str) -> PathBuf {
        self.write(&format!("{rel}/profile.toml"), MANIFEST);
        self.write(&format!("{rel}/claude-code/settings.json"), CLAUDE);
        self.0.join(rel)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn rules() -> LoadRules {
    LoadRules::new(vec![
        HarnessRule::new("claude-code", "settings.json", FragmentFormat::Json)
            .with_mcp_path("mcpServers"),
        HarnessRule::new("codex", "config.toml", FragmentFormat::Toml).with_mcp_path("mcp_servers"),
        HarnessRule::new("goose", "config.yaml", FragmentFormat::Yaml),
    ])
    .with_credential_names(&["token", "api_key", "credentials"])
}

fn load_dir(dir: &Path) -> Result<Vec<Profile>, UsherError> {
    Profile::load(&Source::Directory(dir.to_path_buf()), &rules())
}

fn must_load(dir: &Path) -> Vec<Profile> {
    load_dir(dir).unwrap_or_else(|err| panic!("a valid profile was refused: {}", err.status()))
}

/// The refusal for `dir`; the control is the smallest valid profile loading
/// from its own scratch, so a reader that refuses everything cannot pass.
fn refusal(dir: &Path) -> UsherError {
    let control = Scratch::new();
    must_load(&control.profile("control"));
    load_dir(dir).expect_err("this profile must be refused")
}

fn names(err: &UsherError, text: &str) {
    let all = format!(
        "{} | {} | {} | {}",
        err.step(),
        err.check(),
        err.status(),
        err.next_action()
    );
    assert!(
        all.contains(text),
        "the error does not name `{text}`: {all}"
    );
}

fn leaves(profile: &Profile, harness: &str) -> Vec<String> {
    profile
        .fragment(harness)
        .unwrap_or_else(|| panic!("no fragment for {harness}"))
        .leaves()
        .to_vec()
}

fn with_harness(extra: &str) -> String {
    format!("{MANIFEST}{extra}")
}

// A directory and a file source.

#[test]
fn a_directory_with_a_manifest_and_a_fragment_loads_as_one_profile() {
    let s = Scratch::new();
    let dir = s.profile("p");
    let profiles = must_load(&dir);
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].manifest().name(), "team-setup");
    assert_eq!(profiles[0].path(), dir.as_path());
    assert_eq!(
        leaves(&profiles[0], "claude-code"),
        ["permissions.allow", "permissions.deny", "model"]
    );
}

#[test]
fn a_file_source_is_the_path_to_profile_toml_and_its_directory_is_the_profile() {
    let s = Scratch::new();
    let dir = s.profile("p");
    let profiles = Profile::load(&Source::File(dir.join("profile.toml")), &rules())
        .unwrap_or_else(|e| panic!("a file source was refused: {}", e.status()));
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].path(), dir.as_path());
}

#[test]
fn a_file_source_that_is_not_named_profile_toml_is_refused() {
    let s = Scratch::new();
    let dir = s.profile("p");
    fs::copy(dir.join("profile.toml"), dir.join("other.toml")).expect("copy");
    must_load(&dir);
    let err = Profile::load(&Source::File(dir.join("other.toml")), &rules())
        .expect_err("only profile.toml names a profile");
    names(&err, "profile.toml");
}

#[test]
fn a_directory_with_neither_a_manifest_nor_profiles_is_refused_naming_profile_toml() {
    let s = Scratch::new();
    fs::create_dir_all(s.path().join("empty")).expect("mkdir");
    names(&refusal(&s.path().join("empty")), "profile.toml");
}

#[test]
fn values_are_kept_as_written_and_no_path_token_is_resolved() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/claude-code/settings.json",
        r#"{"statusLine": "${HOME}/bin/line"}"#,
    );
    let profiles = must_load(&dir);
    let fragment = profiles[0].fragment("claude-code").expect("fragment");
    assert_eq!(
        fragment.string_at("statusLine").as_deref(),
        Some("${HOME}/bin/line")
    );
}

// Fragments in three formats, and their leaves in the printed key-path form.

#[test]
fn a_toml_fragment_gives_its_leaves_and_an_array_is_a_leaf() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/profile.toml",
        &with_harness("\n[harnesses.codex]\nversions = \">=0.160.0\"\n"),
    );
    s.write(
        "p/codex/config.toml",
        "model = \"gpt\"\n[sandbox]\nmode = \"read-only\"\nwritable = [\"a\", \"b\"]\n",
    );
    let profiles = must_load(&dir);
    assert_eq!(
        leaves(&profiles[0], "codex"),
        ["model", "sandbox.mode", "sandbox.writable"]
    );
}

#[test]
fn a_yaml_fragment_gives_its_leaves() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/profile.toml",
        &with_harness("\n[harnesses.goose]\nversions = \">=1.0.0\"\n"),
    );
    s.write(
        "p/goose/config.yaml",
        "GOOSE_PROVIDER: openai\nextensions:\n  developer:\n    enabled: true\n",
    );
    let profiles = must_load(&dir);
    assert_eq!(
        leaves(&profiles[0], "goose"),
        ["GOOSE_PROVIDER", "extensions.developer.enabled"]
    );
}

#[test]
fn an_empty_yaml_file_reads_as_an_empty_mapping() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/profile.toml",
        &with_harness("\n[harnesses.goose]\nversions = \">=1.0.0\"\n"),
    );
    s.write("p/goose/config.yaml", "");
    let profiles = must_load(&dir);
    assert!(leaves(&profiles[0], "goose").is_empty());
}

#[test]
fn a_key_with_a_dot_is_written_as_a_json_string_segment() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/claude-code/settings.json",
        r#"{"env": {"A.B": "1", "plain": "2"}}"#,
    );
    let profiles = must_load(&dir);
    assert_eq!(
        leaves(&profiles[0], "claude-code"),
        ["env.\"A.B\"", "env.plain"]
    );
}

#[test]
fn a_fragment_that_does_not_parse_is_refused_naming_the_file() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write("p/claude-code/settings.json", "{not json");
    names(&refusal(&dir), "settings.json");
}

// A key set twice.

#[test]
fn a_repeated_key_in_a_json_fragment_is_refused_naming_the_path() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/claude-code/settings.json",
        r#"{"permissions": {"allow": ["a"], "allow": ["b"]}}"#,
    );
    names(&refusal(&dir), "permissions.allow");
}

#[test]
fn a_repeated_key_in_a_toml_fragment_is_refused() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/profile.toml",
        &with_harness("\n[harnesses.codex]\nversions = \">=0.160.0\"\n"),
    );
    s.write("p/codex/config.toml", "model = \"a\"\nmodel = \"b\"\n");
    names(&refusal(&dir), "config.toml");
}

#[test]
fn a_repeated_key_in_a_yaml_fragment_is_refused_naming_the_path() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/profile.toml",
        &with_harness("\n[harnesses.goose]\nversions = \">=1.0.0\"\n"),
    );
    s.write("p/goose/config.yaml", "a: 1\na: 2\n");
    names(&refusal(&dir), "a");
}

#[test]
fn a_repeated_key_in_the_manifest_is_refused() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write("p/profile.toml", &format!("{MANIFEST}\nname = \"again\"\n"));
    names(&refusal(&dir), "profile.toml");
}

// YAML: one document, no anchors, aliases or tags, string keys only.

fn yaml_profile(s: &Scratch, yaml: &str) -> PathBuf {
    let dir = s.profile("p");
    s.write(
        "p/profile.toml",
        &with_harness("\n[harnesses.goose]\nversions = \">=1.0.0\"\n"),
    );
    s.write("p/goose/config.yaml", yaml);
    dir
}

#[test]
fn a_yaml_alias_is_refused() {
    let s = Scratch::new();
    names(
        &refusal(&yaml_profile(&s, "a: &x 1\nb: *x\n")),
        "config.yaml",
    );
}

#[test]
fn a_yaml_anchor_is_refused() {
    let s = Scratch::new();
    names(&refusal(&yaml_profile(&s, "a: &x 1\n")), "config.yaml");
}

#[test]
fn a_yaml_tag_is_refused() {
    let s = Scratch::new();
    names(&refusal(&yaml_profile(&s, "a: !!str 1\n")), "config.yaml");
}

#[test]
fn two_yaml_documents_are_refused() {
    let s = Scratch::new();
    names(
        &refusal(&yaml_profile(&s, "a: 1\n---\nb: 2\n")),
        "config.yaml",
    );
}

#[test]
fn a_yaml_key_that_is_not_a_string_is_refused() {
    let s = Scratch::new();
    names(&refusal(&yaml_profile(&s, "1: one\n")), "config.yaml");
}

// Nesting: 64 levels read, 65 refused, in every format.

fn nested_json(levels: usize) -> String {
    let mut text = String::from("1");
    for _ in 0..levels {
        text = format!("{{\"k\": {text}}}");
    }
    text
}

fn nested_yaml(levels: usize) -> String {
    let mut text = String::new();
    for depth in 0..levels {
        text.push_str(&format!("{}k:\n", "  ".repeat(depth)));
    }
    text.push_str(&format!("{}k: 1\n", "  ".repeat(levels)));
    text
}

fn nested_toml(levels: usize) -> String {
    let header = vec!["k"; levels].join(".");
    format!("[{header}]\nleaf = 1\n")
}

#[test]
fn json_nested_sixty_four_levels_loads_and_sixty_five_is_refused() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write("p/claude-code/settings.json", &nested_json(64));
    must_load(&dir);
    s.write("p/claude-code/settings.json", &nested_json(65));
    names(&refusal(&dir), "settings.json");
}

#[test]
fn toml_nested_sixty_four_levels_loads_and_sixty_five_is_refused() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/profile.toml",
        &with_harness("\n[harnesses.codex]\nversions = \">=0.160.0\"\n"),
    );
    s.write("p/codex/config.toml", &nested_toml(63));
    must_load(&dir);
    s.write("p/codex/config.toml", &nested_toml(64));
    names(&refusal(&dir), "config.toml");
}

#[test]
fn yaml_nested_sixty_four_levels_loads_and_sixty_five_is_refused() {
    let s = Scratch::new();
    let dir = yaml_profile(&s, &nested_yaml(63));
    must_load(&dir);
    s.write("p/goose/config.yaml", &nested_yaml(64));
    names(&refusal(&dir), "config.yaml");
}

// Which files are opened.

#[test]
fn a_file_the_table_does_not_name_is_listed_as_not_read_and_never_opened() {
    let s = Scratch::new();
    let dir = s.profile("p");
    let secret = s.write("p/claude-code/auth.json", r#"{"token": "never-read"}"#);
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o000)).expect("chmod");
    let loaded = load_dir(&dir);
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o600)).expect("chmod back");
    let profiles =
        loaded.unwrap_or_else(|e| panic!("an unread file broke the load: {}", e.status()));
    assert_eq!(profiles[0].not_read(), ["claude-code/auth.json"]);
    assert!(
        profiles[0].credential_flags().is_empty(),
        "an unread file is not scanned for credential keys"
    );
}

#[test]
fn a_covered_harness_without_its_file_is_refused_naming_the_file() {
    let s = Scratch::new();
    let dir = s.profile("p");
    fs::remove_file(dir.join("claude-code/settings.json")).expect("rm");
    names(&refusal(&dir), "settings.json");
}

#[test]
fn a_directory_for_a_harness_the_manifest_does_not_cover_is_listed_as_not_read() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write("p/codex/config.toml", "model = \"x\"\n");
    let profiles = must_load(&dir);
    assert_eq!(profiles[0].not_read(), ["codex/config.toml"]);
}

#[test]
fn a_covered_harness_the_caller_has_no_rule_for_is_refused_naming_the_id() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/profile.toml",
        &with_harness("\n[harnesses.unknown-agent]\nversions = \">=1.0.0\"\n"),
    );
    names(&refusal(&dir), "unknown-agent");
}

#[test]
fn a_rule_whose_file_name_leaves_the_directory_is_refused() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write("outside.json", "{}");
    let bad = LoadRules::new(vec![HarnessRule::new(
        "claude-code",
        "../../outside.json",
        FragmentFormat::Json,
    )]);
    must_load(&dir);
    let err = Profile::load(&Source::Directory(dir), &bad).expect_err("a leaving name");
    names(&err, "outside.json");
}

// Symlinks, including the named root.

#[test]
fn a_symlink_inside_the_profile_makes_it_invalid() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write("elsewhere.txt", "x");
    symlink(s.path().join("elsewhere.txt"), dir.join("notes.txt")).expect("symlink");
    names(&refusal(&dir), "notes.txt");
}

#[test]
fn a_symlinked_fragment_makes_the_profile_invalid() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write("real.json", CLAUDE);
    fs::remove_file(dir.join("claude-code/settings.json")).expect("rm");
    symlink(
        s.path().join("real.json"),
        dir.join("claude-code/settings.json"),
    )
    .expect("symlink");
    names(&refusal(&dir), "settings.json");
}

#[test]
fn a_symlinked_profile_directory_is_refused() {
    let s = Scratch::new();
    let real = s.profile("real");
    let link = s.path().join("link");
    symlink(&real, &link).expect("symlink");
    must_load(&real);
    let err = load_dir(&link).expect_err("a symlinked root");
    names(&err, "link");
}

#[test]
fn a_symlinked_profile_toml_is_refused() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write("real.toml", MANIFEST);
    fs::remove_file(dir.join("profile.toml")).expect("rm");
    symlink(s.path().join("real.toml"), dir.join("profile.toml")).expect("symlink");
    names(&refusal(&dir), "profile.toml");
}

// Caps: 1 MiB a file, 64 files, 4 MiB in total, depth 8; nothing partly read.

#[test]
fn a_file_over_one_mebibyte_makes_the_profile_invalid() {
    let s = Scratch::new();
    let dir = s.profile("p");
    let pad = "x".repeat(1024 * 1024 + 1);
    s.write(
        "p/claude-code/settings.json",
        &format!("{{\"pad\": \"{pad}\"}}"),
    );
    names(&refusal(&dir), "settings.json");
}

#[test]
fn a_file_of_exactly_one_mebibyte_is_read() {
    let s = Scratch::new();
    let dir = s.profile("p");
    let head = "{\"pad\": \"";
    let tail = "\"}";
    let pad = "x".repeat(1024 * 1024 - head.len() - tail.len());
    s.write("p/claude-code/settings.json", &format!("{head}{pad}{tail}"));
    assert_eq!(must_load(&dir).len(), 1);
}

#[test]
fn sixty_five_files_make_the_profile_invalid_and_sixty_four_do_not() {
    let s = Scratch::new();
    let dir = s.profile("p");
    // profile.toml, settings.json and 62 more: 64 files.
    for n in 0..62 {
        s.write(&format!("p/docs/f{n}.txt"), "x");
    }
    assert_eq!(must_load(&dir).len(), 1);
    s.write("p/docs/f62.txt", "x");
    names(&refusal(&dir), "64");
}

#[test]
fn the_total_of_bytes_read_is_capped_at_four_mebibytes() {
    let s = Scratch::new();
    let dir = s.profile("p");
    let mut manifest = String::from(MANIFEST);
    for id in ["codex", "goose"] {
        manifest.push_str(&format!("\n[harnesses.{id}]\nversions = \">=1.0.0\"\n"));
    }
    s.write("p/profile.toml", &manifest);
    // Three fragments just under 1 MiB each load; files that are not read never count.
    let each = 1024 * 1024 - 64;
    s.write(
        "p/claude-code/settings.json",
        &format!("{{\"pad\": \"{}\"}}", "x".repeat(each)),
    );
    s.write(
        "p/codex/config.toml",
        &format!("pad = \"{}\"\n", "x".repeat(each)),
    );
    s.write(
        "p/goose/config.yaml",
        &format!("pad: \"{}\"\n", "x".repeat(each)),
    );
    s.write("p/docs/readme.txt", &"y".repeat(1024 * 1024));
    s.write("p/docs/more.txt", &"y".repeat(1024 * 1024));
    assert_eq!(
        must_load(&dir).len(),
        1,
        "about 3 MiB read and 2 MiB not read is under the cap"
    );
    // A fourth and fifth readable fragment would pass 4 MiB.
    let mut more = manifest.clone();
    for id in ["claude-desktop", "extra"] {
        more.push_str(&format!("\n[harnesses.{id}]\nversions = \">=1.0.0\"\n"));
    }
    s.write("p/profile.toml", &more);
    let wide = |name: &str| HarnessRule::new(name, "settings.json", FragmentFormat::Json);
    for id in ["claude-desktop", "extra"] {
        s.write(
            &format!("p/{id}/settings.json"),
            &format!("{{\"pad\": \"{}\"}}", "x".repeat(each)),
        );
    }
    let mut table = vec![
        HarnessRule::new("claude-code", "settings.json", FragmentFormat::Json),
        HarnessRule::new("codex", "config.toml", FragmentFormat::Toml),
        HarnessRule::new("goose", "config.yaml", FragmentFormat::Yaml),
    ];
    table.push(wide("claude-desktop"));
    table.push(wide("extra"));
    let err = Profile::load(&Source::Directory(dir), &LoadRules::new(table))
        .expect_err("five near-1-MiB fragments pass the 4 MiB total");
    names(&err, "4");
}

#[test]
fn a_directory_nested_deeper_than_eight_levels_makes_the_profile_invalid() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write("p/a/b/c/d/e/f/g/h/ok.txt", "x");
    assert_eq!(must_load(&dir).len(), 1, "depth eight is read");
    s.write("p/a/b/c/d/e/f/g/h/i/deep.txt", "x");
    names(&refusal(&dir), "deep.txt");
}

// A collection: profiles/<name>/.

fn collection(s: &Scratch, count: usize) -> PathBuf {
    for n in 0..count {
        let name = format!("team-{n}");
        s.write(
            &format!("repo/profiles/{name}/profile.toml"),
            &MANIFEST.replace("team-setup", &name),
        );
        s.write(
            &format!("repo/profiles/{name}/claude-code/settings.json"),
            CLAUDE,
        );
    }
    s.path().join("repo")
}

#[test]
fn a_directory_holding_profiles_loads_each_profile_and_returns_them_all() {
    let s = Scratch::new();
    let repo = collection(&s, 3);
    let profiles = must_load(&repo);
    let mut found: Vec<&str> = profiles.iter().map(|p| p.manifest().name()).collect();
    found.sort_unstable();
    assert_eq!(found, ["team-0", "team-1", "team-2"]);
}

#[test]
fn a_manifest_name_that_differs_from_its_profiles_directory_is_refused_naming_it() {
    let s = Scratch::new();
    let repo = collection(&s, 2);
    must_load(&repo);
    s.write(
        "repo/profiles/team-1/profile.toml",
        &MANIFEST.replace("team-setup", "other"),
    );
    names(&refusal(&repo), "team-1");
}

#[test]
fn one_invalid_profile_fails_the_whole_source_naming_that_profile() {
    let s = Scratch::new();
    let repo = collection(&s, 3);
    must_load(&repo);
    s.write("repo/profiles/team-2/claude-code/settings.json", "{bad");
    names(&refusal(&repo), "team-2");
}

#[test]
fn a_collection_holds_at_most_sixty_four_profiles() {
    let s = Scratch::new();
    let repo = collection(&s, 64);
    assert_eq!(must_load(&repo).len(), 64);
    let s2 = Scratch::new();
    let repo2 = collection(&s2, 65);
    names(&refusal(&repo2), "64");
}

#[test]
fn a_directory_that_holds_a_manifest_is_one_profile_and_is_not_checked_for_its_name() {
    let s = Scratch::new();
    let dir = s.profile("whatever-the-directory-is-called");
    let profiles = must_load(&dir);
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].manifest().name(), "team-setup");
}

// MCP: a fragment entry replaces the shared entry as a whole.

fn mcp_profile(s: &Scratch, shared: &str, fragment: &str) -> PathBuf {
    let dir = s.profile("p");
    s.write("p/profile.toml", &with_harness(shared));
    s.write("p/claude-code/settings.json", fragment);
    dir
}

const SHARED_GITHUB: &str =
    "\n[mcp_servers.github]\ncommand = \"github-mcp\"\nargs = [\"--stdio\"]\n";

#[test]
fn the_shared_entry_applies_when_the_fragment_has_none() {
    let s = Scratch::new();
    let dir = mcp_profile(&s, SHARED_GITHUB, "{}");
    let profiles = must_load(&dir);
    let entry = profiles[0]
        .mcp_entry("claude-code", "github")
        .expect("entry");
    assert_eq!(entry.origin(), McpOrigin::Shared);
    assert_eq!(entry.field_str("command"), Some("github-mcp"));
}

#[test]
fn a_fragment_entry_replaces_the_shared_entry_as_a_whole_with_no_field_merge() {
    let s = Scratch::new();
    let dir = mcp_profile(
        &s,
        SHARED_GITHUB,
        r#"{"mcpServers": {"github": {"command": "other-mcp"}}}"#,
    );
    let profiles = must_load(&dir);
    let entry = profiles[0]
        .mcp_entry("claude-code", "github")
        .expect("entry");
    assert_eq!(entry.origin(), McpOrigin::Fragment);
    assert_eq!(entry.field_str("command"), Some("other-mcp"));
    assert_eq!(
        entry.field_str("args"),
        None,
        "the shared field `args` is not merged in"
    );
}

#[test]
fn a_server_only_in_a_fragment_stays_in_that_fragment() {
    let s = Scratch::new();
    let dir = mcp_profile(
        &s,
        SHARED_GITHUB,
        r#"{"mcpServers": {"local": {"command": "l"}}}"#,
    );
    let profiles = must_load(&dir);
    assert!(profiles[0].mcp_entry("claude-code", "local").is_some());
    s.write(
        "p/profile.toml",
        &with_harness(&format!(
            "{SHARED_GITHUB}\n[harnesses.codex]\nversions = \">=0.160.0\"\n"
        )),
    );
    s.write("p/codex/config.toml", "model = \"x\"\n");
    let profiles = must_load(&dir);
    assert!(
        profiles[0].mcp_entry("codex", "local").is_none(),
        "a claude-code server is not a codex server"
    );
}

#[test]
fn a_harness_with_no_mcp_path_has_no_override() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/profile.toml",
        &with_harness(&format!(
            "{SHARED_GITHUB}\n[harnesses.goose]\nversions = \">=1.0.0\"\n"
        )),
    );
    s.write(
        "p/goose/config.yaml",
        "mcp_servers:\n  github:\n    command: nope\n",
    );
    let profiles = must_load(&dir);
    let entry = profiles[0].mcp_entry("goose", "github").expect("shared");
    assert_eq!(entry.origin(), McpOrigin::Shared);
}

#[test]
fn an_mcp_path_that_is_not_an_object_is_refused_naming_the_path() {
    let s = Scratch::new();
    let dir = mcp_profile(&s, SHARED_GITHUB, r#"{"mcpServers": ["github"]}"#);
    names(&refusal(&dir), "mcpServers");
}

// Notes name an entry that exists.

#[test]
fn a_note_naming_a_leaf_or_an_object_of_the_fragment_is_accepted() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/profile.toml",
        &with_harness(
            "\n[notes]\n\"claude-code:permissions.allow\" = \"leaf\"\n\"claude-code:permissions\" = \"object\"\n",
        ),
    );
    assert_eq!(must_load(&dir)[0].manifest().notes().len(), 2);
}

#[test]
fn a_note_naming_no_entry_of_the_fragment_is_refused_naming_the_note() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/profile.toml",
        &with_harness("\n[notes]\n\"claude-code:permissions.nothing\" = \"x\"\n"),
    );
    names(&refusal(&dir), "claude-code:permissions.nothing");
}

#[test]
fn an_mcp_note_names_a_shared_entry_or_a_fragment_server_and_otherwise_is_refused() {
    let s = Scratch::new();
    let dir = mcp_profile(
        &s,
        &format!(
            "{SHARED_GITHUB}\n[notes]\n\"mcp:github\" = \"shared\"\n\"mcp:local\" = \"fragment\"\n"
        ),
        r#"{"mcpServers": {"local": {"command": "l"}}}"#,
    );
    assert_eq!(must_load(&dir)[0].manifest().notes().len(), 2);
    s.write(
        "p/profile.toml",
        &with_harness(&format!(
            "{SHARED_GITHUB}\n[notes]\n\"mcp:missing\" = \"x\"\n"
        )),
    );
    names(&refusal(&dir), "mcp:missing");
}

// Credential keys are flagged, never stripped, and the value is never copied.

#[test]
fn a_leaf_is_flagged_when_any_segment_of_its_path_matches_a_listed_name_in_any_case() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/claude-code/settings.json",
        r#"{"env": {"API_KEY": "sk-live-1"}, "credentials": {"user": "u"}, "model": "m"}"#,
    );
    let profiles = must_load(&dir);
    let mut flagged: Vec<(&str, &str)> = profiles[0]
        .credential_flags()
        .iter()
        .map(|f| (f.harness(), f.key_path()))
        .collect();
    flagged.sort_unstable();
    assert_eq!(
        flagged,
        [
            ("claude-code", "credentials.user"),
            ("claude-code", "env.API_KEY")
        ]
    );
}

#[test]
fn a_flagged_key_stays_in_the_fragment_and_its_value_is_in_no_flag_or_error() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write(
        "p/claude-code/settings.json",
        r#"{"token": "sk-live-secret"}"#,
    );
    let profiles = must_load(&dir);
    assert_eq!(
        leaves(&profiles[0], "claude-code"),
        ["token"],
        "not stripped"
    );
    for flag in profiles[0].credential_flags() {
        assert!(!format!("{flag:?}").contains("sk-live-secret"));
    }
    assert_eq!(profiles[0].credential_flags().len(), 1);
}

// Errors carry four fields and name the profile path.

#[test]
fn a_refusal_has_four_fields_and_names_the_profile_path() {
    let s = Scratch::new();
    let dir = s.profile("p");
    s.write("p/claude-code/settings.json", "{bad");
    let err = refusal(&dir);
    for field in [err.step(), err.check(), err.status(), err.next_action()] {
        assert!(!field.is_empty(), "a field of the refusal is empty");
    }
    names(&err, dir.to_str().expect("utf-8 path"));
}
