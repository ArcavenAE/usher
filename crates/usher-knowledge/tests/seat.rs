//! Story 1.5, PR D0: schema 4 and the `seat` class, the parts of the seat around a
//! harness (MCP registrations, packs, fleet-tool client config, the auth route and the
//! toolchain). Each item says which part it is, where to look and how much may be read:
//! presence, one directory listing, key names at listed key paths, or the presence of
//! named environment variables. Public API only.

use usher_knowledge::{Class, KnowledgeFile, Layer, Platform, ReferenceKind, SeatPart, SeatRead};

const FILE: &str = r#"
schema_version = 4
sequence = 11

[[item]]
id = "demo.seat-mcp"
harness = "demo"
layer = "slow"
class = "seat"
versions = ">=1.0.0, <2.0.0"
summary = "MCP server registrations in the global state file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/mcp" }
last_verified = 2026-11-20
seat = { part = "mcp", path = "${HOME}/.demo.json", read = "names", key_paths = ["mcpServers.*", "mcpServers.*.env", "mcpServers.*.headers"] }

[[item]]
id = "demo.seat-packs"
harness = "demo"
layer = "slow"
class = "seat"
versions = ">=1.0.0, <2.0.0"
summary = "The installed packs, one entry per directory."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/packs" }
last_verified = 2026-11-20
seat = { part = "packs", path = "${HOME}/.demo/packs", read = "listing" }

[[item]]
id = "demo.seat-client"
harness = "demo"
layer = "slow"
class = "seat"
versions = ">=1.0.0, <2.0.0"
summary = "The client config a fleet seat carries."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/client" }
last_verified = 2026-11-20
seat = { part = "fleet-client", path = "${HOME}/.demo/client-env.sh", read = "presence" }

[[item]]
id = "demo.seat-auth"
harness = "demo"
layer = "slow"
class = "seat"
versions = ">=1.0.0, <2.0.0"
summary = "Environment variables that select the backend and auth route."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/auth" }
last_verified = 2026-11-20
seat = { part = "auth-route", read = "env", variables = ["DEMO_API_KEY", "DEMO_USE_CLOUD_2"] }

[[item]]
id = "demo.seat-toolchain"
harness = "demo"
layer = "slow"
class = "seat"
versions = ">=1.0.0, <2.0.0"
summary = "The toolchain declaration a repository carries."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/toolchain" }
last_verified = 2026-11-20
seat = { part = "toolchain", path = "${REPO}/mise.toml", read = "presence", platforms = ["macos", "linux"] }
"#;

const MCP: &str = r#"seat = { part = "mcp", path = "${HOME}/.demo.json", read = "names", key_paths = ["mcpServers.*", "mcpServers.*.env", "mcpServers.*.headers"] }"#;
const PACKS: &str = r#"seat = { part = "packs", path = "${HOME}/.demo/packs", read = "listing" }"#;
const AUTH: &str = r#"seat = { part = "auth-route", read = "env", variables = ["DEMO_API_KEY", "DEMO_USE_CLOUD_2"] }"#;
const TOOLCHAIN: &str = r#"seat = { part = "toolchain", path = "${REPO}/mise.toml", read = "presence", platforms = ["macos", "linux"] }"#;

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
fn a_schema_4_file_carries_the_five_seat_parts() {
    let file = KnowledgeFile::parse(FILE).expect("a valid schema 4 file");
    assert_eq!(file.version().schema_version(), 4);
    let seats: Vec<_> = file.items().iter().map(|item| item.seat()).collect();
    assert!(
        seats.iter().all(Option::is_some),
        "every item has a seat fact"
    );
    let mcp = file.items()[0].seat().expect("mcp");
    assert_eq!(file.items()[0].class(), Class::Seat);
    assert_eq!(file.items()[0].layer(), Layer::Slow);
    assert_eq!(mcp.part(), SeatPart::Mcp);
    assert_eq!(mcp.read(), SeatRead::Names);
    assert_eq!(mcp.path(), Some("${HOME}/.demo.json"));
    assert_eq!(
        mcp.key_paths(),
        ["mcpServers.*", "mcpServers.*.env", "mcpServers.*.headers"]
    );
    assert!(mcp.variables().is_empty());
    let packs = file.items()[1].seat().expect("packs");
    assert_eq!(
        (packs.part(), packs.read()),
        (SeatPart::Packs, SeatRead::Listing)
    );
    let client = file.items()[2].seat().expect("fleet client");
    assert_eq!(
        (client.part(), client.read()),
        (SeatPart::FleetClient, SeatRead::Presence)
    );
    let auth = file.items()[3].seat().expect("auth route");
    assert_eq!(
        (auth.part(), auth.read()),
        (SeatPart::AuthRoute, SeatRead::Env)
    );
    assert_eq!(auth.path(), None);
    assert_eq!(auth.variables(), ["DEMO_API_KEY", "DEMO_USE_CLOUD_2"]);
    assert!(auth.key_paths().is_empty());
    let toolchain = file.items()[4].seat().expect("toolchain");
    assert_eq!(toolchain.part(), SeatPart::Toolchain);
    assert_eq!(
        toolchain.platforms(),
        Some(&[Platform::Macos, Platform::Linux][..])
    );
    assert_eq!(file.items()[0].seat().expect("mcp").platforms(), None);
}

