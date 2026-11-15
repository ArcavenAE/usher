//! Story 1.5, PR B: the store (AD-7). Runs against the public API, and opens the
//! database file with a second connection where a test must look at the file itself.

use std::fs;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use usher_store::{RunRecord, SourceRecord, Store};
use usher_testkit::ScratchHome;

const DB: &str = "usher.db";

fn state_dir(scratch: &ScratchHome) -> PathBuf {
    scratch.home().join("Library/Application Support/usher")
}

fn open(scratch: &ScratchHome) -> (Store, PathBuf) {
    let dir = state_dir(scratch);
    let store = Store::open(&dir).expect("the store opens in a fresh state directory");
    (store, dir)
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path).expect("metadata").permissions().mode() & 0o777
}

fn run(at: &str) -> RunRecord<'_> {
    RunRecord {
        recorded_at: at,
        harness: "claude-code",
        harness_version: Some("2.1.317"),
        knowledge_schema: 2,
        knowledge_sequence: 2,
        environment_key: "env-key-1",
    }
}

fn source<'a>(item_id: &'a str, path: &'a str, names: &'a [&'a str]) -> SourceRecord<'a> {
    SourceRecord {
        item_id,
        path,
        layer: "user",
        present: true,
        size: Some(120),
        modified: Some("2026-11-15T10:00:00Z"),
        key_names: names,
    }
}

fn raw(dir: &Path) -> Connection {
    let conn = Connection::open(dir.join(DB)).expect("a second connection to the file");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("pragma");
    conn
}

fn refused(result: Result<Store, usher_core::UsherError>, needles: &[&str]) {
    let err = result.expect_err("the open must be refused");
    let shown = err.to_string();
    for needle in needles {
        assert!(shown.contains(needle), "`{needle}` not in `{shown}`");
    }
    assert!(!err.step().is_empty() && !err.check().is_empty());
    assert!(!err.status().is_empty() && !err.next_action().is_empty());
}

#[test]
fn a_first_open_creates_the_state_directory_at_0700_and_the_database_at_0600() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (_store, dir) = open(&scratch);
    assert_eq!(mode(&dir), 0o700);
    assert_eq!(mode(&dir.join(DB)), 0o600);
}

#[test]
fn the_store_runs_wal_with_synchronous_full_a_busy_timeout_and_foreign_keys() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (store, dir) = open(&scratch);
    let settings = store.settings().expect("settings");
    assert_eq!(settings.journal_mode, "wal");
    assert_eq!(settings.synchronous, 2, "synchronous=FULL is 2");
    assert!(settings.busy_timeout_ms > 0, "a busy timeout is set");
    assert_eq!(settings.foreign_keys, 1);
    let file_mode: String = raw(&dir)
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .expect("journal_mode from the file");
    assert_eq!(file_mode, "wal", "the file itself is in WAL mode");
}

#[test]
fn a_new_store_is_at_user_version_one() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (_store, dir) = open(&scratch);
    let version: i64 = raw(&dir)
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .expect("user_version");
    assert_eq!(version, 1);
}

#[test]
fn the_wal_and_shm_files_come_out_owner_only() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (mut store, dir) = open(&scratch);
    store
        .record_inventory(&run("2026-11-15T10:00:00Z"), &[])
        .expect("a write that makes the WAL");
    let mut seen = 0;
    for name in ["usher.db-wal", "usher.db-shm"] {
        let path = dir.join(name);
        if path.exists() {
            seen += 1;
            assert_eq!(mode(&path) & 0o077, 0, "{name} has group or other bits");
        }
    }
    assert!(seen > 0, "a WAL write leaves a -wal or -shm file to check");
}

#[test]
fn an_existing_state_directory_with_group_or_other_bits_is_refused_and_left_alone() {
    let scratch = ScratchHome::new().expect("scratch home");
    let dir = state_dir(&scratch);
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o755)
        .create(&dir)
        .expect("a lax directory");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).expect("lax mode");
    refused(Store::open(&dir), &["group or other"]);
    assert_eq!(
        mode(&dir),
        0o755,
        "usher never changes a directory that exists"
    );
    assert!(!dir.join(DB).exists(), "nothing was created in it");
}

#[test]
fn a_symlinked_state_directory_is_refused() {
    let scratch = ScratchHome::new().expect("scratch home");
    let real = scratch.home().join("real-state");
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&real)
        .expect("real dir");
    let link = scratch.home().join("state-link");
    symlink(&real, &link).expect("symlink");
    refused(Store::open(&link), &["symlink"]);
    assert!(!real.join(DB).exists());
}

#[test]
fn an_existing_owner_only_directory_is_used_and_an_empty_database_file_is_initialized() {
    let scratch = ScratchHome::new().expect("scratch home");
    let dir = state_dir(&scratch);
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&dir)
        .expect("an owner-only directory");
    fs::write(dir.join(DB), b"").expect("an empty file");
    let store = Store::open(&dir).expect("opens in a directory it did not create");
    assert!(store.runs().expect("runs").is_empty());
    assert_eq!(
        fs::metadata(&dir).expect("meta").uid(),
        fs::metadata(dir.join(DB)).expect("meta").uid()
    );
}

