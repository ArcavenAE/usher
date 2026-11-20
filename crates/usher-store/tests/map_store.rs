//! Story 1.6, PR B: the store moves to version 3. A run carries the verb that made it
//! (`inventory` or `map`), and a map run records one row per setting reading in an
//! insert-only table. The migration is one transaction under the write lock, from version
//! 2 and straight from version 1. Public API, and a second connection where a test must
//! look at the file itself.

use std::fs;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use usher_store::{RunRecord, SeatRecord, SettingRecord, SettingRow, SourceRecord, Store};
use usher_testkit::ScratchHome;

const DB: &str = "usher.db";

/// The version 2 schema as the seat inventory shipped it: version 1 plus the seat table.
const V2: &str = "
CREATE TABLE inventory_run (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recorded_at TEXT NOT NULL,
    harness TEXT NOT NULL,
    harness_version TEXT,
    knowledge_schema INTEGER NOT NULL,
    knowledge_sequence INTEGER NOT NULL,
    environment_key TEXT NOT NULL
);
CREATE TABLE inventory_source (
    run_id INTEGER NOT NULL REFERENCES inventory_run(id),
    item_id TEXT NOT NULL,
    path TEXT NOT NULL,
    layer TEXT NOT NULL,
    present INTEGER NOT NULL CHECK (present IN (0, 1)),
    size INTEGER,
    modified TEXT,
    key_names TEXT NOT NULL
);
CREATE TRIGGER inventory_run_no_update BEFORE UPDATE ON inventory_run
BEGIN SELECT RAISE(ABORT, 'inventory rows are insert-only'); END;
CREATE TRIGGER inventory_run_no_delete BEFORE DELETE ON inventory_run
BEGIN SELECT RAISE(ABORT, 'inventory rows are insert-only'); END;
CREATE TRIGGER inventory_source_no_update BEFORE UPDATE ON inventory_source
BEGIN SELECT RAISE(ABORT, 'inventory rows are insert-only'); END;
CREATE TRIGGER inventory_source_no_delete BEFORE DELETE ON inventory_source
BEGIN SELECT RAISE(ABORT, 'inventory rows are insert-only'); END;
CREATE TABLE inventory_seat (
    run_id INTEGER NOT NULL REFERENCES inventory_run(id),
    part TEXT NOT NULL,
    item_id TEXT NOT NULL,
    present INTEGER CHECK (present IS NULL OR present IN (0, 1)),
    names TEXT NOT NULL,
    members TEXT NOT NULL,
    note TEXT
);
CREATE TRIGGER inventory_seat_no_update BEFORE UPDATE ON inventory_seat
BEGIN SELECT RAISE(ABORT, 'inventory rows are insert-only'); END;
CREATE TRIGGER inventory_seat_no_delete BEFORE DELETE ON inventory_seat
BEGIN SELECT RAISE(ABORT, 'inventory rows are insert-only'); END;
INSERT INTO inventory_run VALUES (1, '2026-11-15T10:00:00Z', 'claude-code', '2.1.317', 4, 4, 'env-key-1');
INSERT INTO inventory_source VALUES (1, 'claude-code.source-user-settings', '/h/.claude/settings.json', 'user', 1, 120, '2026-11-15T09:00:00Z', '[\"model\",\"theme\"]');
INSERT INTO inventory_seat VALUES (1, 'mcp', 'claude-code.seat-mcp-launch-dir', 1, '[[\"mcpServers\",\"s1\"]]', '[]', NULL);
PRAGMA user_version = 2;
";

fn state_dir(scratch: &ScratchHome) -> PathBuf {
    scratch.home().join(".local/state/usher")
}

fn version_two(scratch: &ScratchHome, extra: &str) -> PathBuf {
    let dir = state_dir(scratch);
    fs::create_dir_all(&dir).expect("dirs");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).expect("mode");
    let conn = Connection::open(dir.join(DB)).expect("a database");
    conn.execute_batch(V2).expect("the version 2 schema");
    if !extra.is_empty() {
        conn.execute_batch(extra).expect("extra setup");
    }
    drop(conn);
    fs::set_permissions(dir.join(DB), fs::Permissions::from_mode(0o600)).expect("mode");
    dir
}

fn raw(dir: &Path) -> Connection {
    Connection::open(dir.join(DB)).expect("a second connection to the file")
}

fn user_version(dir: &Path) -> i64 {
    raw(dir)
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .expect("user_version")
}

