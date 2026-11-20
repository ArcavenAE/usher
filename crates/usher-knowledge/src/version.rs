//! Harness versions and the ranges that key knowledge items to them.

use std::fmt;

/// A strict three-part harness version, `major.minor.patch`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    /// Builds a version from its three parts.
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parses `1.2.3`: exactly three dot-separated numbers, nothing else.
    pub fn parse(text: &str) -> Result<Self, String> {
        let bad = || format!("`{text}` is not a three-part version like 1.2.3");
        let mut parts = text.trim().split('.');
        let mut next = || -> Result<u64, String> {
            let part = parts.next().ok_or_else(bad)?;
            let leading_zero = part.len() > 1 && part.starts_with('0');
            if part.is_empty() || leading_zero || !part.bytes().all(|b| b.is_ascii_digit()) {
                return Err(bad());
            }
            part.parse::<u64>().map_err(|_| bad())
        };
        let version = Self::new(next()?, next()?, next()?);
        if parts.next().is_some() {
            return Err(bad());
        }
        Ok(version)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Ge,
    Gt,
    Le,
    Lt,
    Eq,
}

/// A set of comparators joined by commas, all of which must hold:
/// `>=1.2.0, <2.0.0`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRange {
    terms: Vec<(Op, Version)>,
}

impl VersionRange {
    /// Parses a comma-joined list of `>=`, `>`, `<=`, `<` or `=` comparators.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut terms = Vec::new();
        for raw in text.split(',') {
            let term = raw.trim();
            let (op, rest) = if let Some(rest) = term.strip_prefix(">=") {
                (Op::Ge, rest)
            } else if let Some(rest) = term.strip_prefix("<=") {
                (Op::Le, rest)
            } else if let Some(rest) = term.strip_prefix('>') {
                (Op::Gt, rest)
            } else if let Some(rest) = term.strip_prefix('<') {
                (Op::Lt, rest)
            } else if let Some(rest) = term.strip_prefix('=') {
                (Op::Eq, rest)
            } else {
                return Err(format!(
                    "`{term}` in version range `{text}` needs one of >=, >, <=, <, ="
                ));
            };
            terms.push((op, Version::parse(rest)?));
        }
        let range = Self { terms };
        if !range.is_satisfiable() {
            return Err(format!("version range `{text}` matches no version"));
        }
        Ok(range)
    }

    /// A range is satisfiable when some version near its bounds satisfies
    /// every comparator; the candidates are 0.0.0, each bound, and each bound
    /// with the patch number raised by one.
    fn is_satisfiable(&self) -> bool {
        std::iter::once(Version::new(0, 0, 0))
            .chain(self.terms.iter().flat_map(|(_, v)| {
                [
                    *v,
                    Version::new(v.major, v.minor, v.patch.saturating_add(1)),
                ]
            }))
            .any(|candidate| self.contains(&candidate))
    }

    /// Whether some version satisfies both ranges. The candidates are the same as for
    /// satisfiability: 0.0.0, each bound of either range, and each bound with the patch
    /// number raised by one.
    pub fn overlaps(&self, other: &VersionRange) -> bool {
        std::iter::once(Version::new(0, 0, 0))
            .chain(
                self.terms
                    .iter()
                    .chain(other.terms.iter())
                    .flat_map(|(_, v)| {
                        [
                            *v,
                            Version::new(v.major, v.minor, v.patch.saturating_add(1)),
                        ]
                    }),
            )
            .any(|candidate| self.contains(&candidate) && other.contains(&candidate))
    }

    /// Whether the two ranges match exactly the same versions.
    pub fn same_versions(&self, other: &VersionRange) -> bool {
        std::iter::once(Version::new(0, 0, 0))
            .chain(
                self.terms
                    .iter()
                    .chain(other.terms.iter())
                    .flat_map(|(_, v)| {
                        [
                            Version::new(v.major, v.minor, v.patch.saturating_sub(1)),
                            *v,
                            Version::new(v.major, v.minor, v.patch.saturating_add(1)),
                        ]
                    }),
            )
            .all(|candidate| self.contains(&candidate) == other.contains(&candidate))
    }

    /// Whether `version` satisfies every comparator.
    pub fn contains(&self, version: &Version) -> bool {
        self.terms.iter().all(|(op, bound)| match op {
            Op::Ge => version >= bound,
            Op::Gt => version > bound,
            Op::Le => version <= bound,
            Op::Lt => version < bound,
            Op::Eq => version == bound,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).expect("a valid version")
    }

    #[test]
    fn a_range_includes_and_excludes_by_its_comparators() {
        let range = VersionRange::parse(">=1.2.0, <2.0.0").expect("range");
        assert!(range.contains(&v("1.2.0")));
        assert!(range.contains(&v("1.9.9")));
        assert!(!range.contains(&v("2.0.0")));
        assert!(!range.contains(&v("1.1.9")));
    }

    #[test]
    fn versions_compare_numerically_not_as_text() {
        let range = VersionRange::parse(">=1.9.0").expect("range");
        assert!(range.contains(&v("1.10.0")));
    }

    #[test]
    fn each_comparator_works() {
        assert!(
            VersionRange::parse("=1.0.0")
                .expect("eq")
                .contains(&v("1.0.0"))
        );
        assert!(
            !VersionRange::parse(">1.0.0")
                .expect("gt")
                .contains(&v("1.0.0"))
        );
        assert!(
            VersionRange::parse("<=1.0.0")
                .expect("le")
                .contains(&v("1.0.0"))
        );
    }

    #[test]
    fn malformed_versions_and_ranges_are_errors() {
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
    }
}
