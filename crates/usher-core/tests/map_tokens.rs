//! Story 1.6, PR C: the eight status tokens and the two method tokens are the stored
//! ones, and each status has its own display words. The mapping is one to one and pinned.
//! Public API.

use std::collections::BTreeSet;

use usher_core::{ReadingMethod, ReadingStatus};

const ALL: [(ReadingStatus, &str, &str); 8] = [
    (ReadingStatus::InEffect, "in_effect", "in effect"),
    (ReadingStatus::Shadowed, "shadowed", "shadowed"),
    (ReadingStatus::Unread, "unread", "unread"),
    (ReadingStatus::NotDecided, "not_decided", "not decided"),
    (
        ReadingStatus::OverriddenByEnvironment,
        "overridden_by_environment",
        "overridden by environment",
    ),
    (
        ReadingStatus::ValueNotRead,
        "value_not_read",
        "value not read",
    ),
    (ReadingStatus::NotChecked, "not_checked", "not checked"),
    (ReadingStatus::Unknown, "unknown", "unknown"),
];

#[test]
fn each_status_has_the_stored_token() {
    for (status, token, _) in ALL {
        assert_eq!(status.token(), token, "{status:?}");
    }
}

#[test]
fn each_status_has_its_own_display_words() {
    for (status, _, words) in ALL {
        assert_eq!(status.display(), words, "{status:?}");
    }
}

#[test]
fn the_tokens_and_the_display_words_are_each_distinct() {
    let tokens: BTreeSet<&str> = ALL.iter().map(|(s, _, _)| s.token()).collect();
    let words: BTreeSet<&str> = ALL.iter().map(|(s, _, _)| s.display()).collect();
    assert_eq!(tokens.len(), 8, "{tokens:?}");
    assert_eq!(words.len(), 8, "{words:?}");
    assert!(!tokens.contains(""), "an empty token is no token");
}

#[test]
fn the_method_tokens_are_the_stored_ones() {
    assert_eq!(ReadingMethod::Rebuilt.token(), "rebuilt");
    assert_eq!(ReadingMethod::HarnessReported.token(), "harness-reported");
}
