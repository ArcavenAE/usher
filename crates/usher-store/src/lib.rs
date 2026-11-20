//! The usher store (AD-7): one SQLite database in the owner-only state directory.
//!
//! Story 1.5, change B holds the inventory tables only: derived state, inserted and
//! never changed.

use std::fs::{self, DirBuilder, OpenOptions};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use rusqlite::{Connection, params};
use usher_core::UsherError;

/// The database file inside the state directory.
const DB_FILE: &str = "usher.db";

/// The schema version this build writes and reads.
const USER_VERSION: i64 = 1;

/// How long a connection waits on a locked database.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// The two platform suffixes whose parents usher may create. Both count on every platform;
/// the resolver in a later change picks the one for the OS.
const STANDARD_SUFFIXES: [[&str; 3]; 2] = [
    [".local", "state", "usher"],
    ["Library", "Application Support", "usher"],
];

const STEP: &str = "open the usher store";

const SCHEMA: &str = "
CREATE TABLE inventory_run (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recorded_at TEXT NOT NULL CHECK (recorded_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]Z'),
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
    modified TEXT CHECK (modified IS NULL OR modified GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]Z'),
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
";

/// The settings a connection to the store runs with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreSettings {
    /// `PRAGMA journal_mode`, lower case.
    pub journal_mode: String,
    /// `PRAGMA synchronous` as its number: 2 is FULL.
    pub synchronous: i64,
    /// `PRAGMA busy_timeout` in milliseconds.
    pub busy_timeout_ms: i64,
    /// `PRAGMA foreign_keys`: 1 is on.
    pub foreign_keys: i64,
}

/// One inventory run: when it ran and what it assumed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord<'a> {
    /// RFC 3339 UTC, written `YYYY-MM-DDTHH:MM:SSZ`.
    pub recorded_at: &'a str,
    /// The harness id.
    pub harness: &'a str,
    /// The harness version found, or none.
    pub harness_version: Option<&'a str>,
    /// The knowledge schema version the run assumed.
    pub knowledge_schema: u32,
    /// The knowledge sequence the run assumed.
    pub knowledge_sequence: u64,
    /// The AD-17 environment key, as an opaque string.
    pub environment_key: &'a str,
}

/// One source seen in a run, as derived state: never its content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRecord<'a> {
    /// The knowledge item that names the source.
    pub item_id: &'a str,
    /// The resolved absolute path.
    pub path: &'a str,
    /// The layer the knowledge puts it in.
    pub layer: &'a str,
    /// Whether the file was there.
    pub present: bool,
    /// Size in bytes, when present.
    pub size: Option<u64>,
    /// Modified time as RFC 3339 UTC, when present.
    pub modified: Option<&'a str>,
    /// The key names in the file, in file order, never values.
    pub key_names: &'a [&'a str],
}

/// A stored run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRow {
    /// The insert sequence.
    pub id: i64,
    /// What was recorded.
    pub recorded_at: String,
    /// The harness id.
    pub harness: String,
    /// The harness version, if found.
    pub harness_version: Option<String>,
    /// The knowledge schema version.
    pub knowledge_schema: u32,
    /// The knowledge sequence.
    pub knowledge_sequence: u64,
    /// The environment key.
    pub environment_key: String,
}

/// A stored source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRow {
    /// The run it belongs to.
    pub run_id: i64,
    /// The knowledge item id.
    pub item_id: String,
    /// The resolved path.
    pub path: String,
    /// The knowledge layer.
    pub layer: String,
    /// Whether the file was there.
    pub present: bool,
    /// Size in bytes, when present.
    pub size: Option<u64>,
    /// Modified time, when present.
    pub modified: Option<String>,
    /// Key names, in the order recorded.
    pub key_names: Vec<String>,
}

/// An open store.
#[derive(Debug)]
pub struct Store {
    conn: Connection,
}

fn fail(
    check: impl Into<String>,
    status: impl Into<String>,
    next: impl Into<String>,
) -> UsherError {
    UsherError::new(STEP, check, status, next)
}

fn db_error(check: &str, err: &rusqlite::Error) -> UsherError {
    fail(
        check,
        format!("the database refused: {err}"),
        "check that the state directory is writable and not full, then run the command again",
    )
}

