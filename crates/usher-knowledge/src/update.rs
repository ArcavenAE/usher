//! Signed knowledge updates (AD-12, FR-31).
//!
//! An update is taken only when, in this order: its minisign signature
//! verifies against a trusted key over the exact bytes received; the bytes
//! parse and pass the structural check; and its sequence number is higher than
//! the one in force. Any failure leaves the knowledge in use unchanged and
//! says why. Nothing here fetches or writes: the caller supplies the bytes.

use minisign_verify::{Error as VerifyError, Signature};
use usher_core::UsherError;

use crate::{
    schema::{KnowledgeFile, KnowledgeVersion},
    trust::{TrustedKeys, key_id_of_signature},
};

const STEP: &str = "update harness knowledge";
const KEEP_SIGNING: &str = "usher keeps using the knowledge it has; obtain the update again, signed with minisign's default mode by a trusted key";
const KEEP_FILE: &str = "usher keeps using the knowledge it has; publish a corrected knowledge file with a higher sequence number and sign it again";

fn fail(
    check: impl Into<String>,
    status: impl Into<String>,
    next: impl Into<String>,
) -> UsherError {
    UsherError::new(STEP, check, status, next)
}

/// The harness knowledge in force: the embedded file, or the last good update.
#[derive(Debug, Clone)]
pub struct Knowledge {
    current: KnowledgeFile,
}

impl Knowledge {
    /// Starts from the knowledge file built into this release.
    pub fn embedded() -> Result<Self, UsherError> {
        Ok(Self {
            current: KnowledgeFile::embedded()?,
        })
    }

    /// The knowledge in force.
    pub fn current(&self) -> &KnowledgeFile {
        &self.current
    }

    /// Takes `file` as the new knowledge if `signature_file` (a minisign
    /// signature) verifies under `trusted` and its sequence is higher.
    ///
    /// On any error `self` is unchanged. On success returns the new version.
    pub fn update(
        &mut self,
        file: &[u8],
        signature_file: &str,
        trusted: &TrustedKeys,
    ) -> Result<KnowledgeVersion, UsherError> {
        verify_signature(file, signature_file, trusted)?;
        let text = std::str::from_utf8(file)
            .map_err(|why| fail("read the update as UTF-8 text", why.to_string(), KEEP_FILE))?;
        let update = KnowledgeFile::parse(text)?;
        let (new, held) = (
            update.version().sequence(),
            self.current.version().sequence(),
        );
        if new <= held {
            return Err(fail(
                "check the update's sequence number",
                format!("sequence {new} is not higher than {held}, the one in force"),
                KEEP_FILE,
            ));
        }
        let version = update.version();
        self.current = update;
        Ok(version)
    }
}

fn verify_signature(
    file: &[u8],
    signature_file: &str,
    trusted: &TrustedKeys,
) -> Result<(), UsherError> {
    const CHECK: &str = "verify the update's minisign signature";
    let signature = Signature::decode(signature_file).map_err(|why| {
        fail(
            CHECK,
            format!("the signature file is not a minisign signature: {why}"),
            KEEP_SIGNING,
        )
    })?;
    if trusted.is_empty() {
        return Err(fail(
            CHECK,
            "this build trusts no keys",
            "rebuild usher with USHER_TRUSTED_KEYS set, or keep the knowledge built into this release",
        ));
    }
    let mut key_matched_but_failed = None;
    for trusted_key in trusted.keys() {
        // Prehashed only: `allow_legacy` is never set in production.
        match trusted_key.key.verify(file, &signature, false) {
            Ok(()) => return Ok(()),
            Err(VerifyError::UnexpectedKeyId) => {}
            Err(why) => key_matched_but_failed = Some((trusted_key.role, why)),
        }
    }
    match key_matched_but_failed {
        Some((role, VerifyError::UnexpectedAlgorithm)) => Err(fail(
            CHECK,
            format!("the `{role}` key signed in legacy (Ed) mode, which usher does not accept"),
            KEEP_SIGNING,
        )),
        Some((role, why)) => Err(fail(
            CHECK,
            format!("the signature does not match the file under the `{role}` key: {why}"),
            KEEP_SIGNING,
        )),
        None => Err(fail(
            CHECK,
            format!(
                "signed by key {}, which is not in the trusted list",
                key_id_of_signature(signature_file)
            ),
            KEEP_SIGNING,
        )),
    }
}

