//! Harness versions and the ranges that key knowledge items and profiles to them.
//!
//! This is a stub: the types exist so the tests compile, and nothing uses them
//! yet.

use std::fmt;

/// A strict three-part harness version, `major.minor.patch`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version;

impl Version {
    /// Builds a version from its three parts.
    pub fn new(_major: u64, _minor: u64, _patch: u64) -> Self {
        Self
    }

    /// Parses `1.2.3`: exactly three dot-separated numbers, nothing else.
    pub fn parse(text: &str) -> Result<Self, String> {
        Err(format!("`{text}` is not parsed yet"))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("0.0.0")
    }
}

/// A set of comparators joined by commas, all of which must hold:
/// `>=1.2.0, <2.0.0`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRange;

impl VersionRange {
    /// Parses a comma-joined list of `>=`, `>`, `<=`, `<` or `=` comparators.
    pub fn parse(text: &str) -> Result<Self, String> {
        Err(format!("`{text}` is not parsed yet"))
    }

    /// Whether some version satisfies both ranges.
    pub fn overlaps(&self, _other: &VersionRange) -> bool {
        false
    }

    /// Whether the two ranges match exactly the same versions.
    pub fn same_versions(&self, _other: &VersionRange) -> bool {
        false
    }

    /// Whether `version` satisfies every comparator.
    pub fn contains(&self, _version: &Version) -> bool {
        false
    }
}
