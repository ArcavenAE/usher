//! The source-listing port (AD-6, Story 1.5): what an adapter says about the
//! settings sources of one harness on one host.
//!
//! Types only. Adapters implement [`SourceLister`]; modes and the store take the
//! [`SourceListing`] it returns. Derived facts only: presence, path, size,
//! modified time and key names, never a value (AD-8).

use std::path::PathBuf;

use crate::{EnvironmentKey, UsherError};

/// Where to look for one harness: the launch context an adapter resolves paths from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchContext {
    /// The environment's key (AD-17).
    pub key: EnvironmentKey,
    /// The OS user's home directory, for `${HOME}`.
    pub home: PathBuf,
    /// The harness config directory, for `${CONFIG_DIR}`, already resolved from
    /// the harness's relocation variable.
    pub config_dir: PathBuf,
    /// The repository root, for `${REPO}`, when the directory is in one.
    pub repo: Option<PathBuf>,
    /// The OS user's name, for `${USER}`.
    pub user: String,
    /// The installed harness version as `major.minor.patch`, when known.
    pub harness_version: Option<String>,
}

/// What usher can say about one settings source (FR-43).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceStatus {
    /// The version reads it, and nothing hides it.
    InEffect,
    /// A present source ranked above it (a smaller rank number; 1 is highest) hides it;
    /// the row names the winner.
    Shadowed {
        /// The source that wins.
        by: PathBuf,
    },
    /// The version does not read it, and why: the knowledge item that says so, or that
    /// a first-wins source delivers no policy key.
    Unread {
        /// The text the row prints inside its parentheses.
        reason: String,
    },
    /// Usher cannot place it, and says why.
    Unknown {
        /// Why the status is unknown.
        reason: String,
    },
}

/// The key names of a source, never their values.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyNames {
    /// The top-level key names, with the stamp that says how far knowledge
    /// could classify them.
    Names {
        /// The names found, sorted by name.
        names: Vec<String>,
        /// Why the names carry no recognized or unrecognized label.
        classification: String,
    },
    /// The names were not read, and why.
    NotRead {
        /// Why the names were not read.
        reason: String,
    },
}

/// One settings source that exists on the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedSource {
    /// Where it is.
    pub path: PathBuf,
    /// The harness it belongs to.
    pub harness: String,
    /// The layer it belongs to, as knowledge names it.
    pub layer: String,
    /// The id of the knowledge item that states the source.
    pub item_id: String,
    /// Its one status.
    pub status: SourceStatus,
    /// Size in bytes, when it could be read.
    pub size: Option<u64>,
    /// Modified time in seconds since the Unix epoch, when the system gives one.
    pub modified: Option<u64>,
    /// Its key names.
    pub keys: KeyNames,
}

/// A layer knowledge states that has no file on this host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsentLayer {
    /// The layer, as knowledge names it.
    pub layer: String,
    /// The paths checked.
    pub paths: Vec<PathBuf>,
    /// The id of the knowledge item behind each path, in the same order.
    pub item_ids: Vec<String>,
}

/// A layer knowledge says always exists whose path could not be resolved, so it was
/// not looked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotCheckedLayer {
    /// The layer, as knowledge names it.
    pub layer: String,
    /// Why the layer was not checked.
    pub reason: String,
}

/// The root that a knowledge path written as absolute (a leading `/` before any token)
/// is read under: `/` on a host. Only the test kit builds another root; no flag,
/// variable or file reaches it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemRoot(PathBuf);

impl SystemRoot {
    /// The host's own root.
    pub fn host() -> Self {
        SystemRoot(PathBuf::from("/"))
    }

    /// A scratch root, for tests that must not read the host's system paths.
    pub fn at(root: PathBuf) -> Self {
        SystemRoot(root)
    }

    /// The directory absolute knowledge paths are placed under.
    pub fn path(&self) -> &std::path::Path {
        &self.0
    }
}

/// Everything one adapter found for one environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceListing {
    /// The harness id.
    pub harness: String,
    /// The harness version the statuses assume, or `None` when it is not known.
    pub harness_version: Option<String>,
    /// The knowledge version the statuses rest on, as its report line reads.
    pub knowledge_version: String,
    /// Present sources, highest layer first.
    pub sources: Vec<ListedSource>,
    /// Layers checked with no file present.
    pub absent_layers: Vec<AbsentLayer>,
    /// Always-present layers whose path could not be resolved.
    pub not_checked: Vec<NotCheckedLayer>,
}

/// The port an adapter implements to list its settings sources.
pub trait SourceLister {
    /// Lists the sources for one launch context.
    fn list_sources(&self, context: &LaunchContext) -> Result<SourceListing, UsherError>;
}
