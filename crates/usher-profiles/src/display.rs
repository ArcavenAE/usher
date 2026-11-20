//! How a name from a profile is shown in an error.

use std::fmt::Write;

/// A name as an error shows it: each control character (U+0000 to U+001F and
/// U+007F) is written as `\u` and four uppercase hex digits, and everything
/// else stays as written. An error built from it never carries a raw control
/// character, an escape sequence included.
pub fn display_name(text: &str) -> String {
    let mut shown = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_control() && (u32::from(c) <= 0x1F || u32::from(c) == 0x7F) {
            // Writing to a String cannot fail.
            let _ = write!(shown, "\\u{:04X}", u32::from(c));
        } else {
            shown.push(c);
        }
    }
    shown
}
