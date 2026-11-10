//! The keys usher trusts to sign a knowledge update (AD-12).
//!
//! The list is built into the binary by `build.rs` from `USHER_TRUSTED_KEYS`,
//! with a `current` key and optionally a `next` one. A key that a release
//! drops is revoked. No key is stored in this repo; tests make a throwaway key
//! pair and trust its public half through [`TrustedKeys::for_tests`].

use minisign_verify::PublicKey;
use usher_core::UsherError;

include!(concat!(env!("OUT_DIR"), "/trusted_keys.rs"));

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Decodes standard padded base64, or `None` on any other input.
fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let text = text.trim();
    if !text.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let bytes = text.as_bytes();
    let last = bytes.len() / 4;
    for (index, chunk) in bytes.chunks(4).enumerate() {
        let pad = chunk.iter().rev().take_while(|b| **b == b'=').count();
        if pad > 2 || (pad > 0 && index + 1 != last) {
            return None;
        }
        let mut n = 0u32;
        for (i, b) in chunk.iter().enumerate() {
            let value = if i >= 4 - pad {
                0
            } else {
                u32::try_from(ALPHABET.iter().position(|a| a == b)?).ok()?
            };
            n = (n << 6) | value;
        }
        let triple = n.to_be_bytes();
        out.extend_from_slice(&triple[1..4 - pad]);
    }
    Some(out)
}

/// A key id as minisign prints it: the eight bytes in reverse file order, as hex.
pub(crate) fn key_id_hex(id: &[u8]) -> String {
    id.iter().rev().map(|b| format!("{b:02X}")).collect()
}

/// The key id on a minisign signature file (its second line), as minisign prints it.
pub(crate) fn key_id_of_signature(signature_file: &str) -> String {
    signature_file
        .lines()
        .nth(1)
        .and_then(base64_decode)
        .and_then(|bin| bin.get(2..10).map(key_id_hex))
        .unwrap_or_else(|| "unknown".to_string())
}

/// One trusted key: its slot and the decoded key.
#[derive(Debug, Clone)]
pub(crate) struct TrustedKey {
    pub(crate) role: &'static str,
    pub(crate) key: PublicKey,
}

/// The set of keys a knowledge update may be signed with.
#[derive(Debug, Clone, Default)]
pub struct TrustedKeys {
    keys: Vec<TrustedKey>,
}

fn untrusted(why: impl Into<String>) -> UsherError {
    UsherError::new(
        "load trusted keys",
        "the build-time trusted-key list",
        why,
        "rebuild with a valid USHER_TRUSTED_KEYS",
    )
}

impl TrustedKeys {
    /// The list built into this binary. It is empty in a build made without
    /// `USHER_TRUSTED_KEYS`, which then accepts no update.
    pub fn build_time() -> Result<Self, UsherError> {
        Self::from_pairs(TRUSTED_KEYS)
    }

    /// Reads role and base64 key pairs, the shape `build.rs` generates.
    pub(crate) fn from_pairs(pairs: &[(&str, &str)]) -> Result<Self, UsherError> {
        let mut keys = Vec::new();
        for (role, base64) in pairs {
            let role = match *role {
                "current" => "current",
                "next" => "next",
                other => return Err(untrusted(format!("`{other}` is not a key role"))),
            };
            let key = PublicKey::from_base64(base64)
                .map_err(|why| untrusted(format!("the `{role}` key is not valid: {why}")))?;
            keys.push(TrustedKey { role, key });
        }
        Ok(Self { keys })
    }

    /// Trusts a throwaway key, for tests only. A release build refuses the
    /// `test-keys` feature that exposes this.
    #[cfg(any(test, feature = "test-keys"))]
    pub fn for_tests(current: &str, next: Option<&str>) -> Result<Self, UsherError> {
        let mut keys = Vec::new();
        for (role, base64) in [Some(("current", current)), next.map(|n| ("next", n))]
            .into_iter()
            .flatten()
        {
            let key = PublicKey::from_base64(base64)
                .map_err(|why| untrusted(format!("the `{role}` key is not valid: {why}")))?;
            keys.push(TrustedKey { role, key });
        }
        Ok(Self { keys })
    }

    /// Whether no key is trusted.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub(crate) fn keys(&self) -> &[TrustedKey] {
        &self.keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips_known_vectors() {
        assert_eq!(base64_decode("Zm9vYmFy"), Some(b"foobar".to_vec()));
        assert_eq!(base64_decode("Zm8="), Some(b"fo".to_vec()));
        assert_eq!(base64_decode("Zg=="), Some(b"f".to_vec()));
        assert_eq!(base64_decode("Zm9"), None);
        assert_eq!(base64_decode("Zg==Zg=="), None);
        assert_eq!(base64_decode("Zg=A"), None);
        assert_eq!(base64_decode("Zm9*"), None);
    }

    #[test]
    fn a_generated_list_maps_to_trusted_keys_by_role() {
        let (current, next) = (
            usher_testkit::Signer::new().public_key_base64(),
            usher_testkit::Signer::new().public_key_base64(),
        );
        let keys = TrustedKeys::from_pairs(&[("current", &current), ("next", &next)])
            .expect("valid pairs");
        let roles: Vec<&str> = keys.keys().iter().map(|k| k.role).collect();
        assert_eq!(roles, ["current", "next"]);
        assert!(TrustedKeys::from_pairs(&[]).expect("empty").is_empty());
        assert!(TrustedKeys::from_pairs(&[("older", &current)]).is_err());
        assert!(TrustedKeys::from_pairs(&[("current", "Zm9vYmFy")]).is_err());
    }

    #[test]
    fn a_key_id_prints_the_way_minisign_prints_it() {
        assert_eq!(
            key_id_hex(&[0x1f, 0xe8, 0xb4, 0x42, 0x18, 0x0f, 0x62, 0xe7]),
            "E7620F1842B4E81F"
        );
    }

    #[test]
    fn a_build_without_the_variable_trusts_nothing() {
        // The test build sets no USHER_TRUSTED_KEYS (CI never does).
        if option_env!("USHER_TRUSTED_KEYS").is_none() {
            assert!(
                TrustedKeys::build_time()
                    .expect("empty list builds")
                    .is_empty()
            );
        }
    }
}