fn names(dir: &Path, kind: &str) -> Vec<String> {
    let conn = raw(dir);
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = ?1 ORDER BY name")
        .expect("prepare");
    stmt.query_map([kind], |r| r.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows")
}

fn columns(dir: &Path, table: &str) -> Vec<String> {
    let conn = raw(dir);
    let mut stmt = conn
        .prepare("SELECT name FROM pragma_table_info(?1) ORDER BY cid")
        .expect("prepare");
    stmt.query_map([table], |r| r.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows")
}

fn run() -> RunRecord<'static> {
    RunRecord {
        recorded_at: "2026-11-20T12:00:00Z",
        harness: "claude-code",
        harness_version: Some("2.1.317"),
        knowledge_schema: 5,
        knowledge_sequence: 5,
        environment_key: "env-key-1",
    }
}

fn setting<'a>(key: &'a str, layer: &'a str, status: &'a str) -> SettingRecord<'a> {
    SettingRecord {
        harness: "claude-code",
        launch_dir: "/work/repo",
        key,
        layer,
        source_path: "/work/repo/.claude/settings.json",
        method: "rebuilt",
        status,
        entry_index: None,
        value_text: None,
        note: None,
    }
}

fn open_fresh() -> (ScratchHome, Store) {
    let scratch = ScratchHome::new().expect("scratch");
    let store = Store::open(&state_dir(&scratch)).expect("a store");
    (scratch, store)
}

#[test]
fn a_version_two_store_opens_as_version_three_with_its_rows_intact_and_its_runs_read_inventory() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = version_two(&scratch, "");
    let store = Store::open(&dir).expect("a version 2 store is migrated, not refused");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].environment_key, "env-key-1");
    assert_eq!(runs[0].verb, "inventory");
    let sources = store.sources_of(runs[0].id).expect("sources");
    assert_eq!(sources[0].key_names, ["model", "theme"]);
    let seat = store.seat_of(runs[0].id).expect("seat");
    assert_eq!(seat.len(), 1);
    assert_eq!(seat[0].item_id, "claude-code.seat-mcp-launch-dir");
    assert!(
        store.settings_of(runs[0].id).expect("settings").is_empty(),
        "an old run has no readings"
    );
    drop(store);
    assert_eq!(user_version(&dir), 3);
    assert!(names(&dir, "table").contains(&"map_setting".to_string()));
    assert!(columns(&dir, "inventory_run").contains(&"verb".to_string()));
}

#[test]
fn a_fresh_store_is_made_at_version_three_with_the_map_table_and_the_verb_column() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = state_dir(&scratch);
    drop(Store::open(&dir).expect("a fresh store"));
    assert_eq!(user_version(&dir), 3);
    assert!(names(&dir, "table").contains(&"map_setting".to_string()));
    assert!(columns(&dir, "inventory_run").contains(&"verb".to_string()));
    let triggers = names(&dir, "trigger");
    for name in ["map_setting_no_update", "map_setting_no_delete"] {
        assert!(triggers.contains(&name.to_string()), "{triggers:?}");
    }
}

#[test]
fn the_map_table_has_the_columns_a_reading_needs_and_no_content_column() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = state_dir(&scratch);
    drop(Store::open(&dir).expect("a fresh store"));
    assert_eq!(
        columns(&dir, "map_setting"),
        [
            "run_id",
            "harness",
            "launch_dir",
            "key",
            "layer",
            "source_path",
            "method",
            "status",
            "entry_index",
            "value_text",
            "note"
        ]
    );
}

#[test]
fn the_map_table_is_insert_only_and_a_run_row_still_is() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = version_two(&scratch, "");
    drop(Store::open(&dir).expect("migrated"));
    let conn = raw(&dir);
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("pragma");
    conn.execute(
        "INSERT INTO map_setting (run_id, harness, launch_dir, key, layer, source_path, method, status) VALUES (1, 'claude-code', '/d', 'model', 'user', '/p', 'rebuilt', 'in_effect')",
        [],
    )
    .expect("an insert is allowed");
    for statement in [
        "UPDATE map_setting SET key = 'x'",
        "DELETE FROM map_setting",
    ] {
        let err = conn.execute(statement, []).expect_err(statement);
        assert!(
            err.to_string().contains("insert-only"),
            "{statement}: {err}"
        );
    }
    let err = conn
        .execute("UPDATE inventory_run SET verb = 'map'", [])
        .expect_err("a run row stays insert-only");
    assert!(err.to_string().contains("insert-only"), "{err}");
}

#[test]
fn the_verb_is_inventory_or_map_and_nothing_else() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = state_dir(&scratch);
    drop(Store::open(&dir).expect("a fresh store"));
    let err = raw(&dir)
        .execute(
            "INSERT INTO inventory_run (recorded_at, harness, knowledge_schema, knowledge_sequence, environment_key, verb) VALUES ('2026-11-20T12:00:00Z', 'h', 5, 5, 'k', 'explain')",
            [],
        )
        .expect_err("an unknown verb is refused by the table");
    assert!(err.to_string().to_lowercase().contains("check"), "{err}");
}

