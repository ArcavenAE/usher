//! The first caller of the egress port: fetch a signed knowledge update.
//!
//! The file and its signature come through the port, and then
//! [`Knowledge::update`] decides, offline, whether to take them. Fetching adds
//! no trust of its own: a file that did not verify leaves the knowledge in use
//! unchanged.

use usher_core::{Policy, Port, Transport, UsherError};

use crate::{
    Knowledge, KnowledgeVersion, TrustedKeys,
    update::{KEEP_SIGNING, SIGNATURE_UTF8_CHECK, STEP},
};

/// Fetches the signed knowledge file and its signature through the port and
/// takes the update if it verifies.
pub fn fetch_update<T: Transport, P: Policy>(
    port: &Port<T, P>,
    knowledge: &mut Knowledge,
    trusted: &TrustedKeys,
) -> Result<KnowledgeVersion, UsherError> {
    let _ = (
        port,
        knowledge,
        trusted,
        STEP,
        SIGNATURE_UTF8_CHECK,
        KEEP_SIGNING,
    );
    Err(UsherError::new(
        "fetch harness knowledge",
        "the fetch is built",
        "not built",
        "build the knowledge fetch",
    ))
}

#[cfg(test)]
mod tests {
    use usher_core::{Class, Method, PersonStartedOnly, Policy, Port, Response, UsherError};
    use usher_testkit::{RecordingTransport, Signer};

    use super::*;

    const SEED: &str = include_str!("../knowledge/harness-knowledge.toml");
    const HOST: &str = "knowledge.usher.invalid";

    fn held() -> Knowledge {
        Knowledge::embedded().expect("embedded knowledge")
    }

    fn sequence(knowledge: &Knowledge) -> u64 {
        knowledge.current().version().sequence()
    }

    fn file_at(sequence: u64) -> String {
        let held = held();
        let base = self::sequence(&held);
        SEED.replace(
            &format!("\nsequence = {base}\n"),
            &format!("\nsequence = {sequence}\n"),
        )
    }

    fn trusting(signer: &Signer) -> TrustedKeys {
        TrustedKeys::for_tests(&signer.public_key_base64(), None).expect("keys")
    }

    fn port(transport: &RecordingTransport) -> Port<RecordingTransport, PersonStartedOnly> {
        Port::new(transport.clone(), PersonStartedOnly)
    }

    fn answers(file: &str, signature: &str) -> RecordingTransport {
        RecordingTransport::new()
            .answering_ok(file.as_bytes())
            .answering_ok(signature.as_bytes())
    }

    fn assert_four_fields(err: &UsherError) {
        assert!(!err.step().is_empty() && !err.check().is_empty(), "{err}");
        assert!(
            !err.status().is_empty() && !err.next_action().is_empty(),
            "{err}"
        );
    }

    /// A signed update at the next sequence goes through, so the failing cases
    /// below cannot pass merely because the fetch refuses everything.
    fn assert_a_valid_fetch_is_taken() {
        let signer = Signer::new();
        let next = sequence(&held()) + 1;
        let file = file_at(next);
        let transport = answers(&file, &signer.sign(file.as_bytes(), "control"));
        let mut knowledge = held();
        fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
            .expect("the control fetch is taken");
        assert_eq!(sequence(&knowledge), next);
    }

    #[test]
    fn a_signed_update_is_fetched_and_taken() {
        let signer = Signer::new();
        let next = sequence(&held()) + 1;
        let file = file_at(next);
        let transport = answers(&file, &signer.sign(file.as_bytes(), "sequence"));
        let mut knowledge = held();
        let version = fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
            .expect("a signed update is taken");
        assert_eq!(version.sequence(), next);
        assert_eq!(sequence(&knowledge), next);
    }

    #[test]
    fn the_file_and_then_its_signature_are_fetched_with_nothing_else_sent() {
        let signer = Signer::new();
        let file = file_at(sequence(&held()) + 1);
        let transport = answers(&file, &signer.sign(file.as_bytes(), "sequence"));
        let mut knowledge = held();
        fetch_update(&port(&transport), &mut knowledge, &trusting(&signer)).expect("taken");
        let sent = transport.sent();
        assert_eq!(
            sent.len(),
            2,
            "one request for the file, one for the signature"
        );
        assert_eq!(sent[0].path, "/harness-knowledge.toml");
        assert_eq!(sent[1].path, "/harness-knowledge.toml.minisig");
        for wire in &sent {
            assert_eq!(wire.host, HOST);
            assert_eq!(wire.method, Method::Get);
            assert!(wire.query.is_empty() && wire.body.is_empty());
            assert_eq!(
                wire.headers,
                vec![("user-agent".to_string(), "usher".to_string())]
            );
        }
    }

