//! The usher store (AD-7): one SQLite database in the owner-only state directory.
//!
//! Story 1.5, change B holds the inventory tables only: derived state, inserted and
//! never changed.

// RED-STAGE STUB (Story 1.5, PR B): the items below give the red tests something to
// compile against and do nothing else. The implementation commit replaces them.

use std::path::Path;

use usher_core::UsherError;

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
    _private: Vec<u8>,
}

fn stub() -> UsherError {
    UsherError::new(
        "red-stage stub",
        "usher-store has no implementation yet",
        "not implemented",
        "write the implementation commit",
    )
}

impl Store {
    /// Opens the store in `state_dir`, creating the directory and the database.
    pub fn open(_state_dir: &Path) -> Result<Self, UsherError> {
        Err(stub())
    }

    /// The settings this connection runs with.
    pub fn settings(&self) -> Result<StoreSettings, UsherError> {
        Err(stub())
    }

    /// Inserts one run and its sources; returns the run id.
    pub fn record_inventory(
        &mut self,
        _run: &RunRecord<'_>,
        _sources: &[SourceRecord<'_>],
    ) -> Result<i64, UsherError> {
        Err(stub())
    }

    /// Every run, oldest first.
    pub fn runs(&self) -> Result<Vec<RunRow>, UsherError> {
        Err(stub())
    }

    /// The sources of one run, in the order they were recorded.
    pub fn sources_of(&self, _run_id: i64) -> Result<Vec<SourceRow>, UsherError> {
        Err(stub())
    }
}