#[test]
fn a_migration_that_fails_midway_leaves_the_store_at_version_two_unchanged() {
    let scratch = ScratchHome::new().expect("scratch");
    // A trigger of the map trigger's name already exists on another table, so the map
    // table and the verb column are made and a later step fails. Only a transaction
    // undoes them.
    let dir = version_two(
        &scratch,
        "CREATE TRIGGER map_setting_no_delete BEFORE DELETE ON inventory_run BEGIN SELECT 1; END;",
    );
    let err = Store::open(&dir).expect_err("the migration cannot complete");
    assert!(!err.step().is_empty() && !err.check().is_empty());
    assert!(!err.status().is_empty() && !err.next_action().is_empty());
    assert_eq!(user_version(&dir), 2, "the version did not move");
    assert!(!names(&dir, "table").contains(&"map_setting".to_string()));
    assert!(
        !columns(&dir, "inventory_run").contains(&"verb".to_string()),
        "the verb column added before the failing step was rolled back"
    );
    let rows: i64 = raw(&dir)
        .query_row("SELECT count(*) FROM inventory_run", [], |r| r.get(0))
        .expect("count");
    assert_eq!(rows, 1, "the rows are untouched");
}

/// A control: a store newer than this build is refused today and stays refused at version 3.
#[test]
fn a_store_of_a_newer_version_is_refused_and_left_alone_control() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = version_two(&scratch, "PRAGMA user_version = 4;");
    let err = Store::open(&dir).expect_err("version 4 is newer than this usher reads");
    assert!(err.to_string().contains("newer"), "{err}");
    assert_eq!(user_version(&dir), 4);
}

#[test]
fn opening_a_migrated_store_again_changes_nothing() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = version_two(&scratch, "");
    drop(Store::open(&dir).expect("migrated"));
    let before = (
        names(&dir, "table"),
        names(&dir, "trigger"),
        columns(&dir, "inventory_run"),
    );
    drop(Store::open(&dir).expect("opened again"));
    assert_eq!(
        before,
        (
            names(&dir, "table"),
            names(&dir, "trigger"),
            columns(&dir, "inventory_run")
        )
    );
    assert_eq!(user_version(&dir), 3);
}

#[test]
fn a_map_run_stores_its_readings_in_order_with_every_field() {
    let (_scratch, mut store) = open_fresh();
    let allow_user = SettingRecord {
        entry_index: Some(0),
        value_text: Some("Bash(npm test)"),
        status: "in_effect",
        layer: "user",
        source_path: "/home/a/.claude/settings.json",
        ..setting("permissions.allow", "user", "in_effect")
    };
    let allow_project = SettingRecord {
        entry_index: Some(1),
        value_text: Some("Bash(npm test)"),
        note: Some("also set in user"),
        layer: "project",
        ..setting("permissions.allow", "project", "in_effect")
    };
    let mode = SettingRecord {
        note: Some("own merge rule"),
        ..setting("permissions.defaultMode", "user", "not_decided")
    };
    let id = store
        .record_map(&run(), &[], &[], &[allow_user, allow_project, mode])
        .expect("a map run");
    let rows = store.settings_of(id).expect("readings");
    assert_eq!(
        rows,
        vec![
            SettingRow {
                harness: "claude-code".to_string(),
                launch_dir: "/work/repo".to_string(),
                key: "permissions.allow".to_string(),
                layer: "user".to_string(),
                source_path: "/home/a/.claude/settings.json".to_string(),
                method: "rebuilt".to_string(),
                status: "in_effect".to_string(),
                entry_index: Some(0),
                value_text: Some("Bash(npm test)".to_string()),
                note: None,
            },
            SettingRow {
                harness: "claude-code".to_string(),
                launch_dir: "/work/repo".to_string(),
                key: "permissions.allow".to_string(),
                layer: "project".to_string(),
                source_path: "/work/repo/.claude/settings.json".to_string(),
                method: "rebuilt".to_string(),
                status: "in_effect".to_string(),
                entry_index: Some(1),
                value_text: Some("Bash(npm test)".to_string()),
                note: Some("also set in user".to_string()),
            },
            SettingRow {
                harness: "claude-code".to_string(),
                launch_dir: "/work/repo".to_string(),
                key: "permissions.defaultMode".to_string(),
                layer: "user".to_string(),
                source_path: "/work/repo/.claude/settings.json".to_string(),
                method: "rebuilt".to_string(),
                status: "not_decided".to_string(),
                entry_index: None,
                value_text: None,
                note: Some("own merge rule".to_string()),
            },
        ]
    );
}