    #[test]
    fn a_tampered_body_is_refused_and_the_knowledge_stays() {
        assert_a_valid_fetch_is_taken();
        let signer = Signer::new();
        let file = file_at(sequence(&held()) + 1);
        let signature = signer.sign(file.as_bytes(), "sequence");
        let tampered = file.replace("goose", "g00se");
        let transport = answers(&tampered, &signature);
        let mut knowledge = held();
        let before = sequence(&knowledge);
        let err = fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
            .expect_err("a tampered body must fail");
        assert_four_fields(&err);
        assert!(err.to_string().contains("signature"), "{err}");
        assert_eq!(sequence(&knowledge), before);
    }

    #[test]
    fn a_lower_or_equal_sequence_is_refused_and_the_knowledge_stays() {
        assert_a_valid_fetch_is_taken();
        let signer = Signer::new();
        let before = sequence(&held());
        for lower in [before, before - 1] {
            let file = file_at(lower);
            let transport = answers(&file, &signer.sign(file.as_bytes(), "sequence"));
            let mut knowledge = held();
            let err = fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
                .expect_err("a sequence that is not higher must fail");
            assert_four_fields(&err);
            assert!(err.to_string().contains("sequence"), "{err}");
            assert_eq!(sequence(&knowledge), before, "sequence {lower}");
        }
    }

    #[test]
    fn a_signature_from_an_untrusted_key_is_refused_and_the_knowledge_stays() {
        assert_a_valid_fetch_is_taken();
        let (trusted, stranger) = (Signer::new(), Signer::new());
        let file = file_at(sequence(&held()) + 1);
        let transport = answers(&file, &stranger.sign(file.as_bytes(), "sequence"));
        let mut knowledge = held();
        let before = sequence(&knowledge);
        let err = fetch_update(&port(&transport), &mut knowledge, &trusting(&trusted))
            .expect_err("an untrusted key must fail");
        assert_four_fields(&err);
        assert_eq!(sequence(&knowledge), before);
    }

    #[test]
    fn the_update_error_comes_back_as_the_update_gave_it() {
        assert_a_valid_fetch_is_taken();
        let (trusted, stranger) = (Signer::new(), Signer::new());
        let file = file_at(sequence(&held()) + 1);
        let signature = stranger.sign(file.as_bytes(), "sequence");
        let direct = held()
            .update(file.as_bytes(), &signature, &trusting(&trusted))
            .expect_err("the update refuses it");
        let transport = answers(&file, &signature);
        let mut knowledge = held();
        let err = fetch_update(&port(&transport), &mut knowledge, &trusting(&trusted))
            .expect_err("the fetch refuses it");
        assert_eq!(err, direct, "the fetch rewrote the update's error");
    }

    #[test]
    fn a_redirect_to_an_unlisted_host_on_the_file_stops_the_fetch_after_one_send() {
        assert_a_valid_fetch_is_taken();
        let signer = Signer::new();
        let transport = RecordingTransport::new()
            .answering_redirect("https://elsewhere.invalid/knowledge.toml");
        let mut knowledge = held();
        let before = sequence(&knowledge);
        let err = fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
            .expect_err("an unlisted redirect must fail");
        assert_four_fields(&err);
        assert!(err.to_string().contains("elsewhere.invalid"), "{err}");
        assert_eq!(
            transport.sent().len(),
            1,
            "the signature was asked for anyway"
        );
        assert_eq!(sequence(&knowledge), before);
    }

    #[test]
    fn a_failed_file_fetch_asks_for_no_signature() {
        assert_a_valid_fetch_is_taken();
        let signer = Signer::new();
        let transport = RecordingTransport::new().answering(Ok(Response {
            status: 404,
            location: None,
            body: Vec::new(),
        }));
        let mut knowledge = held();
        let before = sequence(&knowledge);
        let err = fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
            .expect_err("a 404 must fail");
        assert!(err.to_string().contains("404"), "{err}");
        assert_eq!(transport.sent().len(), 1);
        assert_eq!(sequence(&knowledge), before);
    }

    #[test]
    fn a_failed_signature_fetch_leaves_the_knowledge_and_names_the_failure() {
        assert_a_valid_fetch_is_taken();
        let signer = Signer::new();
        let file = file_at(sequence(&held()) + 1);
        let failure = UsherError::new("send", "the host answers", "connection reset", "try later");
        let transport = RecordingTransport::new()
            .answering_ok(file.as_bytes())
            .answering(Err(failure.clone()));
        let mut knowledge = held();
        let before = sequence(&knowledge);
        let err = fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
            .expect_err("a failed signature fetch must fail");
        assert_eq!(err, failure);
        assert_eq!(sequence(&knowledge), before);
    }