#[test]
fn a_schema_3_file_is_refused_naming_both_schemas() {
    rejects(
        &with("schema_version = 4", "schema_version = 3"),
        &["schema 3", "schema 4"],
    );
}

#[test]
fn the_embedded_file_is_schema_4() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    assert_eq!(file.version().schema_version(), 4);
}

/// Whichever auth-route item the seed carries, it cites something. Its exact form waits
/// for the ruling on how the account-key signal is written.
#[test]
fn the_embedded_file_carries_an_auth_route_item_with_a_citation() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    let item = file
        .items()
        .iter()
        .find(|item| {
            item.seat()
                .is_some_and(|seat| seat.part() == SeatPart::AuthRoute)
        })
        .expect("an auth-route seat item");
    assert!(!item.reference().locator().is_empty());
}

#[test]
fn a_seat_item_of_the_fast_layer_is_refused() {
    let text = FILE.replacen(
        "layer = \"slow\"\nclass = \"seat\"",
        "layer = \"fast\"\nclass = \"seat\"",
        1,
    );
    rejects(
        &text,
        &["demo.seat-mcp", "class `seat`", "other layer", "`fast`"],
    );
}

#[test]
fn a_seat_table_on_another_class_is_refused_naming_the_item() {
    let text = with(
        "class = \"seat\"\nversions = \">=1.0.0, <2.0.0\"\nsummary = \"The installed packs",
        "class = \"paths\"\nversions = \">=1.0.0, <2.0.0\"\nsummary = \"The installed packs",
    );
    rejects(&text, &["demo.seat-packs", "seat", "class `seat`"]);
}

#[test]
fn a_seat_item_without_its_seat_table_is_refused() {
    let text = with(PACKS, "");
    rejects(&text, &["demo.seat-packs", "seat"]);
}