#[cfg(test)]
mod tests {
    use usher_testkit::Signer;

    use super::*;

    const SEED: &str = include_str!("../knowledge/harness-knowledge.toml");

    fn with_sequence(sequence: u64) -> String {
        SEED.replace("\nsequence = 5\n", &format!("\nsequence = {sequence}\n"))
    }

    /// The held file for these tests: the embedded content at sequence 1, so
    /// the update cases keep their numbers when the embedded sequence moves.
    fn held() -> Knowledge {
        Knowledge {
            current: KnowledgeFile::parse(&with_sequence(1)).expect("embedded content"),
        }
    }

    fn sequence(knowledge: &Knowledge) -> u64 {
        knowledge.current().version().sequence()
    }

    fn trusting(current: &Signer, next: Option<&Signer>) -> TrustedKeys {
        TrustedKeys::for_tests(
            &current.public_key_base64(),
            next.map(Signer::public_key_base64).as_deref(),
        )
        .expect("keys")
    }

    #[test]
    fn a_valid_update_is_taken_and_its_version_reported() {
        let signer = Signer::new();
        let file = with_sequence(2);
        let mut knowledge = held();
        let version = knowledge
            .update(
                file.as_bytes(),
                &signer.sign(file.as_bytes(), "sequence 2"),
                &trusting(&signer, None),
            )
            .expect("accepted");
        assert_eq!(version.to_string(), "knowledge v2 (schema 5)");
        assert_eq!(sequence(&knowledge), 2);
    }

    #[test]
    fn a_list_read_the_way_the_build_script_writes_it_accepts_an_update() {
        let (current, next) = (Signer::new(), Signer::new());
        let (current_key, next_key) = (current.public_key_base64(), next.public_key_base64());
        let trusted = TrustedKeys::from_pairs(&[("current", &current_key), ("next", &next_key)])
            .expect("pairs");
        let file = with_sequence(2);
        let mut knowledge = held();
        knowledge
            .update(
                file.as_bytes(),
                &current.sign(file.as_bytes(), "sequence 2"),
                &trusted,
            )
            .expect("accepted");
        assert_eq!(sequence(&knowledge), 2);
    }

    #[test]
    fn a_next_key_signature_is_accepted() {
        let (current, next) = (Signer::new(), Signer::new());
        let file = with_sequence(2);
        let mut knowledge = held();
        knowledge
            .update(
                file.as_bytes(),
                &next.sign(file.as_bytes(), "sequence 2"),
                &trusting(&current, Some(&next)),
            )
            .expect("accepted");
        assert_eq!(sequence(&knowledge), 2);
    }

    #[test]
    fn a_changed_byte_is_rejected_and_the_old_knowledge_stays() {
        let signer = Signer::new();
        let file = with_sequence(2);
        let signature = signer.sign(file.as_bytes(), "sequence 2");
        let tampered = file.replace("goose", "g00se");
        let mut knowledge = held();
        let err = knowledge
            .update(tampered.as_bytes(), &signature, &trusting(&signer, None))
            .expect_err("bad signature");
        assert_eq!(err.check(), "verify the update's minisign signature");
        assert!(err.status().contains("does not match"), "{}", err.status());
        assert!(!err.next_action().is_empty());
        assert_eq!(sequence(&knowledge), 1);
    }

    #[test]
    fn a_key_outside_the_list_is_rejected_and_named() {
        let (trusted, stranger) = (Signer::new(), Signer::new());
        let file = with_sequence(2);
        let signature = stranger.sign(file.as_bytes(), "sequence 2");
        let mut knowledge = held();
        let err = knowledge
            .update(file.as_bytes(), &signature, &trusting(&trusted, None))
            .expect_err("unknown key");
        assert!(
            err.status().contains("not in the trusted list"),
            "{}",
            err.status()
        );
        assert!(
            err.status().contains(&stranger.key_id_hex()),
            "{}",
            err.status()
        );
        assert_eq!(sequence(&knowledge), 1);
    }