#[test]
fn a_map_run_reads_as_map_and_an_inventory_run_as_inventory() {
    let (_scratch, mut store) = open_fresh();
    let inventory = store.record_inventory(&run(), &[]).expect("an inventory");
    let seat: [SeatRecord<'_>; 0] = [];
    let with_seat = store
        .record_inventory_with_seat(&run(), &[], &seat)
        .expect("an inventory with a seat");
    let map = store
        .record_map(&run(), &[], &[], &[setting("model", "user", "in_effect")])
        .expect("a map");
    let verbs: Vec<(i64, String)> = store
        .runs()
        .expect("runs")
        .into_iter()
        .map(|r| (r.id, r.verb))
        .collect();
    assert_eq!(
        verbs,
        [
            (inventory, "inventory".to_string()),
            (with_seat, "inventory".to_string()),
            (map, "map".to_string())
        ]
    );
}

#[test]
fn a_map_run_keeps_its_sources_and_seat_lines_beside_its_readings() {
    let (_scratch, mut store) = open_fresh();
    let sources = [SourceRecord {
        item_id: "claude-code.source-user-settings",
        path: "/home/a/.claude/settings.json",
        layer: "user",
        present: true,
        size: Some(10),
        modified: Some("2026-11-20T11:00:00Z"),
        key_names: &["model"],
    }];
    let seat = [SeatRecord {
        part: "mcp",
        item_id: "claude-code.seat-mcp-launch-dir",
        present: Some(false),
        names: &[],
        members: &[],
        note: None,
    }];
    let id = store
        .record_map(
            &run(),
            &sources,
            &seat,
            &[setting("model", "user", "in_effect")],
        )
        .expect("a map run");
    assert_eq!(store.sources_of(id).expect("sources").len(), 1);
    assert_eq!(store.seat_of(id).expect("seat").len(), 1);
    assert_eq!(store.settings_of(id).expect("readings").len(), 1);
}

#[test]
fn readings_are_kept_per_run_and_an_inventory_run_has_none() {
    let (_scratch, mut store) = open_fresh();
    let first = store
        .record_map(&run(), &[], &[], &[setting("model", "user", "in_effect")])
        .expect("first map");
    let second = store
        .record_map(
            &run(),
            &[],
            &[],
            &[
                setting("model", "project", "shadowed"),
                setting("env", "user", "in_effect"),
            ],
        )
        .expect("second map");
    let inventory = store.record_inventory(&run(), &[]).expect("an inventory");
    assert_eq!(store.settings_of(first).expect("first").len(), 1);
    assert_eq!(store.settings_of(second).expect("second").len(), 2);
    assert!(store.settings_of(inventory).expect("inventory").is_empty());
}

#[test]
fn a_method_other_than_rebuilt_or_harness_reported_is_refused_and_nothing_is_stored() {
    let (_scratch, mut store) = open_fresh();
    let bad = SettingRecord {
        method: "guessed",
        ..setting("model", "user", "in_effect")
    };
    let err = store
        .record_map(
            &run(),
            &[],
            &[],
            &[setting("env", "user", "in_effect"), bad],
        )
        .expect_err("an unknown method is refused");
    assert!(
        err.to_string().contains("method") && err.to_string().contains("guessed"),
        "{err}"
    );
    assert!(!err.step().is_empty() && !err.next_action().is_empty());
    assert!(
        store.runs().expect("runs").is_empty(),
        "the run was not inserted"
    );
}

#[test]
fn both_methods_the_map_names_are_accepted() {
    let (_scratch, mut store) = open_fresh();
    let reported = SettingRecord {
        method: "harness-reported",
        ..setting("model", "user", "in_effect")
    };
    let id = store
        .record_map(
            &run(),
            &[],
            &[],
            &[reported, setting("env", "user", "in_effect")],
        )
        .expect("both methods");
    let methods: Vec<String> = store
        .settings_of(id)
        .expect("readings")
        .into_iter()
        .map(|r| r.method)
        .collect();
    assert_eq!(methods, ["harness-reported", "rebuilt"]);
}

#[test]
fn an_empty_status_or_key_is_refused_and_nothing_is_stored() {
    for bad in [
        SettingRecord {
            status: "",
            ..setting("model", "user", "in_effect")
        },
        SettingRecord {
            key: "",
            ..setting("model", "user", "in_effect")
        },
        SettingRecord {
            layer: "",
            ..setting("model", "user", "in_effect")
        },
    ] {
        let (_scratch, mut store) = open_fresh();
        let err = store
            .record_map(&run(), &[], &[], &[bad])
            .expect_err("an empty field is refused");
        assert!(!err.step().is_empty() && !err.next_action().is_empty());
        assert!(store.runs().expect("runs").is_empty());
    }
}

/// A control: the run time check exists already, so this passes against the stub and fails
/// if the new insert path skips it.
#[test]
fn a_bad_run_time_inserts_no_readings_control() {
    let (_scratch, mut store) = open_fresh();
    let bad_run = RunRecord {
        recorded_at: "yesterday",
        ..run()
    };
    store
        .record_map(&bad_run, &[], &[], &[setting("model", "user", "in_effect")])
        .expect_err("a bad time is refused");
    assert!(store.runs().expect("runs").is_empty());
}

#[test]
fn a_value_and_a_note_round_trip_exactly_including_quotes_and_unicode() {
    let (_scratch, mut store) = open_fresh();
    let text = "a \"quoted\" value with 'ticks', a \\ slash and caf\u{e9}";
    let r = SettingRecord {
        value_text: Some(text),
        note: Some(text),
        ..setting("permissions.allow", "user", "in_effect")
    };
    let id = store.record_map(&run(), &[], &[], &[r]).expect("a map run");
    let rows = store.settings_of(id).expect("readings");
    assert_eq!(rows[0].value_text.as_deref(), Some(text));
    assert_eq!(rows[0].note.as_deref(), Some(text));
}

#[test]
fn a_null_value_stays_null_and_is_never_an_empty_string() {
    let (_scratch, mut store) = open_fresh();
    let id = store
        .record_map(&run(), &[], &[], &[setting("env", "user", "in_effect")])
        .expect("a map run");
    let rows = store.settings_of(id).expect("readings");
    assert_eq!(rows[0].value_text, None);
    assert_eq!(rows[0].entry_index, None);
    assert_eq!(rows[0].note, None);
}

const TOKENS: [&str; 8] = [
    "in_effect",
    "shadowed",
    "unread",
    "not_decided",
    "overridden_by_environment",
    "value_not_read",
    "not_checked",
    "unknown",
];

/// A fresh store and a version 2 store migrated to version 3, to be held to one schema.
fn fresh_and_migrated() -> (ScratchHome, PathBuf, ScratchHome, PathBuf) {
    let fresh = ScratchHome::new().expect("scratch");
    let fresh_dir = state_dir(&fresh);
    drop(Store::open(&fresh_dir).expect("a fresh store"));
    let old = ScratchHome::new().expect("scratch");
    let old_dir = version_two(&old, "");
    drop(Store::open(&old_dir).expect("migrated"));
    (fresh, fresh_dir, old, old_dir)
}

fn raw_insert(dir: &Path, status: &str, value: &str) -> rusqlite::Result<usize> {
    let conn = raw(dir);
    conn.execute_batch("PRAGMA foreign_keys = OFF;")
        .expect("pragma");
    conn.execute(
        &format!(
            "INSERT INTO map_setting (run_id, harness, launch_dir, key, layer, source_path, method, status, value_text) VALUES (1, 'claude-code', '/d', 'model', 'user', '/p', 'rebuilt', '{status}', {value})"
        ),
        [],
    )
}

#[test]
fn each_of_the_eight_status_tokens_round_trips() {
    let (_scratch, mut store) = open_fresh();
    let readings: Vec<SettingRecord<'_>> = TOKENS
        .iter()
        .map(|token| setting("model", "user", token))
        .collect();
    let id = store
        .record_map(&run(), &[], &[], &readings)
        .expect("every token is accepted");
    let stored: Vec<String> = store
        .settings_of(id)
        .expect("readings")
        .into_iter()
        .map(|r| r.status)
        .collect();
    assert_eq!(stored, TOKENS);
}

#[test]
fn a_status_outside_the_eight_is_refused_by_the_table_on_a_fresh_and_a_migrated_store() {
    let (_f, fresh, _o, old) = fresh_and_migrated();
    for dir in [&fresh, &old] {
        for bad in ["in effect", "In_Effect", "", "unreadable", "not decided"] {
            let err = raw_insert(dir, bad, "NULL").expect_err(bad);
            assert!(
                err.to_string().to_lowercase().contains("constraint"),
                "`{bad}`: {err}"
            );
        }
        for token in TOKENS.iter().filter(|t| **t != "value_not_read") {
            raw_insert(dir, token, "NULL").unwrap_or_else(|e| panic!("`{token}`: {e}"));
        }
    }
}

#[test]
fn a_status_outside_the_eight_is_refused_through_the_api_naming_it_and_nothing_is_stored() {
    let (_scratch, mut store) = open_fresh();
    let bad = setting("model", "user", "in effect");
    let err = store
        .record_map(
            &run(),
            &[],
            &[],
            &[setting("env", "user", "in_effect"), bad],
        )
        .expect_err("a status outside the set is refused before the table sees it");
    let shown = err.to_string();
    assert!(
        shown.contains("status") && shown.contains("in effect"),
        "{shown}"
    );
    assert!(!err.step().is_empty() && !err.next_action().is_empty());
    assert!(
        store.runs().expect("runs").is_empty(),
        "the run was not inserted"
    );
}

#[test]
fn a_value_not_read_row_with_a_value_is_refused_through_the_api_and_by_the_table() {
    let (_scratch, mut store) = open_fresh();
    let leaking = SettingRecord {
        value_text: Some("a-value"),
        ..setting("env", "user", "value_not_read")
    };
    let err = store
        .record_map(&run(), &[], &[], &[leaking])
        .expect_err("a row that says it did not read the value cannot hold one");
    assert!(err.to_string().contains("value_not_read"), "{err}");
    assert!(store.runs().expect("runs").is_empty());
    let (_f, fresh, _o, old) = fresh_and_migrated();
    for dir in [&fresh, &old] {
        let err = raw_insert(dir, "value_not_read", "'a-value'").expect_err("a value");
        assert!(
            err.to_string().to_lowercase().contains("constraint"),
            "{err}"
        );
        raw_insert(dir, "value_not_read", "NULL").expect("a NULL value is the row's own shape");
    }
}

#[test]
fn the_verb_check_holds_on_a_migrated_store_as_on_a_fresh_one() {
    let (_f, fresh, _o, old) = fresh_and_migrated();
    for dir in [&fresh, &old] {
        let err = raw(dir)
            .execute(
                "INSERT INTO inventory_run (recorded_at, harness, knowledge_schema, knowledge_sequence, environment_key, verb) VALUES ('2026-11-20T12:00:00Z', 'h', 5, 5, 'k', 'explain')",
                [],
            )
            .expect_err("an unknown verb is refused");
        assert!(
            err.to_string().to_lowercase().contains("constraint"),
            "{err}"
        );
        let id: i64 = raw(dir)
            .query_row(
                "INSERT INTO inventory_run (recorded_at, harness, knowledge_schema, knowledge_sequence, environment_key) VALUES ('2026-11-20T12:00:00Z', 'h', 5, 5, 'k') RETURNING id",
                [],
                |r| r.get(0),
            )
            .expect("a run with no verb");
        let verb: String = raw(dir)
            .query_row("SELECT verb FROM inventory_run WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .expect("verb");
        assert_eq!(verb, "inventory", "the default is the same on both");
    }
}

#[test]
fn a_migrated_store_has_the_schema_of_a_fresh_one_for_the_map_table_and_its_triggers() {
    let (_f, fresh, _o, old) = fresh_and_migrated();
    let sql = |dir: &Path| -> Vec<(String, String)> {
        let conn = raw(dir);
        let mut stmt = conn
            .prepare(
                "SELECT name, sql FROM sqlite_master WHERE name LIKE 'map_setting%' ORDER BY name",
            )
            .expect("prepare");
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("rows")
    };
    let squash = |rows: Vec<(String, String)>| -> Vec<(String, String)> {
        rows.into_iter()
            .map(|(n, s)| (n, s.split_whitespace().collect::<Vec<_>>().join(" ")))
            .collect()
    };
    let a = squash(sql(&fresh));
    assert!(a.len() >= 3, "the table and two triggers: {a:?}");
    assert_eq!(a, squash(sql(&old)));
    let verb = |dir: &Path| -> (String, i64, Option<String>) {
        raw(dir)
            .query_row(
                "SELECT type, \"notnull\", dflt_value FROM pragma_table_xinfo('inventory_run') WHERE name = 'verb'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .expect("a verb column")
    };
    assert_eq!(verb(&fresh), verb(&old));
    assert_eq!(verb(&old).1, 1, "NOT NULL");
}

#[test]
fn the_store_holds_derived_state_only_once_the_map_table_joins_the_allowlist() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = state_dir(&scratch);
    let mut store = Store::open(&dir).expect("a store");
    let readable = SettingRecord {
        value_text: Some("acceptEdits"),
        ..setting("permissions.defaultMode", "user", "in_effect")
    };
    let hidden = SettingRecord {
        note: Some("ANTHROPIC_MODEL"),
        ..setting("env", "user", "value_not_read")
    };
    store
        .record_map(&run(), &[], &[], &[readable, hidden])
        .expect("a map run");
    drop(store);
    let conn = raw(&dir);
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .expect("prepare")
        .query_map([], |r| r.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows");
    assert_eq!(
        tables,
        [
            "inventory_run",
            "inventory_seat",
            "inventory_source",
            "map_setting"
        ],
        "no table beyond the allowlist"
    );
    assert_eq!(
        columns(&dir, "inventory_run"),
        [
            "id",
            "recorded_at",
            "harness",
            "harness_version",
            "knowledge_schema",
            "knowledge_sequence",
            "environment_key",
            "verb"
        ]
    );
    // Every text cell of every table is a field a record produced, and the one value the
    // store holds is the readable one. The secret never handed over cannot appear.
    let allowed = [
        "2026-11-20T12:00:00Z",
        "claude-code",
        "2.1.317",
        "env-key-1",
        "map",
        "/work/repo",
        "permissions.defaultMode",
        "env",
        "user",
        "/work/repo/.claude/settings.json",
        "rebuilt",
        "in_effect",
        "value_not_read",
        "acceptEdits",
        "ANTHROPIC_MODEL",
    ];
    for table in &tables {
        let mut stmt = conn
            .prepare(&format!("SELECT * FROM {table}"))
            .expect("prepare");
        let width = stmt.column_count();
        let mut rows = stmt.query([]).expect("query");
        while let Some(row) = rows.next().expect("row") {
            for i in 0..width {
                let cell: rusqlite::types::Value = row.get(i).expect("cell");
                match cell {
                    rusqlite::types::Value::Null | rusqlite::types::Value::Integer(_) => {}
                    rusqlite::types::Value::Text(text) => assert!(
                        allowed.contains(&text.as_str()),
                        "{table} holds `{text}`, which no recorded field produced"
                    ),
                    other => panic!("{table} holds a {other:?} cell"),
                }
            }
        }
    }
    let value_cells: i64 = conn
        .query_row(
            "SELECT count(*) FROM map_setting WHERE value_text IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert_eq!(value_cells, 1, "only the readable key's value is held");
}

#[test]
fn four_opens_at_once_of_a_fresh_store_leave_the_four_tables_and_version_three() {
    for round in 0..10 {
        let scratch = ScratchHome::new().expect("scratch");
        let dir = state_dir(&scratch);
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&dir)
            .expect("an owner-only directory");
        let barrier = std::sync::Barrier::new(4);
        let results: Vec<Result<Store, usher_core::UsherError>> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        Store::open(&dir)
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("thread"))
                .collect()
        });
        assert!(
            results.iter().all(Result::is_ok),
            "round {round}: an open failed"
        );
        drop(results);
        let tables: i64 = raw(&dir)
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND (name LIKE 'inventory_%' OR name = 'map_setting')",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(tables, 4, "round {round}: one schema, four tables");
        assert_eq!(user_version(&dir), 3, "round {round}");
    }
}