#[test]
fn part_and_read_must_be_named_and_known() {
    rejects(
        &with(r#"part = "packs""#, r#"part = "plugins""#),
        &["demo.seat-packs", "part", "plugins"],
    );
    rejects(
        &with(r#"part = "packs", "#, ""),
        &["demo.seat-packs", "part"],
    );
    rejects(
        &with(r#"read = "listing""#, r#"read = "contents""#),
        &["demo.seat-packs", "read", "contents"],
    );
    rejects(
        &with(r#", read = "listing""#, ""),
        &["demo.seat-packs", "read"],
    );
    rejects(
        &with(
            r#"read = "listing" }"#,
            r#"read = "listing", colour = "red" }"#,
        ),
        &["demo.seat-packs", "colour"],
    );
}

#[test]
fn key_paths_belong_to_names_and_names_needs_them() {
    for read in ["presence", "listing"] {
        let text = with(
            PACKS,
            &format!(
                r#"seat = {{ part = "packs", path = "${{HOME}}/.demo/packs", read = "{read}", key_paths = ["a.b"] }}"#
            ),
        );
        rejects(&text, &["demo.seat-packs", "key_paths"]);
    }
    let env_with_paths = with(
        AUTH,
        r#"seat = { part = "auth-route", read = "env", variables = ["DEMO_API_KEY"], key_paths = ["a"] }"#,
    );
    rejects(&env_with_paths, &["demo.seat-auth", "key_paths"]);
}

#[test]
fn names_without_key_paths_is_refused() {
    let none = with(
        MCP,
        r#"seat = { part = "mcp", path = "${HOME}/.demo.json", read = "names" }"#,
    );
    rejects(&none, &["demo.seat-mcp", "key_paths"]);
    let empty = with(
        MCP,
        r#"seat = { part = "mcp", path = "${HOME}/.demo.json", read = "names", key_paths = [] }"#,
    );
    rejects(&empty, &["demo.seat-mcp", "key_paths"]);
}

#[test]
fn variables_belong_to_env_and_env_needs_them() {
    let on_names = with(
        MCP,
        r#"seat = { part = "mcp", path = "${HOME}/.demo.json", read = "names", key_paths = ["mcpServers.*"], variables = ["DEMO_API_KEY"] }"#,
    );
    rejects(&on_names, &["demo.seat-mcp", "variables"]);
    let on_presence = with(
        TOOLCHAIN,
        r#"seat = { part = "toolchain", path = "${REPO}/mise.toml", read = "presence", variables = ["DEMO_API_KEY"] }"#,
    );
    rejects(&on_presence, &["demo.seat-toolchain", "variables"]);
    let none = with(AUTH, r#"seat = { part = "auth-route", read = "env" }"#);
    rejects(&none, &["demo.seat-auth", "variables"]);
    let empty = with(
        AUTH,
        r#"seat = { part = "auth-route", read = "env", variables = [] }"#,
    );
    rejects(&empty, &["demo.seat-auth", "variables"]);
}

#[test]
fn a_path_is_absent_for_env_and_present_for_every_other_read() {
    let with_path = with(
        AUTH,
        r#"seat = { part = "auth-route", path = "${HOME}/.demo", read = "env", variables = ["DEMO_API_KEY"] }"#,
    );
    rejects(&with_path, &["demo.seat-auth", "path"]);
    let without = with(PACKS, r#"seat = { part = "packs", read = "listing" }"#);
    rejects(&without, &["demo.seat-packs", "path"]);
}

#[test]
fn a_variable_name_is_upper_case_letters_digits_and_underscores() {
    for bad in [
        "demo_api_key",
        "1DEMO",
        "DEMO-KEY",
        "DEMO KEY",
        "",
        "_DEMO",
        "DEMO=1",
    ] {
        let text = with(
            AUTH,
            &format!(r#"seat = {{ part = "auth-route", read = "env", variables = ["{bad}"] }}"#),
        );
        rejects(&text, &["demo.seat-auth", "variables"]);
    }
    accepts(&with(
        AUTH,
        r#"seat = { part = "auth-route", read = "env", variables = ["A", "A_1", "ZZ9_"] }"#,
    ));
}

#[test]
fn a_key_path_is_names_joined_by_dots_and_a_star_is_one_whole_level() {
    for bad in [
        "", "a..b", ".a", "a.", "mcp*.env", "a.*x", "**", "a.**.b", "a.?",
    ] {
        let text = with(
            MCP,
            &format!(
                r#"seat = {{ part = "mcp", path = "${{HOME}}/.demo.json", read = "names", key_paths = ["{bad}"] }}"#
            ),
        );
        rejects(&text, &["demo.seat-mcp", "key_paths"]);
    }
    accepts(&with(
        MCP,
        r#"seat = { part = "mcp", path = "${HOME}/.demo.json", read = "names", key_paths = ["a", "a.b", "projects.*.mcpServers.*", "mcpServers.*.env"] }"#,
    ));
}

#[test]
fn no_key_path_reaches_the_account_key() {
    for bad in [
        "oauthAccount",
        "oauthAccount.emailAddress",
        "oauthAccount.*",
        "projects.oauthAccount",
        "projects.*.oauthAccount.x",
        "*",
        "*.env",
    ] {
        let text = with(
            MCP,
            &format!(
                r#"seat = {{ part = "mcp", path = "${{HOME}}/.demo.json", read = "names", key_paths = ["mcpServers.*", "{bad}"] }}"#
            ),
        );
        rejects(&text, &["demo.seat-mcp", "key_paths", bad]);
    }
    accepts(&with(
        MCP,
        r#"seat = { part = "mcp", path = "${HOME}/.demo.json", read = "names", key_paths = ["oauthAccountNote", "myoauthAccount", "mcpServers.*"] }"#,
    ));
}

#[test]
fn a_seat_path_uses_the_path_tokens_and_nothing_else() {
    accepts(&with(
        PACKS,
        r#"seat = { part = "packs", path = "${LAUNCH_DIR}/.demo/packs", read = "listing" }"#,
    ));
    for bad in [
        "${ELSEWHERE}/packs",
        "packs/relative",
        "${HOME}/packs/${NOPE}",
    ] {
        let text = with(
            PACKS,
            &format!(r#"seat = {{ part = "packs", path = "{bad}", read = "listing" }}"#),
        );
        rejects(&text, &["demo.seat-packs", "path"]);
    }
}

#[test]
fn platforms_on_a_seat_item_are_checked_as_on_a_source() {
    rejects(
        &with(
            TOOLCHAIN,
            &TOOLCHAIN.replace(r#"["macos", "linux"]"#, r#"["amiga"]"#),
        ),
        &["demo.seat-toolchain", "platforms", "amiga"],
    );
    rejects(
        &with(TOOLCHAIN, &TOOLCHAIN.replace(r#"["macos", "linux"]"#, "[]")),
        &["demo.seat-toolchain", "platforms"],
    );
    rejects(
        &with(
            TOOLCHAIN,
            &TOOLCHAIN.replace(r#"["macos", "linux"]"#, r#""macos""#),
        ),
        &["demo.seat-toolchain", "platforms"],
    );
}

#[test]
fn a_seat_item_needs_its_reference_and_its_date() {
    let no_reference = with(
        "reference = { kind = \"vendor-docs\", locator = \"https://example.test/docs/packs\" }\n",
        "",
    );
    rejects(&no_reference, &["demo.seat-packs", "reference"]);
    let no_date = FILE.replacen(
        "last_verified = 2026-11-20\nseat = { part = \"packs\"",
        "seat = { part = \"packs\"",
        1,
    );
    rejects(&no_date, &["demo.seat-packs", "last_verified"]);
}

// keys-present: whether a top-level key is a member of a JSON file's top-level object,
// the one place a credential key may be named. It never reads under the key.

const ACCOUNT_ITEM: &str = r#"
[[item]]
id = "demo.seat-account"
harness = "demo"
layer = "slow"
class = "seat"
versions = ">=1.0.0, <2.0.0"
summary = "Whether the global state file carries the account key."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/account" }
last_verified = 2026-11-20
seat = { part = "auth-route", path = "${HOME}/.demo.json", read = "keys-present", keys = ["oauthAccount"] }
"#;

const ACCOUNT: &str = r#"seat = { part = "auth-route", path = "${HOME}/.demo.json", read = "keys-present", keys = ["oauthAccount"] }"#;

fn with_account(seat: &str) -> String {
    format!("{FILE}{}", ACCOUNT_ITEM.replace(ACCOUNT, seat))
}

#[test]
fn keys_present_naming_the_account_key_parses_and_reads_back() {
    let file = KnowledgeFile::parse(&with_account(ACCOUNT)).expect("a valid file");
    let account = file.items()[5].seat().expect("the account seat fact");
    assert_eq!(account.part(), SeatPart::AuthRoute);
    assert_eq!(account.read(), SeatRead::KeysPresent);
    assert_eq!(account.path(), Some("${HOME}/.demo.json"));
    assert_eq!(account.keys(), ["oauthAccount"]);
    assert!(account.key_paths().is_empty());
    assert!(account.variables().is_empty());
    let mcp = file.items()[0].seat().expect("mcp");
    assert!(mcp.keys().is_empty());
}

#[test]
fn a_key_path_into_the_account_key_is_still_refused_beside_a_keys_present_item() {
    for bad in [
        "oauthAccount",
        "oauthAccount.*",
        "oauthAccount.emailAddress",
    ] {
        let seat = format!(
            r#"seat = {{ part = "mcp", path = "${{HOME}}/.demo.json", read = "names", key_paths = ["{bad}"] }}"#
        );
        let text = format!("{}{ACCOUNT_ITEM}", FILE.replace(MCP, &seat));
        rejects(&text, &["demo.seat-mcp", "key_paths", bad]);
    }
}

#[test]
fn keys_are_plain_unique_top_level_names() {
    for bad in [
        r#"["oauthAccount.x"]"#,
        r#"["oauthAccount", "oauthAccount"]"#,
        r#"["a.b"]"#,
        r#"["*"]"#,
        r#"["a*"]"#,
        r#"[""]"#,
        "[]",
    ] {
        let seat = format!(
            r#"seat = {{ part = "auth-route", path = "${{HOME}}/.demo.json", read = "keys-present", keys = {bad} }}"#
        );
        rejects(&with_account(&seat), &["demo.seat-account", "keys"]);
    }
    let missing =
        r#"seat = { part = "auth-route", path = "${HOME}/.demo.json", read = "keys-present" }"#;
    rejects(&with_account(missing), &["demo.seat-account", "keys"]);
    accepts(&with_account(
        r#"seat = { part = "auth-route", path = "${HOME}/.demo.json", read = "keys-present", keys = ["hasCompletedOnboarding", "oauthAccount"] }"#,
    ));
}

#[test]
fn keys_belong_to_keys_present_and_keys_present_takes_nothing_else() {
    let on_names = with(
        MCP,
        r#"seat = { part = "mcp", path = "${HOME}/.demo.json", read = "names", key_paths = ["mcpServers.*"], keys = ["oauthAccount"] }"#,
    );
    rejects(&on_names, &["demo.seat-mcp", "keys"]);
    let on_presence = with(
        TOOLCHAIN,
        r#"seat = { part = "toolchain", path = "${REPO}/mise.toml", read = "presence", keys = ["x"] }"#,
    );
    rejects(&on_presence, &["demo.seat-toolchain", "keys"]);
    for extra in [r#"key_paths = ["a"]"#, r#"variables = ["DEMO_API_KEY"]"#] {
        let seat = format!(
            r#"seat = {{ part = "auth-route", path = "${{HOME}}/.demo.json", read = "keys-present", keys = ["oauthAccount"], {extra} }}"#
        );
        rejects(&with_account(&seat), &["demo.seat-account", "keys-present"]);
    }
    let no_path =
        r#"seat = { part = "auth-route", read = "keys-present", keys = ["oauthAccount"] }"#;
    rejects(&with_account(no_path), &["demo.seat-account", "path"]);
}

#[test]
fn keys_present_reads_only_a_file_another_item_already_names() {
    let unnamed = r#"seat = { part = "auth-route", path = "${HOME}/.other.json", read = "keys-present", keys = ["oauthAccount"] }"#;
    rejects(
        &with_account(unnamed),
        &["demo.seat-account", "${HOME}/.other.json"],
    );
    let source = r#"
[[item]]
id = "demo.source-state"
harness = "demo"
layer = "slow"
class = "sources"
versions = ">=1.0.0, <2.0.0"
summary = "The other state file."
reference = { kind = "vendor-docs", locator = "https://example.test/docs/state" }
last_verified = 2026-11-20
source = { path = "${HOME}/.other.json", layer = "user", read = "read", rank = 1, always_present = false }
"#;
    accepts(&format!("{}{source}", with_account(unnamed)));
}

fn item_at(id: &str, seat: &str) -> String {
    ACCOUNT_ITEM
        .replace("demo.seat-account", id)
        .replace(ACCOUNT, seat)
}

#[test]
fn a_presence_or_listing_item_does_not_name_a_json_file_for_keys_present() {
    for (id, read) in [
        ("demo.seat-only-presence", "presence"),
        ("demo.seat-only-listing", "listing"),
    ] {
        let namer = item_at(
            id,
            &format!(
                r#"seat = {{ part = "packs", path = "${{HOME}}/.only.json", read = "{read}" }}"#
            ),
        );
        let asks = item_at(
            "demo.seat-account",
            r#"seat = { part = "auth-route", path = "${HOME}/.only.json", read = "keys-present", keys = ["oauthAccount"] }"#,
        );
        rejects(
            &format!("{FILE}{namer}{asks}"),
            &["demo.seat-account", "${HOME}/.only.json"],
        );
    }
}

fn anchored(id: &str, harness: &str, versions: &str, seat: &str) -> String {
    ACCOUNT_ITEM
        .replace("demo.seat-account", id)
        .replace("harness = \"demo\"", &format!("harness = \"{harness}\""))
        .replace(">=1.0.0, <2.0.0", versions)
        .replace(ACCOUNT, seat)
}

const ANCHORED_ASK: &str = r#"seat = { part = "auth-route", path = "${HOME}/.x.json", read = "keys-present", keys = ["oauthAccount"] }"#;
const ANCHOR: &str = r#"seat = { part = "mcp", path = "${HOME}/.x.json", read = "names", key_paths = ["mcpServers.*"] }"#;

#[test]
fn a_keys_present_item_does_not_anchor_a_file_for_another() {
    let first = anchored("demo.seat-first", "demo", ">=1.0.0, <2.0.0", ANCHORED_ASK);
    let second = anchored(
        "demo.seat-second",
        "demo",
        ">=1.0.0, <2.0.0",
        &ANCHORED_ASK.replace("oauthAccount", "other"),
    );
    let err =
        KnowledgeFile::parse(&format!("{FILE}{first}{second}")).expect_err("both are refused");
    let shown = err.to_string();
    assert!(shown.contains("no sources or names item names"), "{shown}");
    for id in ["demo.seat-first", "demo.seat-second"] {
        let alone = KnowledgeFile::parse(&format!(
            "{FILE}{}",
            anchored(id, "demo", ">=1.0.0, <2.0.0", ANCHORED_ASK)
        ))
        .expect_err("refused alone too");
        assert!(alone.to_string().contains(id), "{alone}");
    }
}

#[test]
fn a_names_item_of_the_same_harness_over_overlapping_versions_anchors_the_file() {
    let anchor = anchored("demo.seat-anchor", "demo", ">=1.5.0, <3.0.0", ANCHOR);
    let ask = anchored("demo.seat-ask", "demo", ">=1.0.0, <2.0.0", ANCHORED_ASK);
    accepts(&format!("{FILE}{anchor}{ask}"));
}

#[test]
fn an_anchor_of_another_harness_or_of_disjoint_versions_does_not_count() {
    let ask = anchored("demo.seat-ask", "demo", ">=1.0.0, <2.0.0", ANCHORED_ASK);
    let other_harness = anchored("other.seat-anchor", "other", ">=1.0.0, <2.0.0", ANCHOR);
    rejects(
        &format!("{FILE}{other_harness}{ask}"),
        &[
            "demo.seat-ask",
            "${HOME}/.x.json",
            "no sources or names item names",
        ],
    );
    let disjoint = anchored("demo.seat-anchor", "demo", ">=2.0.0, <3.0.0", ANCHOR);
    rejects(
        &format!("{FILE}{disjoint}{ask}"),
        &[
            "demo.seat-ask",
            "${HOME}/.x.json",
            "no sources or names item names",
        ],
    );
}

/// The environment half of the auth route, from the vendor's environment-variable page.
#[test]
fn the_embedded_file_pins_the_claude_code_auth_route_variables() {
    let file = KnowledgeFile::embedded().expect("the embedded file");
    let item = file
        .items()
        .iter()
        .find(|item| item.id() == "claude-code.seat-auth-env")
        .expect("the auth-route environment item");
    assert_eq!(item.harness(), "claude-code");
    assert_eq!(item.class(), Class::Seat);
    assert_eq!(item.layer(), Layer::Slow);
    let seat = item.seat().expect("a seat fact");
    assert_eq!(seat.part(), SeatPart::AuthRoute);
    assert_eq!(seat.read(), SeatRead::Env);
    assert_eq!(seat.path(), None);
    assert_eq!(
        seat.variables(),
        [
            "ANTHROPIC_API_KEY",
            "ANTHROPIC_AUTH_TOKEN",
            "CLAUDE_CODE_OAUTH_TOKEN",
            "CLAUDE_CODE_USE_BEDROCK",
            "CLAUDE_CODE_USE_VERTEX",
            "CLAUDE_CODE_USE_FOUNDRY",
        ]
    );
    assert!(seat.key_paths().is_empty() && seat.keys().is_empty());
    assert_eq!(item.reference().kind(), ReferenceKind::VendorDocs);
    assert_eq!(
        item.reference().locator(),
        "https://code.claude.com/docs/en/env-vars"
    );
}
