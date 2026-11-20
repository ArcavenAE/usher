//! Story 1.15, part B: the one real transport. The client is planned, bounded and
//! configured by pure functions tested here without a network; the real connection is
//! tested only against a listener on this host that speaks no TLS.

use std::{
    io::{self, Read},
    net::{IpAddr, TcpListener},
    thread,
    time::Duration,
};

use usher_core::{
    ALL, AllowAll, Class, Descriptor, Egress, Field, Method, Place, Port, Request, UsherError,
    WireRequest, descriptor,
};
use usher_egress::{
    AgentSettings, CALL_TIMEOUT_SECS, CONNECT_TIMEOUT_SECS, HttpTransport, TimeoutKind,
    agent_settings, plan_request, read_limited, response_from, timeout_error,
};
use usher_testkit::RecordingTransport;

const UA: Field = Field {
    name: "user-agent",
    place: Place::Header,
    meaning: "the program name",
};

const fn field(name: &'static str, place: Place) -> Field {
    Field {
        name,
        place,
        meaning: "test",
    }
}

const QUERYING: Descriptor = Descriptor {
    name: "test-querying",
    method: Method::Get,
    host: "files.usher.invalid",
    path: "/list",
    purpose: "a test call with query fields",
    class: Class::PersonStarted,
    fields: &[
        UA,
        Field {
            name: "name",
            place: Place::Query,
            meaning: "test",
        },
        Field {
            name: "kind",
            place: Place::Query,
            meaning: "test",
        },
    ],
    redirect_hosts: &[],
    max_body_bytes: 1024,
};

const BODIED: Descriptor = Descriptor {
    name: "test-bodied",
    method: Method::Post,
    host: "files.usher.invalid",
    path: "/submit",
    purpose: "a test call that sends a body field",
    class: Class::PersonStarted,
    fields: &[UA, field("note", Place::Body)],
    redirect_hosts: &[],
    max_body_bytes: 1024,
};

const HEADERED: Descriptor = Descriptor {
    name: "test-headered",
    method: Method::Get,
    host: "files.usher.invalid",
    path: "/list",
    purpose: "a test call with several header fields",
    class: Class::PersonStarted,
    fields: &[
        UA,
        field("x-trace", Place::Header),
        field("accept", Place::Header),
    ],
    redirect_hosts: &[],
    max_body_bytes: 1024,
};

const POSTING: Descriptor = Descriptor {
    name: "test-posting",
    method: Method::Post,
    host: "files.usher.invalid",
    path: "/ping",
    purpose: "a test call that posts with no body field",
    class: Class::PersonStarted,
    fields: &[UA],
    redirect_hosts: &[],
    max_body_bytes: 1024,
};

fn recorded_wire(descriptor: &Descriptor, request: Request) -> WireRequest {
    let transport = RecordingTransport::new().answering_ok("ok");
    Port::new(transport.clone(), AllowAll)
        .call_described(descriptor, request)
        .expect("the fake accepts a valid call");
    transport.sent().remove(0)
}

fn assert_four_fields(err: &UsherError) {
    assert!(!err.step().is_empty(), "step is empty: {err}");
    assert!(!err.check().is_empty(), "check is empty: {err}");
    assert!(!err.status().is_empty(), "status is empty: {err}");
    assert!(!err.next_action().is_empty(), "next is empty: {err}");
}

/// A reader that never ends, so a read that is not bounded never returns.
struct Endless;

impl Read for Endless {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        buf.fill(b'x');
        Ok(buf.len())
    }
}

struct Broken;

impl Read for Broken {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("connection reset"))
    }
}

// ---- configuration ----

#[test]
fn the_time_limits_are_ten_seconds_to_connect_and_thirty_for_a_whole_call() {
    assert_eq!(CONNECT_TIMEOUT_SECS, 10);
    assert_eq!(CALL_TIMEOUT_SECS, 30);
}

#[test]
fn the_agent_follows_no_redirect_refuses_http_and_adds_nothing_of_its_own() {
    assert_eq!(
        agent_settings(),
        AgentSettings {
            connect_timeout_secs: 10,
            call_timeout_secs: 30,
            max_redirects: 0,
            https_only: true,
            http_status_as_error: false,
            gzip: false,
            cookies: false,
        }
    );
}

#[test]
fn a_connect_timeout_reads_as_the_ruled_text() {
    let err = timeout_error("files.usher.invalid", TimeoutKind::Connect);
    assert_eq!(err.step(), "send request");
    assert_eq!(err.check(), "the host answers within the limit");
    assert_eq!(
        err.status(),
        "files.usher.invalid: no answer within 10 s (connect)"
    );
    assert_four_fields(&err);
}