#[test]
fn four_opens_at_once_of_a_version_two_store_migrate_it_once() {
    for round in 0..10 {
        let scratch = ScratchHome::new().expect("scratch");
        let dir = version_two(&scratch, "");
        let barrier = std::sync::Barrier::new(4);
        let results: Vec<Result<Store, usher_core::UsherError>> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        Store::open(&dir)
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("thread"))
                .collect()
        });
        assert!(
            results.iter().all(Result::is_ok),
            "round {round}: an open failed"
        );
        drop(results);
        assert_eq!(user_version(&dir), 3, "round {round}");
        let verbs = columns(&dir, "inventory_run")
            .iter()
            .filter(|c| *c == "verb")
            .count();
        assert_eq!(verbs, 1, "round {round}: the column was added once");
    }
}

/// The version 1 schema as the first inventory shipped it: runs and sources only.
const V1: &str = "
CREATE TABLE inventory_run (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recorded_at TEXT NOT NULL,
    harness TEXT NOT NULL,
    harness_version TEXT,
    knowledge_schema INTEGER NOT NULL,
    knowledge_sequence INTEGER NOT NULL,
    environment_key TEXT NOT NULL
);
CREATE TABLE inventory_source (
    run_id INTEGER NOT NULL REFERENCES inventory_run(id),
    item_id TEXT NOT NULL,
    path TEXT NOT NULL,
    layer TEXT NOT NULL,
    present INTEGER NOT NULL CHECK (present IN (0, 1)),
    size INTEGER,
    modified TEXT,
    key_names TEXT NOT NULL
);
CREATE TRIGGER inventory_run_no_update BEFORE UPDATE ON inventory_run
BEGIN SELECT RAISE(ABORT, 'inventory rows are insert-only'); END;
CREATE TRIGGER inventory_run_no_delete BEFORE DELETE ON inventory_run
BEGIN SELECT RAISE(ABORT, 'inventory rows are insert-only'); END;
CREATE TRIGGER inventory_source_no_update BEFORE UPDATE ON inventory_source
BEGIN SELECT RAISE(ABORT, 'inventory rows are insert-only'); END;
CREATE TRIGGER inventory_source_no_delete BEFORE DELETE ON inventory_source
BEGIN SELECT RAISE(ABORT, 'inventory rows are insert-only'); END;
INSERT INTO inventory_run VALUES (1, '2026-11-15T10:00:00Z', 'claude-code', '2.1.317', 3, 3, 'env-key-1');
INSERT INTO inventory_source VALUES (1, 'claude-code.source-user-settings', '/h/.claude/settings.json', 'user', 1, 120, '2026-11-15T09:00:00Z', '[\"model\",\"theme\"]');
PRAGMA user_version = 1;
";