    #[test]
    fn a_dropped_key_is_revoked() {
        let (old, current) = (Signer::new(), Signer::new());
        let file = with_sequence(2);
        let mut knowledge = held();
        let err = knowledge
            .update(
                file.as_bytes(),
                &old.sign(file.as_bytes(), "sequence 2"),
                &trusting(&current, None),
            )
            .expect_err("revoked");
        assert!(err.status().contains("not in the trusted list"));
    }

    #[test]
    fn an_equal_or_lower_sequence_is_rejected() {
        let signer = Signer::new();
        let trusted = trusting(&signer, None);
        let mut knowledge = held();
        let three = with_sequence(3);
        knowledge
            .update(
                three.as_bytes(),
                &signer.sign(three.as_bytes(), "three"),
                &trusted,
            )
            .expect("accepted");
        for sequence_number in [3, 2] {
            let file = with_sequence(sequence_number);
            let err = knowledge
                .update(
                    file.as_bytes(),
                    &signer.sign(file.as_bytes(), "replay"),
                    &trusted,
                )
                .expect_err("not higher");
            assert_eq!(err.check(), "check the update's sequence number");
            assert!(
                err.status()
                    .contains(&format!("sequence {sequence_number} is not higher than 3")),
                "{}",
                err.status()
            );
            assert_eq!(sequence(&knowledge), 3);
        }
    }

    #[test]
    fn a_signed_file_that_breaks_the_schema_is_rejected() {
        let signer = Signer::new();
        let file = with_sequence(2).replace(
            "reference = { kind = \"vendor-docs\", locator = \"https://code.claude.com/docs/en/env-vars\" }\n",
            "",
        );
        let mut knowledge = held();
        let err = knowledge
            .update(
                file.as_bytes(),
                &signer.sign(file.as_bytes(), "sequence 2"),
                &trusting(&signer, None),
            )
            .expect_err("structural check");
        assert_ne!(err.check(), "verify the update's minisign signature");
        assert!(
            err.check().contains("claude-code.config-dir-variable"),
            "{}",
            err.check()
        );
        assert_eq!(sequence(&knowledge), 1);
    }

    #[test]
    fn unsigned_garbage_reports_the_signature_not_a_parse_error() {
        let signer = Signer::new();
        let garbage = b"\xff\xfe not toml [[[";
        let mut knowledge = held();
        let err = knowledge
            .update(
                garbage,
                &signer.sign(b"something else", "x"),
                &trusting(&signer, None),
            )
            .expect_err("bad signature");
        assert_eq!(err.check(), "verify the update's minisign signature");
        assert_eq!(sequence(&knowledge), 1);
    }

    #[test]
    fn a_legacy_ed_signature_is_rejected() {
        let signer = Signer::new();
        let file = with_sequence(2);
        let mut knowledge = held();
        let err = knowledge
            .update(
                file.as_bytes(),
                &signer.sign_legacy(file.as_bytes(), "sequence 2"),
                &trusting(&signer, None),
            )
            .expect_err("legacy");
        assert!(
            err.status().contains("legacy (Ed) mode"),
            "{}",
            err.status()
        );
        assert_eq!(sequence(&knowledge), 1);
    }

    #[test]
    fn a_malformed_signature_is_rejected() {
        let signer = Signer::new();
        let file = with_sequence(2);
        for bad in ["", "not a signature", "untrusted comment: x\nAAAA\n"] {
            let mut knowledge = held();
            let err = knowledge
                .update(file.as_bytes(), bad, &trusting(&signer, None))
                .expect_err(bad);
            assert!(
                err.status().contains("not a minisign signature"),
                "{bad}: {}",
                err.status()
            );
            assert_eq!(sequence(&knowledge), 1);
        }
    }

    #[test]
    fn a_build_with_no_trusted_keys_accepts_nothing() {
        let signer = Signer::new();
        let file = with_sequence(2);
        let mut knowledge = held();
        let err = knowledge
            .update(
                file.as_bytes(),
                &signer.sign(file.as_bytes(), "sequence 2"),
                &TrustedKeys::default(),
            )
            .expect_err("no keys");
        assert_eq!(err.status(), "this build trusts no keys");
        assert!(err.next_action().contains("USHER_TRUSTED_KEYS"));
        assert_eq!(sequence(&knowledge), 1);
    }
}