#[test]
fn a_whole_call_timeout_reads_as_the_ruled_text() {
    let err = timeout_error("files.usher.invalid", TimeoutKind::Call);
    assert_eq!(err.step(), "send request");
    assert_eq!(err.check(), "the host answers within the limit");
    assert_eq!(
        err.status(),
        "files.usher.invalid: no answer within 30 s (whole call)"
    );
    assert_four_fields(&err);
}

// ---- the planned request ----

#[test]
fn a_registered_call_plans_an_https_get_to_its_host_and_path() {
    let wire = recorded_wire(
        &QUERYING,
        Request::new(Egress::KnowledgeUpdate)
            .with("name", "a b&c")
            .with("kind", "x"),
    );
    let plan = plan_request(&wire).expect("a plan");
    assert_eq!(plan.method, Method::Get);
    assert_eq!(
        plan.uri,
        "https://files.usher.invalid/list?name=a%20b%26c&kind=x"
    );
}

#[test]
fn a_call_with_no_query_plans_no_question_mark() {
    let transport = RecordingTransport::new().answering_ok("ok");
    Port::new(transport.clone(), usher_core::PersonStartedOnly)
        .call(Request::new(Egress::KnowledgeUpdate).with("file", "harness-knowledge.toml"))
        .expect("a valid call");
    let plan = plan_request(&transport.sent()[0]).expect("a plan");
    assert_eq!(
        plan.uri,
        "https://knowledge.usher.invalid/harness-knowledge.toml"
    );
    assert!(!plan.uri.contains('?'), "{}", plan.uri);
}

#[test]
fn the_plan_carries_exactly_the_ports_headers_in_order() {
    let wire = recorded_wire(
        &QUERYING,
        Request::new(Egress::KnowledgeUpdate).with("name", "a"),
    );
    let plan = plan_request(&wire).expect("a plan");
    assert_eq!(plan.headers, wire.headers());
    assert_eq!(
        plan.headers,
        [("user-agent".to_string(), "usher".to_string())]
    );
}

#[test]
fn the_plan_keeps_several_headers_in_the_order_the_descriptor_lists_them() {
    let wire = recorded_wire(
        &HEADERED,
        Request::new(Egress::KnowledgeUpdate)
            .with("x-trace", "t1")
            .with("accept", "text/plain"),
    );
    let plan = plan_request(&wire).expect("a plan");
    assert_eq!(
        plan.headers,
        [
            ("user-agent".to_string(), "usher".to_string()),
            ("x-trace".to_string(), "t1".to_string()),
            ("accept".to_string(), "text/plain".to_string()),
        ],
        "neither sorted nor reversed"
    );
}

#[test]
fn a_post_with_no_body_field_plans_the_post_method_and_is_not_refused() {
    let wire = recorded_wire(&POSTING, Request::new(Egress::KnowledgeUpdate));
    assert!(
        wire.body().is_empty(),
        "the control request carries no body"
    );
    let plan = plan_request(&wire).expect("a bodyless post is not a body refusal");
    assert_eq!(plan.method, Method::Post);
    assert_eq!(plan.uri, "https://files.usher.invalid/ping");
}

#[test]
fn no_registered_host_names_a_port_or_is_an_address() {
    assert!(!ALL.is_empty(), "ALL names no call");
    for egress in ALL {
        let host = descriptor(*egress).host;
        assert!(
            !host.contains(':'),
            "a registered host names a port: {host}"
        );
        assert!(
            host.parse::<IpAddr>().is_err(),
            "a registered host is an address: {host}"
        );
    }
}

#[test]
fn a_plan_is_never_an_http_address() {
    let wire = recorded_wire(
        &QUERYING,
        Request::new(Egress::KnowledgeUpdate).with("name", "a"),
    );
    let plan = plan_request(&wire).expect("a plan");
    assert!(plan.uri.starts_with("https://"), "{}", plan.uri);
    assert!(!plan.uri.contains("http://"), "{}", plan.uri);
}

#[test]
fn a_request_with_a_body_field_is_refused_in_the_transports_own_words() {
    let wire = recorded_wire(
        &BODIED,
        Request::new(Egress::KnowledgeUpdate).with("note", "hello"),
    );
    assert!(
        !wire.body().is_empty(),
        "the control request carries a body"
    );
    let err = plan_request(&wire).expect_err("no registered call sends a body");
    assert_eq!(err.step(), "send request");
    assert_eq!(err.check(), "the call sends no body");
    assert_eq!(
        err.status(),
        "files.usher.invalid: no registered call sends a body"
    );
    assert_four_fields(&err);
}

// ---- bounded reads ----

#[test]
fn a_read_stops_one_byte_past_the_limit_even_on_an_endless_answer() {
    assert_eq!(read_limited(Endless, 10).expect("a read").len(), 11);
}

#[test]
fn an_answer_of_exactly_the_limit_is_read_whole() {
    let body = vec![b'y'; 10];
    assert_eq!(read_limited(&body[..], 10).expect("a read"), body);
}

