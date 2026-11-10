//! Test support for usher.
//!
//! This crate exists so every story tests the same way (architecture decision
//! AD-20): scratch homes, the observation invariant over the real harness
//! homes, and pinned harness binaries. Fixtures captured from real harness
//! output arrive with the adapter stories that need them.
//!
//! Unix only for now (macOS first): the invariant reads ctime and inode.

#![forbid(unsafe_code)]

pub mod harness;
pub mod invariant;
pub mod scratch;

pub use harness::{Probe, Skip, probe, skip, skipped};
pub use invariant::{
    InvariantViolation, NamedHome, RealHomeGuard, RealHomes, TreeDigest, digest_tree, observe,
};
pub use scratch::ScratchHome;

/// The crate name, for tests that assert the workspace layout.
pub const CRATE_NAME: &str = "usher-testkit";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_matches_the_workspace_member() {
        assert_eq!(CRATE_NAME, env!("CARGO_PKG_NAME"));
    }
}