fn version_one(scratch: &ScratchHome, extra: &str) -> PathBuf {
    let dir = state_dir(scratch);
    fs::create_dir_all(&dir).expect("dirs");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).expect("mode");
    let conn = Connection::open(dir.join(DB)).expect("a database");
    conn.execute_batch(V1).expect("the version 1 schema");
    if !extra.is_empty() {
        conn.execute_batch(extra).expect("extra setup");
    }
    drop(conn);
    fs::set_permissions(dir.join(DB), fs::Permissions::from_mode(0o600)).expect("mode");
    dir
}

#[test]
fn a_version_one_store_opens_at_version_three_with_its_rows_intact() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = version_one(&scratch, "");
    let store = Store::open(&dir).expect("a version 1 store is migrated straight to 3");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].environment_key, "env-key-1");
    assert_eq!(runs[0].verb, "inventory");
    let sources = store.sources_of(runs[0].id).expect("sources");
    assert_eq!(sources[0].key_names, ["model", "theme"]);
    assert!(store.seat_of(runs[0].id).expect("seat").is_empty());
    assert!(store.settings_of(runs[0].id).expect("readings").is_empty());
    drop(store);
    assert_eq!(user_version(&dir), 3);
    let tables = names(&dir, "table");
    assert!(tables.contains(&"inventory_seat".to_string()), "{tables:?}");
    assert!(tables.contains(&"map_setting".to_string()), "{tables:?}");
}