#[test]
fn a_shorter_answer_is_read_whole() {
    assert_eq!(read_limited(&b"abc"[..], 10).expect("a read"), b"abc");
}

#[test]
fn a_limit_of_zero_still_reads_one_byte_so_the_port_can_see_an_over_limit_answer() {
    assert_eq!(read_limited(Endless, 0).expect("a read").len(), 1);
}

#[test]
fn a_redirect_answer_is_bounded_like_any_other() {
    let response =
        response_from(302, Some("https://assets.usher.invalid/x"), Endless, 4).expect("a response");
    assert_eq!(response.status, 302);
    assert_eq!(
        response.location.as_deref(),
        Some("https://assets.usher.invalid/x")
    );
    assert_eq!(response.body.len(), 5);
}

#[test]
fn a_success_answer_keeps_its_status_and_its_bounded_body() {
    let response = response_from(200, None, Endless, 4).expect("a response");
    assert_eq!(response.status, 200);
    assert_eq!(response.location, None);
    assert_eq!(response.body.len(), 5);
}

#[test]
fn a_failed_read_is_an_error_naming_the_cause() {
    let err = response_from(200, None, Broken, 4).expect_err("the read failed");
    assert_eq!(err.step(), "send request");
    assert_eq!(err.check(), "the answer is readable");
    assert!(err.status().contains("connection reset"), "{err}");
    assert_four_fields(&err);
}

// ---- the real connection, to a listener on this host ----

fn leaked_host(port: u16) -> &'static str {
    Box::leak(format!("127.0.0.1:{port}").into_boxed_str())
}

fn local_descriptor(host: &'static str, name: &'static str) -> Descriptor {
    Descriptor {
        name,
        method: Method::Get,
        host,
        path: "/{file}",
        purpose: "a test call to a listener on this host",
        class: Class::PersonStarted,
        fields: Box::leak(Box::new([UA, field("file", Place::Path)])),
        redirect_hosts: &[],
        max_body_bytes: 1024,
    }
}

#[test]
fn a_listener_that_speaks_no_tls_sees_a_tls_handshake_and_the_error_is_the_transports_own() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    listener.set_nonblocking(true).expect("nonblocking");
    // Waits up to five seconds for the one connection, so a client that never dials
    // fails the test instead of hanging it. It reads the client's first bytes and
    // writes nothing back, so no HTTP status can ever come back from it.
    let server = thread::spawn(move || {
        for _ in 0..50 {
            if let Ok((mut stream, _)) = listener.accept() {
                stream
                    .set_nonblocking(false)
                    .expect("a blocking stream for the read");
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .expect("a read timeout");
                let mut first = [0u8; 2];
                let read = stream.read(&mut first).unwrap_or(0);
                return Some(first[..read].to_vec());
            }
            thread::sleep(Duration::from_millis(100));
        }
        None
    });
    let host = leaked_host(port);
    let descriptor = local_descriptor(host, "test-local");
    let result = Port::new(HttpTransport::new(), AllowAll).call_described(
        &descriptor,
        Request::new(Egress::KnowledgeUpdate).with("file", "k.toml"),
    );
    let first = server.join().expect("the listener thread");
    let first = first.expect("the client never dialed the listener");
    assert!(
        first.len() >= 2 && first[0] == 0x16 && first[1] == 0x03,
        "the client's first bytes are a TLS handshake record (0x16, 0x03), not plain HTTP: {first:02x?}"
    );
    let err = result.expect_err("a handshake with a listener that speaks no TLS must fail");
    assert_eq!(err.step(), "send test-local");
    assert!(
        err.status().starts_with(&format!("{host}: ")),
        "the transport's own status names the host first: {err}"
    );
    assert!(
        !err.status().contains("answered status"),
        "a port refusal of a status is not a TLS error: {err}"
    );
    assert_ne!(
        err.check(),
        "the host answers with a success status",
        "the port's own refusal check is not the transport's: {err}"
    );
    assert_four_fields(&err);
}

#[test]
fn a_request_with_a_body_field_makes_no_connection_at_all() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.set_nonblocking(true).expect("nonblocking");
    let port = listener.local_addr().expect("addr").port();
    let host = leaked_host(port);
    let descriptor = Descriptor { host, ..BODIED };
    let err = Port::new(HttpTransport::new(), AllowAll)
        .call_described(
            &descriptor,
            Request::new(Egress::KnowledgeUpdate).with("note", "hello"),
        )
        .expect_err("a body is refused");
    assert_eq!(err.step(), "send test-bodied");
    assert_eq!(err.check(), "the call sends no body");
    assert_eq!(
        err.status(),
        format!("{host}: no registered call sends a body")
    );
    let accepted = listener.accept();
    assert!(
        matches!(&accepted, Err(e) if e.kind() == io::ErrorKind::WouldBlock),
        "the transport connected before it refused: {accepted:?}"
    );
}
