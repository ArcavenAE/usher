//! Harness knowledge for usher: one TOML schema in two layers, a structural
//! check that every item carries a reference and a last-verified date, the
//! file embedded in each release, and the support level usher reports for a
//! harness version.
//!
//! A signed update replaces the embedded file only when its signature, its
//! structure and its sequence number all check out ([`Knowledge::update`]).

#![forbid(unsafe_code)]

mod location;
mod schema;
mod support;
mod trust;
mod update;
mod version;

#[cfg(test)]
#[path = "../build/keys.rs"]
mod build_keys;

pub use location::{Location, PLACEHOLDERS, Platform, Scope, Shape};
pub use schema::{
    Class, Date, Item, KnowledgeFile, KnowledgeVersion, Layer, Reference, ReferenceKind,
};
pub use support::{Capability, SupportReport};
pub use trust::TrustedKeys;
pub use update::Knowledge;
pub use version::{Version, VersionRange};
