//! The settings-map port (AD-6, Story 1.6): what an adapter says about the value in
//! effect for each setting a harness reads, which layer and file it came from, and why.
//!
//! Types only. Adapters implement [`SettingsMapper`] over a [`SourceListing`] and the
//! files it names. Values appear only for keys knowledge marks readable and not
//! sensitive (AD-8).

use std::path::PathBuf;

use crate::{LaunchContext, SourceListing, UsherError};

/// How a reading was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReadingMethod {
    /// Usher rebuilt it from the files and the knowledge file.
    Rebuilt,
    /// The harness reported it.
    HarnessReported,
}

impl ReadingMethod {
    /// The stored token.
    pub fn token(self) -> &'static str {
        ""
    }
}

/// What the map says about one reading. The tokens are the stored ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReadingStatus {
    /// This layer's value, or this list entry, applies.
    InEffect,
    /// A scalar set here is outranked by a higher layer that sets the key.
    Shadowed,
    /// The harness does not read this key from this layer.
    Unread,
    /// Own rule, no cited scope, or an undecidable first-wins.
    NotDecided,
    /// The key's override variable is present.
    OverriddenByEnvironment,
    /// The key is not marked readable, or is sensitive.
    ValueNotRead,
    /// The source could not be placed or read.
    NotChecked,
    /// The version is unknown or the file did not parse.
    Unknown,
}

impl ReadingStatus {
    /// The stored token.
    pub fn token(self) -> &'static str {
        ""
    }

    /// The words the map prints for the status.
    pub fn display(self) -> &'static str {
        ""
    }
}

/// One reading: one key (or one list entry) in one layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingReading {
    /// The key path, dotted.
    pub key: String,
    /// The layer, as knowledge names it.
    pub layer: String,
    /// The file the reading came from.
    pub source_path: PathBuf,
    /// How it was obtained.
    pub method: ReadingMethod,
    /// What the map says about it.
    pub status: ReadingStatus,
    /// The entry's position in its layer's array, for a list key.
    pub entry_index: Option<u32>,
    /// The value as text, only for a readable key that is not sensitive.
    pub value: Option<String>,
    /// The reason or the cross-reference the status rests on.
    pub note: Option<String>,
}

/// A layer whose file took no part in deciding values, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedLayer {
    /// The layer, as knowledge names it.
    pub layer: String,
    /// The reasons knowledge states, and that the harness gave none.
    pub reason: String,
}

/// Everything one adapter decided for one environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsMap {
    /// The harness id.
    pub harness: String,
    /// The harness version the readings assume, or `None` when it is not known.
    pub harness_version: Option<String>,
    /// Every reading, by key, then layer from highest to lowest, then entry.
    pub readings: Vec<SettingReading>,
    /// Layers that took no part in deciding values.
    pub skipped_layers: Vec<SkippedLayer>,
}

/// The port an adapter implements to map its settings.
pub trait SettingsMapper {
    /// Maps the settings of one launch context over the listing found for it.
    fn map_settings(
        &self,
        listing: &SourceListing,
        context: &LaunchContext,
    ) -> Result<SettingsMap, UsherError>;
}
