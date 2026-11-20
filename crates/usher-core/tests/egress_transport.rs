//! Story 1.15, part B, the port's half: the answer limit travels with the wire
//! request, and a transport error is reported under the call's own step.

use usher_core::{
    Class, Descriptor, Egress, Field, Method, PersonStartedOnly, Place, Port, Request, UsherError,
    descriptor,
};
use usher_testkit::RecordingTransport;

const FILES_HOST: &str = "files.usher.invalid";
const ASSETS_HOST: &str = "assets.usher.invalid";

const UA: Field = Field {
    name: "user-agent",
    place: Place::Header,
    meaning: "the program name",
};

const LIMITED: Descriptor = Descriptor {
    name: "test-limited",
    method: Method::Get,
    host: FILES_HOST,
    path: "/{file}",
    purpose: "a test call with its own limit",
    class: Class::PersonStarted,
    fields: &[
        UA,
        Field {
            name: "file",
            place: Place::Path,
            meaning: "which file",
        },
    ],
    redirect_hosts: &[ASSETS_HOST],
    max_body_bytes: 2048,
};

fn port(transport: &RecordingTransport) -> Port<RecordingTransport, PersonStartedOnly> {
    Port::new(transport.clone(), PersonStartedOnly)
}

fn request() -> Request {
    Request::new(Egress::KnowledgeUpdate).with("file", "k.toml")
}

fn transport_failure() -> Result<usher_core::Response, UsherError> {
    Err(UsherError::new(
        "send request",
        "the host answers within the limit",
        "files.usher.invalid: no answer within 10 s (connect)",
        "check the network, then try again",
    ))
}

#[test]
fn the_wire_request_carries_the_descriptors_answer_limit() {
    let transport = RecordingTransport::new().answering_ok("body");
    port(&transport)
        .call_described(&LIMITED, request())
        .expect("a valid call");
    let sent = transport.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].max_response_bytes(), 2048);
}

#[test]
fn a_registered_call_carries_its_registered_limit() {
    let transport = RecordingTransport::new().answering_ok("body");
    port(&transport).call(request()).expect("a valid call");
    let limit = descriptor(Egress::KnowledgeUpdate).max_body_bytes;
    assert_ne!(limit, 0);
    assert_eq!(transport.sent()[0].max_response_bytes(), limit);
}

#[test]
fn the_followed_request_carries_the_same_limit() {
    let transport = RecordingTransport::new()
        .answering_redirect(format!("https://{ASSETS_HOST}/moved"))
        .answering_ok("body");
    port(&transport)
        .call_described(&LIMITED, request())
        .expect("a followed call");
    let sent = transport.sent();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[1].host(), ASSETS_HOST);
    assert_eq!(sent[1].max_response_bytes(), 2048);
}

#[test]
fn a_transport_error_on_the_first_send_is_reported_under_the_calls_step() {
    let transport = RecordingTransport::new().answering(transport_failure());
    let err = port(&transport)
        .call_described(&LIMITED, request())
        .expect_err("the transport failed");
    assert_eq!(err.step(), "send test-limited");
    assert_eq!(err.check(), "the host answers within the limit");
    assert_eq!(
        err.status(),
        "files.usher.invalid: no answer within 10 s (connect)"
    );
    assert_eq!(err.next_action(), "check the network, then try again");
}

#[test]
fn a_transport_error_on_the_followed_send_is_reported_under_the_calls_step() {
    let transport = RecordingTransport::new()
        .answering_redirect(format!("https://{ASSETS_HOST}/moved"))
        .answering(transport_failure());
    let err = port(&transport)
        .call_described(&LIMITED, request())
        .expect_err("the followed send failed");
    assert_eq!(err.step(), "send test-limited");
    assert_eq!(err.check(), "the host answers within the limit");
    assert_eq!(
        err.status(),
        "files.usher.invalid: no answer within 10 s (connect)"
    );
    assert_eq!(err.next_action(), "check the network, then try again");
    assert_eq!(transport.sent().len(), 2);
}

#[test]
fn a_registered_call_reports_a_transport_error_under_its_registered_name() {
    let transport = RecordingTransport::new().answering(transport_failure());
    let err = port(&transport)
        .call(request())
        .expect_err("the transport failed");
    assert_eq!(err.step(), "send knowledge-update");
}
