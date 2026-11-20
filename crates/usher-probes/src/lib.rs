//! Re-runnable probes for harness claims.
//!
//! A probe runs a real harness under a scratch home, checks its positive
//! control first, then the claim, and prints the result as a fragment a
//! maintainer commits into harness knowledge. A run never writes the knowledge
//! file. A probe that cannot run fails with `not run: <reason>`; it never
//! reports a pass.

#![forbid(unsafe_code)]

pub mod codex_home;
pub mod fragment;
pub mod network;
pub mod run;
pub mod structural;

/// The crate name, for tests that assert the workspace layout.
pub const CRATE_NAME: &str = "usher-probes";