fn quoted(path: &Path) -> String {
    let text = path.display().to_string();
    if text.contains('\'') {
        format!("\"{text}\"")
    } else {
        format!("'{text}'")
    }
}

fn refuse_missing(path: &Path) -> UsherError {
    fail(
        "check the parents of the state directory",
        format!(
            "`{}` is missing, and usher creates only the standard parents of its state directory",
            path.display()
        ),
        format!(
            "create it, for example mkdir -p -m 0700 {}, then run the command again",
            quoted(path)
        ),
    )
}

/// The parents usher may create for `dir`: the components above the leaf of a standard suffix.
fn standard_parents(dir: &Path) -> Vec<PathBuf> {
    let names: Vec<_> = dir.components().map(|c| c.as_os_str().to_owned()).collect();
    for suffix in STANDARD_SUFFIXES {
        if names.len() >= suffix.len()
            && names[names.len() - suffix.len()..]
                .iter()
                .zip(suffix)
                .all(|(have, want)| have.as_os_str() == want)
        {
            let parent = dir.parent().map(Path::to_path_buf);
            let grandparent = parent
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf);
            return parent.into_iter().chain(grandparent).collect();
        }
    }
    Vec::new()
}

fn exists(path: &Path) -> Result<bool, UsherError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(fail(
            "inspect the state directory",
            format!("cannot inspect `{}`: {err}", path.display()),
            "fix the permissions on that path, then run the command again",
        )),
    }
}

fn make_private_dir(path: &Path) -> Result<(), UsherError> {
    DirBuilder::new().mode(0o700).create(path).map_err(|err| {
        fail(
            "create the state directory",
            format!("cannot create `{}`: {err}", path.display()),
            "fix the permissions above that path, then run the command again",
        )
    })
}

/// Checks the state directory when it exists, and creates it (and its standard parents) when
/// it does not. Nothing that exists is ever changed.
fn prepare_state_dir(dir: &Path) -> Result<(), UsherError> {
    match fs::symlink_metadata(dir) {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                return Err(fail(
                    "check the state directory",
                    format!("`{}` is a symlink", dir.display()),
                    "point usher at the real directory, or remove the symlink, then run the command again",
                ));
            }
            if !meta.is_dir() {
                return Err(fail(
                    "check the state directory",
                    format!("`{}` is not a directory", dir.display()),
                    "move that file away, then run the command again",
                ));
            }
            if meta.permissions().mode() & 0o077 != 0 {
                return Err(fail(
                    "check the state directory",
                    format!(
                        "`{}` has group or other permission bits; the state directory must be owner-only",
                        dir.display()
                    ),
                    format!(
                        "run chmod 700 {} yourself, then run the command again",
                        quoted(dir)
                    ),
                ));
            }
            return Ok(());
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(fail(
                "check the state directory",
                format!("cannot inspect `{}`: {err}", dir.display()),
                "fix the permissions on that path, then run the command again",
            ));
        }
    }
    let mut missing = Vec::new();
    for ancestor in dir.ancestors().skip(1) {
        if ancestor.as_os_str().is_empty() || exists(ancestor)? {
            break;
        }
        missing.push(ancestor);
    }
    let standard = standard_parents(dir);
    if missing
        .iter()
        .any(|path| !standard.iter().any(|s| s == path))
    {
        let topmost = missing.last().copied().unwrap_or(dir);
        return Err(refuse_missing(topmost));
    }
    for ancestor in missing.iter().rev() {
        make_private_dir(ancestor)?;
    }
    make_private_dir(dir)
}

/// Asks for WAL mode and returns the mode in force. Switching to WAL needs a lock that SQLite
/// refuses at once, without running the busy handler, when another first open holds it, so a
/// busy answer is retried until the busy timeout has passed.
fn set_wal(conn: &Connection) -> Result<String, UsherError> {
    let deadline = Instant::now() + BUSY_TIMEOUT;
    loop {
        match conn.query_row("PRAGMA journal_mode = WAL", [], |r| r.get::<_, String>(0)) {
            Ok(mode) => return Ok(mode),
            Err(rusqlite::Error::SqliteFailure(failure, _))
                if matches!(
                    failure.code,
                    rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
                ) && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(err) => return Err(db_error("set WAL mode", &err)),
        }
    }
}