#[test]
fn a_failure_in_the_version_three_step_leaves_a_version_one_store_at_version_one() {
    let scratch = ScratchHome::new().expect("scratch");
    // The map trigger name on the run table: the seat step succeeds and the map step
    // fails, so only one transaction over both steps leaves the store at version 1.
    let dir = version_one(
        &scratch,
        "CREATE TRIGGER map_setting_no_delete BEFORE DELETE ON inventory_run BEGIN SELECT 1; END;",
    );
    let err = Store::open(&dir).expect_err("the migration cannot complete");
    assert!(!err.step().is_empty() && !err.check().is_empty());
    assert!(!err.status().is_empty() && !err.next_action().is_empty());
    assert_eq!(user_version(&dir), 1, "the version did not move");
    let tables = names(&dir, "table");
    assert!(
        !tables.contains(&"inventory_seat".to_string())
            && !tables.contains(&"map_setting".to_string()),
        "no table of a later version was left behind: {tables:?}"
    );
    assert!(!columns(&dir, "inventory_run").contains(&"verb".to_string()));
    let rows: i64 = raw(&dir)
        .query_row("SELECT count(*) FROM inventory_run", [], |r| r.get(0))
        .expect("count");
    assert_eq!(rows, 1, "the rows are untouched");
}

#[test]
fn a_version_one_store_has_every_later_trigger_after_it_migrates() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = version_one(&scratch, "");
    drop(Store::open(&dir).expect("migrated"));
    let triggers = names(&dir, "trigger");
    for name in [
        "inventory_seat_no_update",
        "inventory_seat_no_delete",
        "map_setting_no_update",
        "map_setting_no_delete",
    ] {
        assert!(
            triggers.contains(&name.to_string()),
            "{name} in {triggers:?}"
        );
    }
}