    #[test]
    fn a_signature_that_is_not_text_is_refused_in_the_updates_own_words_and_the_knowledge_stays() {
        assert_a_valid_fetch_is_taken();
        let signer = Signer::new();
        let file = file_at(sequence(&held()) + 1);
        let transport = RecordingTransport::new()
            .answering_ok(file.as_bytes())
            .answering_ok(vec![0xff, 0xfe, 0x00]);
        let mut knowledge = held();
        let before = sequence(&knowledge);
        let err = fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
            .expect_err("a binary signature must fail");
        assert_eq!(err.step(), "update harness knowledge");
        assert_eq!(err.check(), "read the signature as UTF-8 text");
        assert_eq!(
            err.next_action(),
            "usher keeps using the knowledge it has; obtain the update again, signed with minisign's default mode by a trusted key"
        );
        assert!(!err.status().is_empty(), "{err}");
        assert_eq!(transport.sent().len(), 2);
        assert_eq!(sequence(&knowledge), before);
    }

    struct Closed;
    impl Policy for Closed {
        fn allows(&self, _class: Class) -> bool {
            false
        }
    }

    #[test]
    fn a_closed_policy_sends_nothing_and_leaves_the_knowledge() {
        assert_a_valid_fetch_is_taken();
        let signer = Signer::new();
        let file = file_at(sequence(&held()) + 1);
        let transport = answers(&file, &signer.sign(file.as_bytes(), "sequence"));
        let closed = Port::new(transport.clone(), Closed);
        let mut knowledge = held();
        let before = sequence(&knowledge);
        let err = fetch_update(&closed, &mut knowledge, &trusting(&signer))
            .expect_err("a closed policy must fail");
        assert_four_fields(&err);
        assert!(transport.sent().is_empty());
        assert_eq!(sequence(&knowledge), before);
    }

    const LIMIT: usize = 1_048_576;

    /// A valid update file padded with a comment to exactly `len` bytes.
    fn padded_to(len: usize) -> String {
        let mut file = file_at(sequence(&held()) + 1);
        if !file.ends_with('\n') {
            file.push('\n');
        }
        let pad = len - file.len();
        file.push('#');
        file.push_str(&"x".repeat(pad - 2));
        file.push('\n');
        assert_eq!(file.len(), len);
        file
    }

    #[test]
    fn a_file_of_exactly_the_limit_is_passed_to_the_update_and_taken() {
        let signer = Signer::new();
        let file = padded_to(LIMIT);
        let transport = answers(&file, &signer.sign(file.as_bytes(), "sequence"));
        let mut knowledge = held();
        fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
            .expect("a file of exactly the limit is taken");
        assert_eq!(sequence(&knowledge), sequence(&held()) + 1);
    }

    #[test]
    fn a_file_one_byte_over_the_limit_is_refused_with_no_signature_request() {
        assert_a_valid_fetch_is_taken();
        let signer = Signer::new();
        let file = padded_to(LIMIT + 1);
        let transport = answers(&file, &signer.sign(file.as_bytes(), "sequence"));
        let mut knowledge = held();
        let before = sequence(&knowledge);
        let err = fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
            .expect_err("a file over the limit must fail");
        assert_four_fields(&err);
        assert!(
            err.to_string().contains("answer larger than the limit"),
            "{err}"
        );
        assert_eq!(transport.sent().len(), 1, "the signature was asked for");
        assert_eq!(sequence(&knowledge), before);
    }

    #[test]
    fn a_failed_fetch_leaves_a_knowledge_already_updated_at_its_own_sequence() {
        let signer = Signer::new();
        let base = sequence(&held());
        let first = file_at(base + 1);
        let transport = answers(&first, &signer.sign(first.as_bytes(), "sequence"));
        let mut knowledge = held();
        fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
            .expect("the first update is taken");
        assert_eq!(sequence(&knowledge), base + 1);

        let second = file_at(base + 2);
        let signature = signer.sign(second.as_bytes(), "sequence");
        let tampered = second.replace("goose", "g00se");
        let transport = answers(&tampered, &signature);
        fetch_update(&port(&transport), &mut knowledge, &trusting(&signer))
            .expect_err("a tampered second update must fail");
        assert_eq!(
            sequence(&knowledge),
            base + 1,
            "a failed fetch must keep the knowledge already in force, not reset it"
        );
    }
}
