//! Test support for usher.
//!
//! This crate exists from the first story so every later story tests the same
//! way (architecture decision AD-20): scratch homes, fixtures captured from
//! real harness output, and the invariant checks. Story 1.2 adds the scratch
//! home and the observation invariant; until then the crate holds only its own
//! name so the workspace shape is settled.

#![forbid(unsafe_code)]

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
