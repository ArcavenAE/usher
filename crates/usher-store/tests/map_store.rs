//! Story 1.6, PR B: the store moves to version 3. A run carries the verb that made it
//! (`inventory` or `map`), and a map run records one row per setting reading in an
//! insert-only table. The migration is one transaction under the write lock, from version
//! 2 and straight from version 1. Public API, and a second connection where a test must
//! look at the file itself.

use std::fs;
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
