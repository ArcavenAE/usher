//! The trusted-key list (`USHER_TRUSTED_KEYS`): its parser and the code it
//! generates. `build.rs` runs it at build time; a unit test includes the same
//! file, so a malformed key is shown to fail the validator the build script
//! calls.
//!
//! The format is `current:<base64>,next:<base64>`: exactly one `current` key,
//! at most one `next`, each a minisign public key as `minisign.pub` writes it
//! on its second line. An empty value is an empty list.

// `build.rs` calls `parse` and `render`; the test build of the crate calls
// `parse` only, so each side leaves something unused.
#![allow(dead_code)]

use minisign_verify::PublicKey;

/// Which slot a trusted key fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The key releases are signed with now.
    Current,
    /// The key that takes over at the next rotation.
    Next,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Current => "current",
            Role::Next => "next",
        }
    }
}

/// One validated entry of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub role: Role,
    pub key: String,
}

/// Parses and validates the list. An empty or blank value gives an empty list.
pub fn parse(list: &str) -> Result<Vec<Entry>, String> {
    if list.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<Entry> = Vec::new();
    for raw in list.split(',') {
        let item = raw.trim();
        let (role, key) = item.split_once(':').ok_or_else(|| {
            format!("entry `{item}` needs the form `current:<base64>` or `next:<base64>`")
        })?;
        let role = match role.trim() {
            "current" => Role::Current,
            "next" => Role::Next,
            other => {
                return Err(format!("entry role `{other}` must be `current` or `next`"));
            }
        };
        let key = key.trim();
        PublicKey::from_base64(key).map_err(|why| {
            format!(
                "the `{}` key is not a minisign public key: {why}",
                role.as_str()
            )
        })?;
        if entries.iter().any(|e| e.role == role) {
            return Err(format!("more than one `{}` key", role.as_str()));
        }
        if entries.iter().any(|e| e.key == key) {
            return Err(format!("the `{}` key repeats another entry", role.as_str()));
        }
        entries.push(Entry {
            role,
            key: key.to_string(),
        });
    }
    if !entries.iter().any(|e| e.role == Role::Current) {
        return Err("the list has a `next` key but no `current` key".to_string());
    }
    Ok(entries)
}

/// The rule for a release build: it must carry at least one trusted key and
/// must not enable the `test-keys` feature. Other profiles are not checked.
pub fn check_release(
    release: bool,
    test_keys_enabled: bool,
    entries: &[Entry],
) -> Result<(), String> {
    if release && entries.is_empty() {
        return Err(
            "a release build needs trusted keys: set USHER_TRUSTED_KEYS to `current:<base64>[,next:<base64>]`"
                .to_string(),
        );
    }
    if release && test_keys_enabled {
        return Err("a release build must not enable the `test-keys` feature".to_string());
    }
    Ok(())
}

/// The Rust source `build.rs` writes into `OUT_DIR`.
pub fn render(entries: &[Entry]) -> String {
    let mut out =
        String::from("/// Role and base64 minisign public key, from `USHER_TRUSTED_KEYS`.\n");
    out.push_str("pub(crate) const TRUSTED_KEYS: &[(&str, &str)] = &[\n");
    for entry in entries {
        out.push_str(&format!(
            "    ({:?}, {:?}),\n",
            entry.role.as_str(),
            entry.key
        ));
    }
    out.push_str("];\n");
    out
}

#[cfg(test)]
mod tests {
    use usher_testkit::Signer;

    use super::*;

    fn key() -> String {
        Signer::new().public_key_base64()
    }

    #[test]
    fn a_blank_value_is_an_empty_list() {
        assert_eq!(parse("").expect("empty"), Vec::new());
        assert_eq!(parse("  \n").expect("blank"), Vec::new());
    }

    #[test]
    fn current_and_next_parse_in_order() {
        let (current, next) = (key(), key());
        let list = parse(&format!("current:{current}, next:{next}")).expect("valid");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].role, Role::Current);
        assert_eq!(list[0].key, current);
        assert_eq!(list[1].role, Role::Next);
    }

    #[test]
    fn a_malformed_key_fails_the_validator_the_build_script_runs() {
        for bad in [
            "current:not-base64!",
            "current:Zm9vYmFy",
            "current:",
            "current:RWQ=",
        ] {
            let why = parse(bad).expect_err(bad);
            assert!(why.contains("not a minisign public key"), "{bad}: {why}");
        }
    }

    #[test]
    fn a_bad_shape_fails() {
        let good = key();
        assert!(
            parse(&good)
                .expect_err("no role")
                .contains("needs the form")
        );
        assert!(
            parse(&format!("older:{good}"))
                .expect_err("bad role")
                .contains("`current` or `next`")
        );
        assert!(
            parse(&format!("next:{good}"))
                .expect_err("no current")
                .contains("no `current` key")
        );
        assert!(
            parse(&format!("current:{good},current:{}", key()))
                .expect_err("two currents")
                .contains("more than one `current`")
        );
        assert!(
            parse(&format!("current:{good},next:{good}"))
                .expect_err("repeat")
                .contains("repeats")
        );
        assert!(parse(&format!("current:{good},")).is_err());
    }

    #[test]
    fn a_release_build_needs_keys_and_refuses_the_test_feature() {
        let keys = parse(&format!("current:{}", key())).expect("valid");
        assert!(check_release(true, false, &keys).is_ok());
        assert!(
            check_release(true, false, &[])
                .expect_err("empty list")
                .contains("needs trusted keys")
        );
        assert!(
            check_release(true, true, &keys)
                .expect_err("test keys")
                .contains("`test-keys`")
        );
        // Other profiles build without keys and may use the feature.
        assert!(check_release(false, false, &[]).is_ok());
        assert!(check_release(false, true, &keys).is_ok());
    }

    #[test]
    fn the_generated_source_names_each_key() {
        let current = key();
        let source = render(&parse(&format!("current:{current}")).expect("valid"));
        assert!(source.contains("TRUSTED_KEYS"));
        assert!(source.contains(&format!("(\"current\", \"{current}\")")));
        assert!(render(&[]).contains("&[\n];"));
    }
}
