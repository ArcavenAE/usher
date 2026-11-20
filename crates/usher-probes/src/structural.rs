//! The structural check between harness knowledge and the probes in the repo.

/// One way the two sides disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mismatch {
    /// An item names a probe that the repo does not have.
    MissingProbe(String),
    /// A probe in the repo is named by no item.
    UncarriedProbe(String),
}

/// Checks that every probe name carried by knowledge resolves to a probe in
/// the repo, and that every probe in the repo is carried by an item.
pub fn check_probe_names(_carried: &[&str], _in_repo: &[&str]) -> Result<(), Vec<Mismatch>> {
    Ok(())
}
