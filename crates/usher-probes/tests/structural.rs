//! Every probe name that knowledge carries resolves to a probe in the repo,
//! and every probe in the repo is carried by an item.

use usher_probes::structural::{Mismatch, check_probe_names};

#[test]
fn a_probe_named_by_knowledge_but_missing_from_the_repo_is_reported() {
    assert_eq!(
        check_probe_names(&["alpha", "beta"], &["alpha", "beta"]),
        Ok(()),
        "control: matching sets pass"
    );
    assert_eq!(
        check_probe_names(&["alpha", "beta"], &["alpha"]),
        Err(vec![Mismatch::MissingProbe("beta".to_string())])
    );
}

#[test]
fn a_probe_in_the_repo_that_no_item_carries_is_reported() {
    assert_eq!(
        check_probe_names(&["alpha"], &["alpha"]),
        Ok(()),
        "control: matching sets pass"
    );
    assert_eq!(
        check_probe_names(&["alpha"], &["alpha", "gamma"]),
        Err(vec![Mismatch::UncarriedProbe("gamma".to_string())])
    );
}

#[test]
fn both_kinds_of_mismatch_are_reported_together() {
    let result = check_probe_names(&["alpha", "beta"], &["alpha", "gamma"]);
    let mismatches = result.expect_err("two mismatches");
    assert!(mismatches.contains(&Mismatch::MissingProbe("beta".to_string())));
    assert!(mismatches.contains(&Mismatch::UncarriedProbe("gamma".to_string())));
    assert_eq!(mismatches.len(), 2);
}
