//! The Claude Code adapter (Story 1.5, FR-43): finds the settings sources of Claude
//! Code on a host and lists them through the source-listing port.
//!
//! Locations, layers, read rules and the version link come from harness knowledge,
//! not from this crate (AD-6). The harness is never run.

#![forbid(unsafe_code)]

pub mod version;

pub use version::{VersionReading, read_version};