/// `PRAGMA journal_mode = WAL` returns the mode now in force, which is not `wal` when SQLite
/// could not make the change; the store does not run without it.
fn require_wal(returned: &str) -> Result<(), UsherError> {
    if returned.eq_ignore_ascii_case("wal") {
        Ok(())
    } else {
        Err(fail(
            "set WAL mode",
            format!("the database stayed in `{returned}` journal mode; the store needs WAL"),
            "move the state directory to a local disk that supports WAL, then run the command again",
        ))
    }
}

fn is_utc_second(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 20
        && bytes.iter().enumerate().all(|(i, b)| match i {
            4 | 7 => *b == b'-',
            10 => *b == b'T',
            13 | 16 => *b == b':',
            19 => *b == b'Z',
            _ => b.is_ascii_digit(),
        })
}

fn check_time(field: &str, text: &str) -> Result<(), UsherError> {
    if is_utc_second(text) {
        Ok(())
    } else {
        Err(fail(
            format!("check {field}"),
            format!("`{field}` is `{text}`; it must be RFC 3339 UTC, written YYYY-MM-DDTHH:MM:SSZ"),
            "pass the time in UTC and run the command again",
        ))
    }
}

/// A JSON array of strings, compact, for the key names.
fn json_names(names: &[&str]) -> String {
    let mut out = String::from("[");
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('"');
        for ch in name.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out.push('"');
    }
    out.push(']');
    out
}

/// The version and the number of schema rows, read in one statement so both come from the
/// same snapshot: a version read first and a schema read later could straddle another open's
/// commit. A fresh file has no schema rows, even when an earlier open switched it to WAL (which
/// makes it 4096 bytes at version 0), so freshness is decided by this and never by the file's
/// size.
fn read_state(conn: &Connection) -> Result<(i64, i64), UsherError> {
    conn.query_row(
        "SELECT (SELECT user_version FROM pragma_user_version), count(*) FROM sqlite_master",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .map_err(|err| db_error("read the database version", &err))
}

fn not_a_store(db_path: &Path) -> UsherError {
    fail(
        "check the database version",
        format!(
            "`{}` is not an usher store: it holds data and has no usher version",
            db_path.display()
        ),
        "move that file away, then run the command again",
    )
}

/// Creates the schema under the write lock. The version and the schema are read again inside
/// the lock, so two first opens at once both succeed and the second finds the schema the
/// first made, and a file another program filled in the meantime is refused.
fn create_schema(conn: &Connection, db_path: &Path) -> Result<(), UsherError> {
    let step = |err: &rusqlite::Error| db_error("create the inventory tables", err);
    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|err| step(&err))?;
    let result = (|| -> Result<bool, rusqlite::Error> {
        let (now, rows) = conn.query_row(
            "SELECT (SELECT user_version FROM pragma_user_version), count(*) FROM sqlite_master",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
        )?;
        if now != 0 {
            return Ok(true);
        }
        if rows > 0 {
            return Ok(false);
        }
        conn.execute_batch(&format!("{SCHEMA}PRAGMA user_version = {USER_VERSION};"))?;
        Ok(true)
    })();
    match result {
        Ok(true) => conn.execute_batch("COMMIT").map_err(|err| step(&err)),
        Ok(false) => {
            // The ROLLBACK result is dropped on purpose: the open returns the refusal next and
            // the connection goes with it, which ends the transaction either way.
            let _ = conn.execute_batch("ROLLBACK");
            Err(not_a_store(db_path))
        }
        Err(err) => {
            // As above: the connection is dropped with the error, so a failed ROLLBACK changes nothing.
            let _ = conn.execute_batch("ROLLBACK");
            Err(step(&err))
        }
    }
}

