//! Profiles: a profile is a directory with a `profile.toml` manifest and one
//! fragment per harness in that harness's own format. This crate reads the
//! manifest; sources and fragments come in later changes.
//!
//! Values are stored as written. Path tokens such as `${HOME}` stay
//! unresolved here.

#![forbid(unsafe_code)]

mod display;
mod manifest;

pub use display::display_name;
pub use manifest::{HarnessCover, Manifest, McpServer, Note, Tool};
