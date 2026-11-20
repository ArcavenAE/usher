//! Story 1.5, PR D: the store moves from version 1 to version 2 in one transaction under
//! the write lock, adding the seat table and its insert-only triggers and keeping every
//! row a version 1 store holds. Public API, and a second connection where a test must
//! look at the file itself.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use usher_store::Store;
use usher_testkit::ScratchHome;

const DB: &str = "usher.db";

/// The version 1 schema as PR B shipped it.
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

fn state_dir(scratch: &ScratchHome) -> PathBuf {
    scratch.home().join(".local/state/usher")
}

/// A state directory holding a version 1 store, built with its own connection.
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

#[test]
fn a_version_one_store_opens_as_version_two_with_its_rows_intact() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = version_one(&scratch, "");
    let store = Store::open(&dir).expect("a version 1 store is migrated, not refused");
    let runs = store.runs().expect("runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].environment_key, "env-key-1");
    let sources = store.sources_of(runs[0].id).expect("sources");
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].item_id, "claude-code.source-user-settings");
    assert_eq!(sources[0].key_names, ["model", "theme"]);
    drop(store);
    assert_eq!(user_version(&dir), 2);
    assert!(names(&dir, "table").contains(&"inventory_seat".to_string()));
}

#[test]
fn the_migration_leaves_the_seat_table_insert_only() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = version_one(&scratch, "");
    drop(Store::open(&dir).expect("migrated"));
    let triggers = names(&dir, "trigger");
    for name in ["inventory_seat_no_update", "inventory_seat_no_delete"] {
        assert!(triggers.contains(&name.to_string()), "{triggers:?}");
    }
    let conn = raw(&dir);
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("pragma");
    conn.execute(
        "INSERT INTO inventory_seat (run_id, part, item_id, present, names, members) VALUES (1, 'mcp', 'x', 1, '[]', '[]')",
        [],
    )
    .expect("an insert is allowed");
    for statement in [
        "UPDATE inventory_seat SET present = 0",
        "DELETE FROM inventory_seat",
    ] {
        let err = conn.execute(statement, []).expect_err(statement);
        assert!(
            err.to_string().contains("insert-only"),
            "{statement}: {err}"
        );
    }
    // The version 1 triggers are still there and still abort.
    let err = conn
        .execute("UPDATE inventory_run SET harness = 'x'", [])
        .expect_err("still insert-only");
    assert!(err.to_string().contains("insert-only"), "{err}");
}

#[test]
fn a_migration_that_fails_midway_leaves_the_store_at_version_one_unchanged() {
    let scratch = ScratchHome::new().expect("scratch");
    // A trigger of the seat trigger's name already exists on another table, so creating
    // the seat table succeeds and a later step fails. Only a transaction undoes the table.
    let planted = "inventory_seat_no_delete";
    let dir = version_one(
        &scratch,
        "CREATE TRIGGER inventory_seat_no_delete BEFORE DELETE ON inventory_run BEGIN SELECT 1; END;",
    );
    let err = Store::open(&dir).expect_err("the migration cannot complete");
    assert!(!err.step().is_empty() && !err.check().is_empty());
    assert!(!err.status().is_empty() && !err.next_action().is_empty());
    assert_eq!(user_version(&dir), 1, "the version did not move");
    assert!(
        !names(&dir, "table").contains(&"inventory_seat".to_string()),
        "the seat table created before the failing step was rolled back"
    );
    let triggers = names(&dir, "trigger");
    assert!(
        !triggers
            .iter()
            .any(|name| name.starts_with("inventory_seat") && name != planted),
        "no seat trigger was left behind: {triggers:?}"
    );
    let rows: i64 = raw(&dir)
        .query_row("SELECT count(*) FROM inventory_run", [], |r| r.get(0))
        .expect("count");
    assert_eq!(rows, 1, "the rows are untouched");
}

#[test]
fn a_fresh_store_is_made_at_version_two_with_the_seat_table() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = state_dir(&scratch);
    drop(Store::open(&dir).expect("a fresh store"));
    assert_eq!(user_version(&dir), 2);
    assert!(names(&dir, "table").contains(&"inventory_seat".to_string()));
    let triggers = names(&dir, "trigger");
    assert!(triggers.contains(&"inventory_seat_no_update".to_string()));
}

#[test]
fn a_store_from_a_newer_usher_and_a_non_empty_unversioned_file_are_still_refused() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = version_one(&scratch, "PRAGMA user_version = 3;");
    let err = Store::open(&dir).expect_err("version 3 is newer than this usher reads");
    assert!(err.to_string().contains("newer"), "{err}");
    assert_eq!(user_version(&dir), 3);
    let other = ScratchHome::new().expect("scratch");
    let dir = version_one(&other, "PRAGMA user_version = 0;");
    let err = Store::open(&dir).expect_err("a non-empty file at version 0");
    assert!(err.to_string().contains("not an usher store"), "{err}");
}
