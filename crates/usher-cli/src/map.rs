//! The `map` verb: prints which value is in effect for each setting, from which layer and
//! why, and records the run.

use std::path::Path;

use usher_core::{SystemRoot, UsherError};
use usher_knowledge::Knowledge;

use crate::InventoryEnv;

/// Runs `usher map <dir>` against `env`, placing absolute knowledge paths under `root`,
/// and returns the text to print. Only tests pass a root other than the host's.
pub fn map(env: &InventoryEnv, dir: &Path, root: SystemRoot) -> Result<String, UsherError> {
    map_with(env, dir, root, &Knowledge::embedded()?)
}

/// As [`map`], over a given knowledge, for tests that need a fact the seed does not hold.
pub fn map_with(
    _env: &InventoryEnv,
    _dir: &Path,
    _root: SystemRoot,
    _knowledge: &Knowledge,
) -> Result<String, UsherError> {
    Ok(String::new())
}