#[test]
fn a_non_empty_database_at_version_zero_is_refused() {
    let scratch = ScratchHome::new().expect("scratch home");
    let dir = state_dir(&scratch);
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&dir)
        .expect("dir");
    Connection::open(dir.join(DB))
        .expect("a database")
        .execute_batch("CREATE TABLE somebody_elses (x TEXT);")
        .expect("a table");
    refused(Store::open(&dir), &["not an usher store"]);
}

#[test]
fn a_database_from_a_newer_usher_is_refused() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (store, dir) = open(&scratch);
    drop(store);
    raw(&dir)
        .execute_batch("PRAGMA user_version = 2;")
        .expect("a newer version");
    refused(Store::open(&dir), &["newer"]);
}

#[test]
fn a_second_open_finds_the_first_runs() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (mut store, dir) = open(&scratch);
    store
        .record_inventory(&run("2026-11-15T10:00:00Z"), &[])
        .expect("insert");
    drop(store);
    let again = Store::open(&dir).expect("reopens");
    assert_eq!(again.runs().expect("runs").len(), 1);
}

#[test]
fn a_second_run_inserts_a_new_run_and_rewrites_nothing() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (mut store, _dir) = open(&scratch);
    let names = ["model", "permissions"];
    let sources = [source(
        "claude-code.source-user-settings",
        "/h/.claude/settings.json",
        &names,
    )];
    let first = store
        .record_inventory(&run("2026-11-15T10:00:00Z"), &sources)
        .expect("first");
    let before_runs = store.runs().expect("runs");
    let before_sources = store.sources_of(first).expect("sources");
    let second = store
        .record_inventory(&run("2026-11-15T11:00:00Z"), &sources)
        .expect("second");
    assert!(second > first, "order is the insert sequence");
    let after = store.runs().expect("runs");
    assert_eq!(after.len(), 2);
    assert_eq!(after[0], before_runs[0], "the first run is untouched");
    assert_eq!(store.sources_of(first).expect("sources"), before_sources);
    assert_eq!(store.sources_of(second).expect("sources").len(), 1);
}

#[test]
fn a_run_and_its_sources_read_back_as_recorded() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (mut store, _dir) = open(&scratch);
    let names = ["permissions", "model", "env"];
    let mut absent = source(
        "claude-code.source-local-settings",
        "/r/.claude/settings.local.json",
        &[],
    );
    absent.present = false;
    absent.size = None;
    absent.modified = None;
    let sources = [
        source(
            "claude-code.source-user-settings",
            "/h/.claude/settings.json",
            &names,
        ),
        absent,
    ];
    let mut r = run("2026-11-15T10:00:00Z");
    r.harness_version = None;
    let id = store.record_inventory(&r, &sources).expect("insert");
    let row = &store.runs().expect("runs")[0];
    assert_eq!(row.id, id);
    assert_eq!(row.recorded_at, "2026-11-15T10:00:00Z");
    assert_eq!(row.harness, "claude-code");
    assert_eq!(row.harness_version, None);
    assert_eq!((row.knowledge_schema, row.knowledge_sequence), (2, 2));
    assert_eq!(row.environment_key, "env-key-1");
    let rows = store.sources_of(id).expect("sources");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].key_names, ["permissions", "model", "env"]);
    assert_eq!(rows[0].size, Some(120));
    assert!(rows[0].present);
    assert!(!rows[1].present);
    assert_eq!((rows[1].size, rows[1].modified.clone()), (None, None));
}

#[test]
fn one_item_may_match_several_files_in_one_run() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (mut store, _dir) = open(&scratch);
    let sources = [
        source("claude-code.source-managed-file", "/etc/a.json", &[]),
        source("claude-code.source-managed-file", "/etc/b.json", &[]),
    ];
    let id = store
        .record_inventory(&run("2026-11-15T10:00:00Z"), &sources)
        .expect("a glob can match several files");
    assert_eq!(store.sources_of(id).expect("sources").len(), 2);
}

#[test]
fn an_update_or_a_delete_on_either_table_is_aborted() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (mut store, dir) = open(&scratch);
    store
        .record_inventory(
            &run("2026-11-15T10:00:00Z"),
            &[source(
                "claude-code.source-user-settings",
                "/h/s.json",
                &["model"],
            )],
        )
        .expect("insert");
    let conn = raw(&dir);
    for sql in [
        "UPDATE inventory_run SET harness = 'x'",
        "DELETE FROM inventory_run",
        "UPDATE inventory_source SET path = '/x'",
        "DELETE FROM inventory_source",
    ] {
        let err = conn.execute(sql, []).expect_err(sql);
        assert!(err.to_string().contains("insert-only"), "{sql}: {err}");
    }
    assert_eq!(store.runs().expect("runs").len(), 1);
}

