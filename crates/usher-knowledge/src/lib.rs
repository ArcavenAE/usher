//! Harness knowledge for usher: one TOML schema in two layers, a structural
//! check that every item carries a reference and a last-verified date, the
//! file embedded in each release, and the support level usher reports for a
//! harness version.
//!
//! Signed updates and sequence checks arrive in Story 1.4; this crate only
//! reads the embedded file and reports which version it used.

#![forbid(unsafe_code)]

mod schema;
mod support;
mod version;

pub use schema::{
    Class, Date, Item, KnowledgeFile, KnowledgeVersion, Layer, Reference, ReferenceKind,
};
pub use support::{Capability, SupportReport};
pub use version::{Version, VersionRange};