fn raw_insert_fields(
    dir: &Path,
    method: &str,
    key: &str,
    layer: &str,
    entry_index: &str,
) -> rusqlite::Result<usize> {
    let conn = raw(dir);
    conn.execute_batch("PRAGMA foreign_keys = OFF;")
        .expect("pragma");
    conn.execute(
        &format!(
            "INSERT INTO map_setting (run_id, harness, launch_dir, key, layer, source_path, method, status, entry_index) VALUES (1, 'claude-code', '/d', '{key}', '{layer}', '/p', '{method}', 'in_effect', {entry_index})"
        ),
        [],
    )
}

/// The method, key, layer and entry index checks hold on the table itself, past the API, on a
/// fresh store and a migrated one. Each refusal names a constraint, and a good row is accepted
/// first so the refusals are not from a broken insert.
#[test]
fn the_method_key_layer_and_entry_index_checks_refuse_a_raw_insert_on_both_stores() {
    let (_f, fresh, _o, old) = fresh_and_migrated();
    for dir in [&fresh, &old] {
        raw_insert_fields(dir, "rebuilt", "model", "user", "NULL").expect("a good row");
        raw_insert_fields(dir, "harness-reported", "model", "user", "0").expect("index zero");
        let bad = [
            ("method", ("reported", "model", "user", "NULL")),
            ("empty key", ("rebuilt", "", "user", "NULL")),
            ("empty layer", ("rebuilt", "model", "", "NULL")),
            ("negative entry index", ("rebuilt", "model", "user", "-1")),
        ];
        for (what, (method, key, layer, index)) in bad {
            let err = raw_insert_fields(dir, method, key, layer, index).expect_err(what);
            assert!(
                err.to_string().to_lowercase().contains("constraint"),
                "{what}: {err}"
            );
        }
    }
}

/// The method and status lists as the table stores them. SQLite cannot alter a stored check,
/// so a change to either list is a table rebuild at a new version, and this test fails until
/// the change is made on purpose.
#[test]
fn the_stored_method_and_status_checks_are_these_exact_lists() {
    let (_f, fresh, _o, old) = fresh_and_migrated();
    for dir in [&fresh, &old] {
        let sql: String = raw(dir)
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'map_setting'",
                [],
                |r| r.get(0),
            )
            .expect("the table definition");
        for golden in [
            "CHECK (method IN ('rebuilt', 'harness-reported'))",
            "CHECK (status IN ('in_effect', 'shadowed', 'unread', 'not_decided', 'overridden_by_environment', 'value_not_read', 'not_checked', 'unknown'))",
        ] {
            assert!(sql.contains(golden), "{golden} is not in {sql}");
        }
    }
}
