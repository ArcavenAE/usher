//! Profiles: a profile is a directory with a `profile.toml` manifest and one
//! fragment per harness in that harness's own format. This crate reads the
//! manifest and, from a directory or a file source, the fragments beside it;
//! conflicts and git sources come in later changes.
//!
//! Values are stored as written. Path tokens such as `${HOME}` stay
//! unresolved here.

#![forbid(unsafe_code)]

mod display;
mod fragment;
mod manifest;
mod source;

pub use display::display_name;
pub use manifest::{HarnessCover, Manifest, McpServer, Note, Tool};
pub use source::{
    CredentialFlag, Fragment, FragmentFormat, HarnessRule, LoadRules, McpEntry, McpOrigin, Profile,
    Source,
};
