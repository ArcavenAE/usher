//! A throwaway minisign signer for tests (Story 1.4).
//!
//! Every [`Signer`] is a fresh ed25519 key pair made at run time, so no key,
//! public or private, is stored in the repo. It writes real minisign files:
//! the default prehashed (`ED`) signature, and on request the legacy (`Ed`)
//! one, so a test can show the verifier refuses it.
//!
//! The seed comes from std (clock, process id, a counter and the hasher's
//! random state). A test key needs to be unique, not secret.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use blake2::{Blake2b512, Digest};
use ed25519_dalek::{Signer as _, SigningKey};

static COUNTER: AtomicU64 = AtomicU64::new(0);

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding, as minisign writes its key and signature lines.
fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for shift in [18, 12, 6, 0] {
            out.push(char::from(ALPHABET[((n >> shift) & 63) as usize]));
        }
        let pad = 3 - chunk.len();
        for _ in 0..pad {
            out.pop();
        }
        for _ in 0..pad {
            out.push('=');
        }
    }
    out
}

fn fresh_bytes<const N: usize>() -> [u8; N] {
    let mut out = [0u8; N];
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    for (block, slot) in out.chunks_mut(8).enumerate() {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u128(nanos);
        hasher.write_u32(std::process::id());
        hasher.write_u64(count);
        hasher.write_usize(block);
        let word = hasher.finish().to_le_bytes();
        slot.copy_from_slice(&word[..slot.len()]);
    }
    out
}

/// A throwaway minisign key pair.
pub struct Signer {
    signing: SigningKey,
    key_id: [u8; 8],
}

impl Default for Signer {
    fn default() -> Self {
        Self::new()
    }
}

impl Signer {
    /// Makes a new key pair with a fresh key id.
    pub fn new() -> Self {
        Self {
            signing: SigningKey::from_bytes(&fresh_bytes::<32>()),
            key_id: fresh_bytes::<8>(),
        }
    }

    /// The public key as minisign writes it: base64 of `Ed`, the key id and the key.
    pub fn public_key_base64(&self) -> String {
        let mut bin = Vec::with_capacity(42);
        bin.extend_from_slice(b"Ed");
        bin.extend_from_slice(&self.key_id);
        bin.extend_from_slice(self.signing.verifying_key().as_bytes());
        base64(&bin)
    }

    /// The key id as minisign prints it: the eight id bytes reversed, as hex.
    pub fn key_id_hex(&self) -> String {
        self.key_id
            .iter()
            .rev()
            .map(|b| format!("{b:02X}"))
            .collect()
    }

    /// A prehashed (`ED`) signature file over `message`, the minisign default.
    pub fn sign(&self, message: &[u8], trusted_comment: &str) -> String {
        let digest = Blake2b512::digest(message);
        self.signature_file(b"ED", &digest, trusted_comment)
    }

    /// A legacy (`Ed`) signature file over `message`, which production must refuse.
    pub fn sign_legacy(&self, message: &[u8], trusted_comment: &str) -> String {
        self.signature_file(b"Ed", message, trusted_comment)
    }

    fn signature_file(&self, algorithm: &[u8; 2], signed: &[u8], trusted_comment: &str) -> String {
        let signature = self.signing.sign(signed).to_bytes();
        let mut global_input = signature.to_vec();
        global_input.extend_from_slice(trusted_comment.as_bytes());
        let global = self.signing.sign(&global_input).to_bytes();
        let mut line = Vec::with_capacity(74);
        line.extend_from_slice(algorithm);
        line.extend_from_slice(&self.key_id);
        line.extend_from_slice(&signature);
        format!(
            "untrusted comment: signature from a usher test key\n{}\ntrusted comment: {}\n{}\n",
            base64(&line),
            trusted_comment,
            base64(&global),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn two_signers_differ() {
        assert_ne!(
            Signer::new().public_key_base64(),
            Signer::new().public_key_base64()
        );
    }

    #[test]
    fn a_key_id_is_sixteen_hex_digits() {
        let id = Signer::new().key_id_hex();
        assert_eq!(id.len(), 16);
        assert!(id.bytes().all(|b| b.is_ascii_hexdigit()));
    }

    #[test]
    fn a_public_key_is_forty_two_bytes_of_base64() {
        assert_eq!(Signer::new().public_key_base64().len(), 56);
    }

    #[test]
    fn a_signature_file_has_four_lines_and_the_trusted_comment() {
        let file = Signer::new().sign(b"hello", "sequence 2");
        let lines: Vec<&str> = file.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[2], "trusted comment: sequence 2");
    }
}
