//! The fragment a probe prints holds version, outcome, date and network, and
//! nothing a host would leak.

use usher_probes::fragment::{Date, Fragment, Network, Outcome, bare_version};

fn date(year: u16, month: u8, day: u8) -> Date {
    Date::new(year, month, day).expect("a real date")
}

#[test]
fn a_passed_fragment_renders_as_one_inline_table() {
    let fragment = Fragment::new(
        "2.1.329",
        Outcome::Passed,
        date(2026, 11, 20),
        Network::Denied,
    )
    .expect("a bare version is accepted");
    assert_eq!(
        fragment.render(),
        r#"{ version = "2.1.329", outcome = "passed", date = 2026-11-20, network = "denied" }"#
    );
}

#[test]
fn a_failed_fragment_with_the_network_not_denied_says_so() {
    let fragment = Fragment::new(
        "0.160.1",
        Outcome::Failed,
        date(2026, 11, 8),
        Network::NotDenied,
    )
    .expect("a bare version is accepted");
    assert_eq!(
        fragment.render(),
        r#"{ version = "0.160.1", outcome = "failed", date = 2026-11-08, network = "not denied" }"#
    );
}

#[test]
fn a_version_that_carries_a_path_a_space_or_a_quote_is_refused() {
    let refused = [
        "/Users/someone/bin/claude",
        "codex-cli 0.160.1",
        "2.1.329\"",
        "",
    ];
    for version in refused {
        let result = Fragment::new(
            version,
            Outcome::Passed,
            date(2026, 11, 20),
            Network::Denied,
        );
        assert!(result.is_err(), "`{version}` must be refused");
    }
}

#[test]
fn a_date_that_does_not_exist_is_refused() {
    assert!(Date::new(2026, 13, 1).is_none(), "month 13");
    assert!(Date::new(2026, 0, 1).is_none(), "month 0");
    assert!(Date::new(2026, 11, 0).is_none(), "day 0");
    assert!(Date::new(2026, 2, 30).is_none(), "30 February");
    assert!(
        Date::new(2023, 2, 29).is_none(),
        "29 February in a common year"
    );
    assert!(
        Date::new(2024, 2, 29).is_some(),
        "29 February in a leap year"
    );
}

#[test]
fn a_date_prints_with_zero_padding() {
    assert_eq!(date(2026, 1, 5).to_string(), "2026-01-05");
}

#[test]
fn a_date_is_read_from_unix_seconds_in_utc() {
    assert_eq!(Date::from_unix_secs(0).to_string(), "1970-01-01");
    assert_eq!(
        Date::from_unix_secs(1_795_132_800).to_string(),
        "2026-11-20"
    );
    assert_eq!(
        Date::from_unix_secs(1_709_164_800 + 86_399).to_string(),
        "2024-02-29"
    );
    assert_eq!(
        Date::from_unix_secs(1_798_675_200).to_string(),
        "2026-12-31"
    );
}

#[test]
fn the_bare_version_is_taken_from_what_each_harness_prints() {
    assert_eq!(
        bare_version("codex-cli 0.160.1").as_deref(),
        Some("0.160.1")
    );
    assert_eq!(
        bare_version("2.1.329 (Claude Code)").as_deref(),
        Some("2.1.329")
    );
    assert_eq!(bare_version("1.59.1").as_deref(), Some("1.59.1"));
    assert_eq!(bare_version("no version in this line"), None);
    assert_eq!(bare_version(""), None);
}

#[test]
fn the_date_is_the_utc_calendar_date_at_the_edges_of_midnight() {
    // 2026-11-20 00:00:00 UTC is 1_795_132_800.
    assert_eq!(
        Date::from_unix_secs(1_795_132_799).to_string(),
        "2026-11-19"
    );
    assert_eq!(
        Date::from_unix_secs(1_795_132_800).to_string(),
        "2026-11-20"
    );
    assert_eq!(
        Date::from_unix_secs(1_795_219_199).to_string(),
        "2026-11-20"
    );
    assert_eq!(
        Date::from_unix_secs(1_795_219_200).to_string(),
        "2026-11-21"
    );
}

#[test]
fn a_version_must_have_the_shape_of_a_version() {
    let today = date(2026, 11, 20);
    // Digits and dots, with an optional pre-release suffix that starts with a
    // digit or a hyphen. A bare word could be a user name, a host name or a
    // key fragment, and the version is the fragment's only free text.
    for accepted in ["2.1.329", "0.160.1", "1.0.0-rc.1"] {
        assert!(
            Fragment::new(accepted, Outcome::Passed, today, Network::Denied).is_ok(),
            "control: `{accepted}` has the shape of a version"
        );
    }
    for refused in ["alex", "kinu", "sk-abc"] {
        assert!(
            Fragment::new(refused, Outcome::Passed, today, Network::Denied).is_err(),
            "`{refused}` must be refused"
        );
    }
}

#[test]
fn a_version_is_three_numbers_and_an_optional_known_prerelease_word() {
    let today = date(2026, 11, 20);
    for accepted in [
        "2.1.329",
        "0.160.1",
        "1.59.1",
        "0.161.0-alpha.3",
        "1.0.0-rc.1",
        "1.0.0-beta",
    ] {
        assert!(
            Fragment::new(accepted, Outcome::Passed, today, Network::Denied).is_ok(),
            "`{accepted}` must be accepted"
        );
    }
    for refused in [
        "1.2.3-alex",
        "1.2.3-sk-abc",
        "1.2.3-",
        "1.2.3-rc.x",
        "1.2.3-rc1",
        "01.2.3",
        "1.2",
    ] {
        assert!(
            Fragment::new(refused, Outcome::Passed, today, Network::Denied).is_err(),
            "`{refused}` must be refused"
        );
    }
    assert_eq!(
        bare_version("codex-cli 0.161.0-alpha.3").as_deref(),
        Some("0.161.0-alpha.3")
    );
    assert_eq!(bare_version("fakeh 1.2.3-alex"), None);
}
