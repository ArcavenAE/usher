//! How a name from a profile is shown in an error.

use std::fmt::Write;

/// The characters an error never carries raw: every control character
/// (U+0000 to U+001F, U+007F and U+0080 to U+009F, exactly `char::is_control`)
/// and the bidirectional format controls U+200E, U+200F, U+202A to U+202E and
/// U+2066 to U+2069, which can make text read in an order it is not stored in.
pub(crate) fn is_escaped(c: char) -> bool {
    c.is_control()
        || matches!(
            u32::from(c),
            0x200E | 0x200F | 0x202A..=0x202E | 0x2066..=0x2069
        )
}

/// A name as an error shows it: each character in the escape set (ruling 7.1
/// PR A, Q7: C0, DEL, C1 and the bidi format controls, see [`is_escaped`]) is
/// written as `\u` and four uppercase hex digits, and everything else stays
/// as written. An error built from it never carries a raw control character,
/// an escape sequence included.
pub fn display_name(text: &str) -> String {
    let mut shown = String::with_capacity(text.len());
    for c in text.chars() {
        if is_escaped(c) {
            // Writing to a String cannot fail.
            let _ = write!(shown, "\\u{:04X}", u32::from(c));
        } else {
            shown.push(c);
        }
    }
    shown
}
