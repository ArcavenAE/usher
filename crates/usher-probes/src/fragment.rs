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
    outcome: Outcome,
    date: Date,
    network: Network,
}

const SECS_PER_DAY: u64 = 86_400;

fn is_leap(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: u16, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 if is_leap(year) => Some(29),
        2 => Some(28),
        _ => None,
    }
}

impl Date {
    /// A date, or `None` when the month or day does not exist.
    pub fn new(year: u16, month: u8, day: u8) -> Option<Date> {
        let last = days_in_month(year, month)?;
        if day == 0 || day > last {
            return None;
        }
        Some(Date { year, month, day })
    }

    /// The UTC date of a Unix time in seconds. A date past the year 9999 is
    /// clamped to 9999-12-31, which no real clock reaches.
    pub fn from_unix_secs(secs: u64) -> Date {
        // Civil date from a day count, after Howard Hinnant's algorithm.
        let days = i64::try_from(secs / SECS_PER_DAY).unwrap_or(i64::MAX / 2);
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        match (u16::try_from(year), u8::try_from(month), u8::try_from(day)) {
            (Ok(year), Ok(month), Ok(day)) if year <= 9999 => Date { year, month, day },
            _ => Date {
                year: 9999,
                month: 12,
                day: 31,
            },
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
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

/// True for the shape of a version: digits and dots, starting and ending with
/// a digit and with no empty part, then an optional suffix that starts with a
/// hyphen (a digit would already belong to the number) and holds only
/// letters, digits, dots and hyphens.
fn has_version_shape(text: &str) -> bool {
    let number_end = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(text.len());
    let (number, suffix) = text.split_at(number_end);
    let parts_ok = !number.is_empty() && number.split('.').all(|part| !part.is_empty());
    let suffix_ok = suffix.is_empty()
        || (suffix.starts_with('-')
            && suffix
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
            && suffix.len() > 1);
    parts_ok && suffix_ok
}

/// The bare version number in the first line a harness prints for
/// `--version`, for example `0.160.1` from `codex-cli 0.160.1`.
pub fn bare_version(line: &str) -> Option<String> {
    line.split_whitespace()
        .map(|word| word.trim_matches(|c| matches!(c, '(' | ')' | ',' | ';')))
        .find(|word| has_version_shape(word))
        .map(str::to_string)
}

impl Fragment {
    /// A fragment for one result. The version must have the shape of a
    /// version; anything else, a path, a user or host name or a key piece
    /// included, is refused.
    pub fn new(
        version: &str,
        outcome: Outcome,
        date: Date,
        network: Network,
    ) -> Result<Fragment, FragmentError> {
        if !has_version_shape(version) {
            return Err(FragmentError(
                "the version does not have the shape of a version number".to_string(),
            ));
        }
        Ok(Fragment {
            version: version.to_string(),
            outcome,
            date,
            network,
        })
    }

    /// The fragment as one inline table, as written into a knowledge item.
    pub fn render(&self) -> String {
        format!(
            "{{ version = \"{}\", outcome = \"{}\", date = {}, network = \"{}\" }}",
            self.version,
            self.outcome.as_str(),
            self.date,
            self.network.as_str()
        )
    }
}
