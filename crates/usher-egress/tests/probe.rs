//! Reds for the send wiring and the plan's host check.

use std::io::{self, Read};

use ureq::http::{HeaderMap, HeaderValue};
use usher_core::{
    AllowAll, Class, Descriptor, Egress, Field, Method, Place, Port, Request, WireRequest,
};
use usher_egress::{answer_for, answer_from, plan_request};
use usher_testkit::RecordingTransport;

/// A finite reader of `n` bytes, so a read that ignores the limit ends with a wrong length
/// and fails an assertion instead of hanging.
fn bytes_of(n: usize) -> &'static [u8] {
    &[b'x'; 64][..n]
}

struct Failing(io::Error);

impl Read for Failing {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(self.0.kind(), "boom"))
    }
}

struct Timed;

impl Read for Timed {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other(ureq::Error::Timeout(
            ureq::Timeout::Global,
        )))
    }
}

struct Reset;

impl Read for Reset {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("connection reset"))
    }
}

#[test]
fn a_body_read_that_times_out_uses_the_whole_call_text() {
    let err = answer_from("h.usher.invalid", 200, &HeaderMap::new(), Timed, 16)
        .expect_err("the read timed out");
    assert_eq!(err.check(), "the host answers within the limit");
    assert_eq!(
        err.status(),
        "h.usher.invalid: no answer within 30 s (whole call)"
    );
}

#[test]
fn a_failed_read_names_the_host() {
    let err = answer_from("h.usher.invalid", 200, &HeaderMap::new(), Reset, 16)
        .expect_err("the read failed");
    assert!(err.status().starts_with("h.usher.invalid: "), "{err}");
    assert!(err.status().contains("connection reset"), "{err}");
}

#[test]
fn a_failed_read_with_another_error_kind_still_names_the_host() {
    let err = answer_from(
        "h.usher.invalid",
        200,
        &HeaderMap::new(),
        Failing(io::Error::from(io::ErrorKind::UnexpectedEof)),
        16,
    )
    .expect_err("the read failed");
    assert!(err.status().starts_with("h.usher.invalid: "), "{err}");
}

#[test]
fn the_location_header_is_carried() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "location",
        HeaderValue::from_static("https://assets.usher.invalid/x"),
    );
    let response = answer_from("h.usher.invalid", 302, &headers, &b""[..], 16).expect("a response");
    assert_eq!(response.status, 302);
    assert_eq!(
        response.location.as_deref(),
        Some("https://assets.usher.invalid/x")
    );
}

#[test]
fn no_location_header_gives_none() {
    let response =
        answer_from("h.usher.invalid", 200, &HeaderMap::new(), &b"ok"[..], 16).expect("a response");
    assert_eq!(response.location, None);
    assert_eq!(response.body, b"ok");
}

#[test]
fn the_body_is_read_at_most_one_byte_past_the_limit() {
    let response = answer_from("h.usher.invalid", 200, &HeaderMap::new(), bytes_of(20), 4)
        .expect("a response");
    assert_eq!(response.body.len(), 5);
}

#[test]
fn the_limit_applies_to_an_error_status_too() {
    let response = answer_from("h.usher.invalid", 503, &HeaderMap::new(), bytes_of(20), 4)
        .expect("a response");
    assert_eq!(response.status, 503);
    assert_eq!(response.body.len(), 5);
}

const UA: Field = Field {
    name: "user-agent",
    place: Place::Header,
    meaning: "the program name",
};

const WITH_USERINFO: Descriptor = Descriptor {
    name: "test-userinfo",
    method: Method::Get,
    host: "user:secret@files.usher.invalid",
    path: "/list",
    purpose: "a test call whose host carries a user name",
    class: Class::PersonStarted,
    fields: &[UA],
    redirect_hosts: &[],
    max_body_bytes: 1024,
};

#[test]
fn a_host_with_a_user_name_or_password_is_refused() {
    let transport = RecordingTransport::new().answering_ok("ok");
    Port::new(transport.clone(), AllowAll)
        .call_described(&WITH_USERINFO, Request::new(Egress::KnowledgeUpdate))
        .expect("the fake accepts the call");
    let wire: WireRequest = transport.sent().remove(0);
    let err = plan_request(&wire).expect_err("a host with userinfo is not planned");
    assert_eq!(err.step(), "send request");
    assert_eq!(err.check(), "the host is a bare host name");
    assert!(err.status().contains("user name or password"), "{err}");
    assert!(!err.status().contains("secret"), "{err}");
}

const SMALL: Descriptor = Descriptor {
    name: "test-small",
    method: Method::Get,
    host: "files.usher.invalid",
    path: "/list",
    purpose: "a test call with a small answer limit",
    class: Class::PersonStarted,
    fields: &[UA],
    redirect_hosts: &[],
    max_body_bytes: 4,
};

fn small_wire() -> WireRequest {
    let transport = RecordingTransport::new().answering_ok("ok");
    Port::new(transport.clone(), AllowAll)
        .call_described(&SMALL, Request::new(Egress::KnowledgeUpdate))
        .expect("the fake accepts the call");
    transport.sent().remove(0)
}

#[test]
fn the_answer_for_a_request_is_read_with_that_requests_limit() {
    let response =
        answer_for(&small_wire(), 200, &HeaderMap::new(), bytes_of(20)).expect("a response");
    assert_eq!(response.body.len(), 5);
}

#[test]
fn the_answer_for_a_request_names_that_requests_host_when_the_read_fails() {
    let err =
        answer_for(&small_wire(), 200, &HeaderMap::new(), Reset).expect_err("the read failed");
    assert!(err.status().starts_with("files.usher.invalid: "), "{err}");
}

#[test]
fn the_answer_for_a_request_carries_the_location_header() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "location",
        HeaderValue::from_static("https://assets.usher.invalid/x"),
    );
    let response = answer_for(&small_wire(), 302, &headers, &b""[..]).expect("a response");
    assert_eq!(
        response.location.as_deref(),
        Some("https://assets.usher.invalid/x")
    );
}