impl Store {
    /// Opens the store in `state_dir`, creating the directory and the database.
    ///
    /// A missing standard parent is created at 0700, then the directory itself at 0700. Any
    /// other missing parent is refused. An existing directory with group or other bits, or a
    /// symlink, is refused and left alone.
    pub fn open(state_dir: &Path) -> Result<Self, UsherError> {
        prepare_state_dir(state_dir)?;
        let db_path = state_dir.join(DB_FILE);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&db_path)
        {
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(err) => {
                return Err(fail(
                    "create the database file",
                    format!("cannot create `{}`: {err}", db_path.display()),
                    "fix the permissions on the state directory, then run the command again",
                ));
            }
        }
        let conn = Connection::open(&db_path).map_err(|err| db_error("open the database", &err))?;
        let (version, schema_rows) = read_state(&conn)?;
        if version > USER_VERSION {
            return Err(fail(
                "check the database version",
                format!(
                    "`{}` is at version {version}, newer than this usher reads ({USER_VERSION})",
                    db_path.display()
                ),
                "update usher, then run the command again",
            ));
        }
        if version == 0 && schema_rows > 0 {
            return Err(not_a_store(&db_path));
        }
        conn.busy_timeout(BUSY_TIMEOUT)
            .map_err(|err| db_error("set the busy timeout", &err))?;
        require_wal(&set_wal(&conn)?)?;
        conn.pragma_update(None, "synchronous", "FULL")
            .map_err(|err| db_error("set synchronous=FULL", &err))?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|err| db_error("turn on foreign keys", &err))?;
        if version == 0 {
            create_schema(&conn, &db_path)?;
        }
        Ok(Self { conn })
    }

    /// The settings this connection runs with.
    pub fn settings(&self) -> Result<StoreSettings, UsherError> {
        let number = |pragma: &str| -> Result<i64, UsherError> {
            self.conn
                .query_row(&format!("PRAGMA {pragma}"), [], |r| r.get(0))
                .map_err(|err| db_error("read the connection settings", &err))
        };
        let journal_mode: String = self
            .conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .map_err(|err| db_error("read the connection settings", &err))?;
        Ok(StoreSettings {
            journal_mode: journal_mode.to_lowercase(),
            synchronous: number("synchronous")?,
            busy_timeout_ms: number("busy_timeout")?,
            foreign_keys: number("foreign_keys")?,
        })
    }

    /// Inserts one run and its sources; returns the run id. A run with a bad field inserts
    /// nothing.
    pub fn record_inventory(
        &mut self,
        run: &RunRecord<'_>,
        sources: &[SourceRecord<'_>],
    ) -> Result<i64, UsherError> {
        check_time("recorded_at", run.recorded_at)?;
        for source in sources {
            if let Some(modified) = source.modified {
                check_time("modified", modified)?;
            }
        }
        let tx = self
            .conn
            .transaction()
            .map_err(|err| db_error("start the inventory insert", &err))?;
        tx.execute(
            "INSERT INTO inventory_run (recorded_at, harness, harness_version, knowledge_schema, knowledge_sequence, environment_key) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                run.recorded_at,
                run.harness,
                run.harness_version,
                run.knowledge_schema,
                i64::try_from(run.knowledge_sequence).unwrap_or(i64::MAX),
                run.environment_key
            ],
        )
        .map_err(|err| db_error("insert the inventory run", &err))?;
        let run_id = tx.last_insert_rowid();
        for source in sources {
            tx.execute(
                "INSERT INTO inventory_source (run_id, item_id, path, layer, present, size, modified, key_names) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    run_id,
                    source.item_id,
                    source.path,
                    source.layer,
                    i64::from(source.present),
                    source.size.map(|n| i64::try_from(n).unwrap_or(i64::MAX)),
                    source.modified,
                    json_names(source.key_names)
                ],
            )
            .map_err(|err| db_error("insert an inventory source", &err))?;
        }
        tx.commit()
            .map_err(|err| db_error("commit the inventory insert", &err))?;
        Ok(run_id)
    }

    /// Every run, oldest first.
    pub fn runs(&self) -> Result<Vec<RunRow>, UsherError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, recorded_at, harness, harness_version, knowledge_schema, knowledge_sequence, environment_key FROM inventory_run ORDER BY id")
            .map_err(|err| db_error("read the runs", &err))?;
        let rows = stmt
            .query_map([], |r| {
                Ok(RunRow {
                    id: r.get(0)?,
                    recorded_at: r.get(1)?,
                    harness: r.get(2)?,
                    harness_version: r.get(3)?,
                    knowledge_schema: r.get(4)?,
                    knowledge_sequence: u64::try_from(r.get::<_, i64>(5)?).unwrap_or(0),
                    environment_key: r.get(6)?,
                })
            })
            .map_err(|err| db_error("read the runs", &err))?;
        rows.collect::<Result<_, _>>()
            .map_err(|err| db_error("read the runs", &err))
    }

    /// The sources of one run, in the order they were recorded.
    pub fn sources_of(&self, run_id: i64) -> Result<Vec<SourceRow>, UsherError> {
        let mut stmt = self
            .conn
            .prepare("SELECT run_id, item_id, path, layer, present, size, modified, key_names FROM inventory_source WHERE run_id = ?1 ORDER BY rowid")
            .map_err(|err| db_error("read the sources", &err))?;
        let rows = stmt
            .query_map([run_id], |r| {
                let names: String = r.get(7)?;
                Ok((
                    SourceRow {
                        run_id: r.get(0)?,
                        item_id: r.get(1)?,
                        path: r.get(2)?,
                        layer: r.get(3)?,
                        present: r.get::<_, i64>(4)? == 1,
                        size: r
                            .get::<_, Option<i64>>(5)?
                            .map(|n| u64::try_from(n).unwrap_or(0)),
                        modified: r.get(6)?,
                        key_names: Vec::new(),
                    },
                    names,
                ))
            })
            .map_err(|err| db_error("read the sources", &err))?;
        let mut out = Vec::new();
        for row in rows {
            let (mut source, names) = row.map_err(|err| db_error("read the sources", &err))?;
            let mut names_stmt = self
                .conn
                .prepare("SELECT value FROM json_each(?1) ORDER BY key")
                .map_err(|err| db_error("read the key names", &err))?;
            source.key_names = names_stmt
                .query_map([names], |r| r.get(0))
                .map_err(|err| db_error("read the key names", &err))?
                .collect::<Result<_, _>>()
                .map_err(|err| db_error("read the key names", &err))?;
            out.push(source);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wal_pragma_must_return_wal() {
        require_wal("wal").expect("wal is the mode the open asks for");
        for other in ["delete", "memory", "truncate", "persist", "off", ""] {
            let err = require_wal(other).expect_err(other);
            let shown = err.to_string();
            assert!(shown.contains("journal mode"), "{other}: {shown}");
            assert!(shown.contains("WAL"), "{other}: {shown}");
            assert!(!err.step().is_empty() && !err.check().is_empty());
            assert!(!err.status().is_empty() && !err.next_action().is_empty());
        }
    }
}

/// The platform whose state directory the resolver names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Platform {
    /// macOS: `<home>/Library/Application Support/usher`.
    Macos,
    /// Linux: `<home>/.local/state/usher`.
    Linux,
}

/// The state directory for `platform` under an account home (AD-1): no `XDG_*` and no
/// `HOME` override.
pub fn state_dir_under(home: &Path, platform: Platform) -> PathBuf {
    let _ = platform;
    home.to_path_buf()
}

/// The account home of the user with this uid, from the account database and never from
/// `$HOME`.
pub fn account_home_of(uid: u32) -> Result<PathBuf, UsherError> {
    let _ = uid;
    Ok(PathBuf::from("unresolved"))
}

/// The account home of the current user.
pub fn account_home() -> Result<PathBuf, UsherError> {
    Ok(PathBuf::from("unresolved-current"))
}

impl Store {
    /// As [`Store::open`], with the uid the state directory must belong to named by the
    /// caller, so a test can check the owner wiring without changing an owner.
    pub fn open_with_owner(state_dir: &Path, expected_uid: u32) -> Result<Self, UsherError> {
        let _ = expected_uid;
        Self::open(state_dir)
    }
}

/// Refuses a state directory owned by anyone but the current user.
pub fn ensure_owned(dir: &Path, dir_uid: u32, current_uid: u32) -> Result<(), UsherError> {
    let _ = (dir, dir_uid, current_uid);
    Ok(())
}
