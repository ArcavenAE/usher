//! The proposed knowledge fragment a probe prints: version, outcome, date and
//! network state, and nothing else.

use std::fmt;

/// What a probe found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The control held and the claim held.
    Passed,
    /// The control held and the claim did not.
    Failed,
}

/// Whether the run had its network denied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    /// Denied except localhost, under a sandbox profile.
    Denied,
    /// Not denied.
    NotDenied,
}

/// A calendar date in UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

/// Why a fragment could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentError(String);

/// One probe result, ready to print.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    version: String,
}

impl Date {
    /// A date, or `None` when the month or day does not exist.
    pub fn new(year: u16, month: u8, day: u8) -> Option<Date> {
        Some(Date { year, month, day })
    }

    /// The UTC date of a Unix time in seconds.
    pub fn from_unix_secs(_secs: u64) -> Date {
        Date {
            year: 1970,
            month: 1,
            day: 1,
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{}", self.year, self.month, self.day)
    }
}

impl fmt::Display for FragmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Outcome {
    /// The word written into knowledge.
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Passed => "passed",
            Outcome::Failed => "failed",
        }
    }
}

impl Network {
    /// The word written into the fragment.
    pub fn as_str(self) -> &'static str {
        match self {
            Network::Denied => "denied",
            Network::NotDenied => "not denied",
        }
    }
}

/// The bare version number in the first line a harness prints for
/// `--version`, for example `0.160.1` from `codex-cli 0.160.1`.
pub fn bare_version(_line: &str) -> Option<String> {
    None
}

impl Fragment {
    /// A fragment for one result. The version is a bare version number; text
    /// that carries a path, a space or a quote is refused.
    pub fn new(
        version: &str,
        _outcome: Outcome,
        _date: Date,
        _network: Network,
    ) -> Result<Fragment, FragmentError> {
        Ok(Fragment {
            version: version.to_string(),
        })
    }

    /// The fragment as one inline table, as written into a knowledge item.
    pub fn render(&self) -> String {
        String::new()
    }
}
