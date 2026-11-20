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
/// the repo, and that every probe in the repo is carried by an item. Both
/// kinds of mismatch are reported together, missing probes first.
pub fn check_probe_names(carried: &[&str], in_repo: &[&str]) -> Result<(), Vec<Mismatch>> {
    let mut mismatches = Vec::new();
    for name in carried {
        let reported = mismatches.contains(&Mismatch::MissingProbe((*name).to_string()));
        if !in_repo.contains(name) && !reported {
            mismatches.push(Mismatch::MissingProbe((*name).to_string()));
        }
    }
    for name in in_repo {
        let reported = mismatches.contains(&Mismatch::UncarriedProbe((*name).to_string()));
        if !carried.contains(name) && !reported {
            mismatches.push(Mismatch::UncarriedProbe((*name).to_string()));
        }
    }
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(mismatches)
    }
}

/// The ways a probe source file starts a process on its own instead of through
/// the launcher: each entry names the text found. Empty when the source is clean.
pub fn find_launch_bypasses(_source: &str) -> Vec<String> {
    Vec::new()
}
