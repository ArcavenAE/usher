//! Version and VersionRange keep the behavior they had in usher-knowledge.

use usher_core::{Version, VersionRange};

fn v(text: &str) -> Version {
    Version::parse(text).expect("a valid version")
}

fn range(text: &str) -> VersionRange {
    VersionRange::parse(text).expect("a valid range")
}

#[test]
fn a_range_includes_and_excludes_by_its_comparators() {
    let range = range(">=1.2.0, <2.0.0");
    assert!(range.contains(&v("1.2.0")));
    assert!(range.contains(&v("1.9.9")));
    assert!(!range.contains(&v("2.0.0")));
    assert!(!range.contains(&v("1.1.9")));
}

#[test]
fn versions_compare_numerically_not_as_text() {
    assert!(range(">=1.9.0").contains(&v("1.10.0")));
    assert!(v("1.10.0") > v("1.9.0"));
    assert_eq!(Version::new(1, 2, 3), v("1.2.3"));
}

#[test]
fn each_comparator_works() {
    assert!(range("=1.0.0").contains(&v("1.0.0")));
    assert!(!range(">1.0.0").contains(&v("1.0.0")));
    assert!(range("<=1.0.0").contains(&v("1.0.0")));
    assert!(!range("<1.0.0").contains(&v("1.0.0")));
    assert!(range(">=1.0.0").contains(&v("1.0.0")));
}

#[test]
fn malformed_versions_and_ranges_are_errors() {
    assert!(
        Version::parse("1.2.3").is_ok(),
        "control: a plain version parses"
    );
    assert!(
        VersionRange::parse(">=1.0.0").is_ok(),
        "control: a plain range parses"
    );
    for bad in [
        "1.2", "1.2.3.4", "v1.2.3", "1.2.x", "", "1..3", "-1.0.0", "01.2.3",
    ] {
        assert!(Version::parse(bad).is_err(), "{bad} must not parse");
    }
    for bad in ["", "1.0.0", ">=1.0.0,", ">= x", "~1.0.0"] {
        assert!(VersionRange::parse(bad).is_err(), "{bad} must not parse");
    }
}

#[test]
fn a_range_that_matches_no_version_is_an_error() {
    for bad in [
        ">=2.0.0, <1.0.0",
        "=1.0.0, =2.0.0",
        ">1.0.0, <1.0.1",
        ">=1.0.0, <1.0.0",
    ] {
        assert!(VersionRange::parse(bad).is_err(), "{bad} must not parse");
    }
    for good in ["<1.0.0", ">1.0.0, <1.0.5", ">=0.0.0", "=3.1.4"] {
        assert!(VersionRange::parse(good).is_ok(), "{good} must parse");
    }
}

#[test]
fn a_version_displays_as_three_parts() {
    assert_eq!(v("0.160.1").to_string(), "0.160.1");
    assert_eq!(Version::new(2, 1, 329).to_string(), "2.1.329");
}

#[test]
fn two_ranges_overlap_when_some_version_satisfies_both() {
    assert!(range(">=1.0.0, <2.0.0").overlaps(&range(">=1.5.0")));
    assert!(!range(">=1.0.0, <2.0.0").overlaps(&range(">=2.0.0")));
    assert!(range("=1.0.0").overlaps(&range("<=1.0.0")));
    assert!(!range("=1.0.0").overlaps(&range(">1.0.0")));
}

#[test]
fn two_ranges_are_the_same_when_they_match_exactly_the_same_versions() {
    assert!(range(">=1.0.0, <2.0.0").same_versions(&range("<2.0.0, >=1.0.0")));
    assert!(!range(">=1.0.0, <2.0.0").same_versions(&range(">=1.0.0, <2.0.1")));
    assert!(!range(">=1.0.0").same_versions(&range(">1.0.0")));
}

#[test]
fn surrounding_whitespace_is_trimmed_from_a_version_and_a_range_term() {
    assert_eq!(
        Version::parse(" 1.2.3 ").expect("a trimmed version"),
        Version::new(1, 2, 3)
    );
    assert!(range(">= 1.0.0").contains(&v("1.0.0")));
    assert!(range(" >=1.0.0 , <2.0.0 ").contains(&v("1.5.0")));
}