#[test]
fn the_tables_refuse_a_bad_present_flag_a_bad_timestamp_and_an_orphan_source() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (mut store, dir) = open(&scratch);
    let id = store
        .record_inventory(&run("2026-11-15T10:00:00Z"), &[])
        .expect("insert");
    let conn = raw(&dir);
    let bad_present = conn.execute(
        "INSERT INTO inventory_source (run_id, item_id, path, layer, present, key_names) VALUES (?1, 'i', '/p', 'user', 2, '[]')",
        [id],
    );
    assert!(
        bad_present
            .expect_err("present must be 0 or 1")
            .to_string()
            .contains("CHECK")
    );
    let orphan = conn.execute(
        "INSERT INTO inventory_source (run_id, item_id, path, layer, present, key_names) VALUES (999, 'i', '/p', 'user', 1, '[]')",
        [],
    );
    assert!(
        orphan
            .expect_err("run_id must exist")
            .to_string()
            .contains("FOREIGN KEY")
    );
    let bad_time = conn.execute(
        "INSERT INTO inventory_run (recorded_at, harness, knowledge_schema, knowledge_sequence, environment_key) VALUES ('yesterday', 'h', 2, 2, 'k')",
        [],
    );
    assert!(
        bad_time
            .expect_err("recorded_at must be RFC 3339 UTC")
            .to_string()
            .contains("CHECK")
    );
}

#[test]
fn a_timestamp_that_is_not_rfc_3339_utc_is_refused_naming_the_field() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (mut store, _dir) = open(&scratch);
    store
        .record_inventory(&run("2026-11-15T10:00:00Z"), &[])
        .expect("a UTC timestamp is taken");
    for bad in [
        "2026-11-15 10:00:00",
        "2026-11-15T10:00:00+01:00",
        "yesterday",
        "",
    ] {
        let err = store
            .record_inventory(&run(bad), &[])
            .expect_err("a timestamp that is not UTC RFC 3339");
        assert!(err.to_string().contains("recorded_at"), "{bad}: {err}");
    }
    assert_eq!(
        store.runs().expect("runs").len(),
        1,
        "a refused run leaves nothing behind"
    );
}

#[test]
fn a_run_with_a_bad_source_inserts_nothing() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (mut store, _dir) = open(&scratch);
    let mut bad = source("claude-code.source-user-settings", "/h/s.json", &[]);
    bad.modified = Some("not a time");
    let err = store
        .record_inventory(&run("2026-11-15T10:00:00Z"), &[bad])
        .expect_err("a bad modified time");
    assert!(err.to_string().contains("modified"), "{err}");
    assert!(
        store.runs().expect("runs").is_empty(),
        "the run rolled back with its source"
    );
}

#[test]
fn the_store_holds_derived_state_only() {
    let scratch = ScratchHome::new().expect("scratch home");
    let (mut store, dir) = open(&scratch);
    let names = ["oauthAccount", "model", "env"];
    let sources = [source(
        "claude-code.source-global-state",
        "/h/.claude.json",
        &names,
    )];
    store
        .record_inventory(&run("2026-11-15T10:00:00Z"), &sources)
        .expect("insert");
    let conn = raw(&dir);

    let mut tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .expect("prepare")
        .query_map([], |r| r.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows");
    tables.sort();
    assert_eq!(
        tables,
        ["inventory_run", "inventory_source"],
        "no table beyond the allowlist"
    );

    let columns = |table: &str| -> Vec<String> {
        conn.prepare(&format!(
            "SELECT name FROM pragma_table_info('{table}') ORDER BY cid"
        ))
        .expect("prepare")
        .query_map([], |r| r.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows")
    };
    assert_eq!(
        columns("inventory_run"),
        [
            "id",
            "recorded_at",
            "harness",
            "harness_version",
            "knowledge_schema",
            "knowledge_sequence",
            "environment_key"
        ]
    );
    assert_eq!(
        columns("inventory_source"),
        [
            "run_id",
            "item_id",
            "path",
            "layer",
            "present",
            "size",
            "modified",
            "key_names"
        ]
    );

    let allowed = [
        "2026-11-15T10:00:00Z",
        "claude-code",
        "2.1.317",
        "env-key-1",
        "claude-code.source-global-state",
        "/h/.claude.json",
        "user",
        "2026-11-15T10:00:00Z",
        r#"["oauthAccount","model","env"]"#,
    ];
    for table in &tables {
        let sql = format!("SELECT * FROM {table}");
        let mut stmt = conn.prepare(&sql).expect("prepare");
        let width = stmt.column_count();
        let mut rows = stmt.query([]).expect("query");
        while let Some(row) = rows.next().expect("row") {
            for i in 0..width {
                let cell: rusqlite::types::Value = row.get(i).expect("cell");
                if let rusqlite::types::Value::Text(text) = cell {
                    assert!(
                        allowed.contains(&text.as_str()),
                        "{table} holds `{text}`, which no recorded field produced"
                    );
                }
            }
        }
    }
}
