//! Reds for the egress port (Story 1.15, part A): the registry, the descriptor
//! rules, the field and redirect checks, and the class policy, over a recording
//! transport that never touches a network.

use std::collections::BTreeSet;

use usher_core::{
    ALL, Class, DELEGATED_INSTALLERS, DELEGATED_NOTE, Descriptor, Egress, Method,
    PersonStartedOnly, Place, Policy, Port, Request, Response, UsherError, descriptor,
};
use usher_testkit::RecordingTransport;

const FILE_URL_HOST: &str = "knowledge.usher.invalid";

/// Names every variant with no wildcard, so a new variant fails to compile here
/// until [`ALL`] and these tests know it.
fn every_variant_named(egress: Egress) -> &'static str {
    match egress {
        Egress::KnowledgeUpdate => "knowledge-update",
    }
}

fn knowledge_request() -> Request {
    Request::new(Egress::KnowledgeUpdate).with("file", "knowledge.toml")
}

fn port(transport: &RecordingTransport) -> Port<RecordingTransport, PersonStartedOnly> {
    Port::new(transport.clone(), PersonStartedOnly)
}

/// A valid call goes through. Negative tests start here so they cannot pass
/// merely because the port refuses everything.
fn assert_a_valid_call_goes_through() {
    let transport = RecordingTransport::new().answering_ok("control");
    let response = port(&transport)
        .call(knowledge_request())
        .expect("the control call goes through");
    assert_eq!(response.body, b"control");
    assert_eq!(transport.sent().len(), 1);
}

fn assert_four_fields(err: &UsherError) {
    assert!(!err.step().is_empty(), "step is empty: {err}");
    assert!(!err.check().is_empty(), "check is empty: {err}");
    assert!(!err.status().is_empty(), "status is empty: {err}");
    assert!(!err.next_action().is_empty(), "next is empty: {err}");
}

fn placeholders(path: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = path;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else { break };
        found.push(&after[..close]);
        rest = &after[close + 1..];
    }
    found
}

// ---- the registry ----

#[test]
fn all_names_every_call_once() {
    let named: BTreeSet<&str> = ALL.iter().map(|e| every_variant_named(*e)).collect();
    assert_eq!(
        named,
        BTreeSet::from(["knowledge-update"]),
        "ALL must name every Egress variant"
    );
    assert_eq!(ALL.len(), named.len(), "ALL names a call twice");
}

#[test]
fn every_descriptor_has_an_https_host_a_path_and_a_purpose() {
    assert!(!ALL.is_empty(), "ALL names no call");
    for egress in ALL {
        let d: &Descriptor = descriptor(*egress);
        let name = every_variant_named(*egress);
        assert!(!d.host.is_empty(), "{name}: host is empty");
        assert!(
            !d.host.contains("://") && !d.host.contains('/') && !d.host.contains(':'),
            "{name}: host must be a bare name, not a URL or a port: {}",
            d.host
        );
        assert!(
            d.path.starts_with('/'),
            "{name}: path must start with /: {}",
            d.path
        );
        assert!(!d.purpose.is_empty(), "{name}: purpose is empty");
    }
}

#[test]
fn every_placeholder_in_a_path_is_a_listed_path_field() {
    assert!(!ALL.is_empty(), "ALL names no call");
    for egress in ALL {
        let d = descriptor(*egress);
        let name = every_variant_named(*egress);
        for holder in placeholders(d.path) {
            assert!(
                d.fields
                    .iter()
                    .any(|f| f.place == Place::Path && f.name == holder),
                "{name}: placeholder {{{holder}}} is not a listed path field"
            );
        }
        for field in d.fields.iter().filter(|f| f.place == Place::Path) {
            assert!(
                placeholders(d.path).contains(&field.name),
                "{name}: path field {} has no placeholder in {}",
                field.name,
                d.path
            );
        }
    }
}

#[test]
fn every_field_carries_a_name_and_a_one_line_meaning() {
    assert!(!ALL.is_empty(), "ALL names no call");
    for egress in ALL {
        let d = descriptor(*egress);
        let name = every_variant_named(*egress);
        assert!(!d.fields.is_empty(), "{name}: lists no fields");
        for field in d.fields {
            assert!(!field.name.is_empty(), "{name}: a field has no name");
            assert!(
                !field.meaning.is_empty() && !field.meaning.contains('\n'),
                "{name}: field {} needs a one-line meaning",
                field.name
            );
        }
    }
}

#[test]
fn every_call_lists_the_user_agent_header() {
    assert!(!ALL.is_empty(), "ALL names no call");
    for egress in ALL {
        let d = descriptor(*egress);
        assert!(
            d.fields
                .iter()
                .any(|f| f.place == Place::Header && f.name == "user-agent"),
            "{}: user-agent is not listed",
            every_variant_named(*egress)
        );
    }
}

#[test]
fn the_knowledge_update_is_a_person_started_read_of_the_reserved_host() {
    let d = descriptor(Egress::KnowledgeUpdate);
    assert_eq!(d.method, Method::Get);
    assert_eq!(d.host, FILE_URL_HOST);
    assert_eq!(d.class, Class::PersonStarted);
    assert_eq!(d.path, "/{file}");
    assert!(d.redirect_hosts.is_empty());
}

// ---- the policy ----

#[test]
fn the_policy_allows_person_started_and_refuses_the_other_classes() {
    assert!(PersonStartedOnly.allows(Class::PersonStarted));
    assert!(!PersonStartedOnly.allows(Class::VersionCheck));
    assert!(!PersonStartedOnly.allows(Class::Reporting));
}

struct DenyAll;
impl Policy for DenyAll {
    fn allows(&self, _class: Class) -> bool {
        false
    }
}

#[test]
fn a_refused_class_never_reaches_the_transport() {
    assert_a_valid_call_goes_through();
    let transport = RecordingTransport::new().answering_ok("body");
    let denied = Port::new(transport.clone(), DenyAll);
    let err = denied
        .call(knowledge_request())
        .expect_err("a refused class must fail");
    assert_four_fields(&err);
    assert!(
        transport.sent().is_empty(),
        "the transport saw a refused call"
    );
}

// ---- the request that leaves ----

#[test]
fn a_call_sends_exactly_the_listed_fields_and_fills_the_path() {
    let transport = RecordingTransport::new().answering_ok("body");
    let response = port(&transport)
        .call(knowledge_request())
        .expect("the call goes through");
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"body");

    let sent = transport.sent();
    assert_eq!(sent.len(), 1);
    let wire = &sent[0];
    assert_eq!(wire.method, Method::Get);
    assert_eq!(wire.host, FILE_URL_HOST);
    assert_eq!(wire.path, "/knowledge.toml");
    assert!(wire.query.is_empty());
    assert!(wire.body.is_empty());
    assert_eq!(
        wire.headers,
        vec![("user-agent".to_string(), "usher".to_string())],
        "the headers are the listed ones and nothing else: no cookie, no accept-encoding"
    );
}

#[test]
fn a_field_the_descriptor_does_not_list_is_refused_before_the_transport() {
    assert_a_valid_call_goes_through();
    let transport = RecordingTransport::new().answering_ok("body");
    let err = port(&transport)
        .call(knowledge_request().with("cookie", "x"))
        .expect_err("an unlisted field must fail");
    assert_four_fields(&err);
    assert!(
        err.to_string().contains("cookie"),
        "the error names the field: {err}"
    );
    assert!(transport.sent().is_empty());
}

#[test]
fn the_caller_cannot_set_the_user_agent() {
    assert_a_valid_call_goes_through();
    let transport = RecordingTransport::new().answering_ok("body");
    let err = port(&transport)
        .call(knowledge_request().with("user-agent", "other"))
        .expect_err("the user agent is the port's");
    assert_four_fields(&err);
    assert!(transport.sent().is_empty());
}

#[test]
fn a_missing_path_value_is_refused_before_the_transport() {
    assert_a_valid_call_goes_through();
    let transport = RecordingTransport::new().answering_ok("body");
    let err = port(&transport)
        .call(Request::new(Egress::KnowledgeUpdate))
        .expect_err("a placeholder with no value must fail");
    assert_four_fields(&err);
    assert!(
        err.to_string().contains("file"),
        "the error names the field: {err}"
    );
    assert!(transport.sent().is_empty());
}

#[test]
fn a_path_value_that_could_change_the_destination_is_refused() {
    assert_a_valid_call_goes_through();
    for value in ["../x", "a/b", "a?b=c", "a#b", "a b", "", "a\r\nb"] {
        let transport = RecordingTransport::new().answering_ok("body");
        let err = port(&transport)
            .call(Request::new(Egress::KnowledgeUpdate).with("file", value))
            .expect_err("a path value with a separator must fail");
        assert_four_fields(&err);
        assert!(
            transport.sent().is_empty(),
            "value {value:?} reached the transport"
        );
    }
}

#[test]
fn a_field_set_twice_is_refused() {
    assert_a_valid_call_goes_through();
    let transport = RecordingTransport::new().answering_ok("body");
    let err = port(&transport)
        .call(knowledge_request().with("file", "other.toml"))
        .expect_err("a field set twice must fail");
    assert_four_fields(&err);
    assert!(transport.sent().is_empty());
}

// ---- the answer ----

#[test]
fn a_non_success_status_is_an_error_that_names_the_status() {
    let transport = RecordingTransport::new().answering(Ok(Response {
        status: 404,
        location: None,
        body: Vec::new(),
    }));
    let err = port(&transport)
        .call(knowledge_request())
        .expect_err("a 404 must fail");
    assert_four_fields(&err);
    assert!(err.to_string().contains("404"), "{err}");
}

#[test]
fn a_transport_error_comes_back_unchanged() {
    let failure = UsherError::new(
        "send",
        "the host answers",
        "connection refused",
        "try later",
    );
    let transport = RecordingTransport::new().answering(Err(failure.clone()));
    let err = port(&transport)
        .call(knowledge_request())
        .expect_err("a transport error must fail");
    assert_eq!(err, failure);
}

// ---- redirects ----

#[test]
fn a_redirect_to_an_unlisted_host_is_an_error_that_names_the_host() {
    let transport =
        RecordingTransport::new().answering_redirect("https://elsewhere.invalid/knowledge.toml");
    let err = port(&transport)
        .call(knowledge_request())
        .expect_err("an unlisted redirect must fail");
    assert_four_fields(&err);
    assert!(err.to_string().contains("elsewhere.invalid"), "{err}");
    assert_eq!(transport.sent().len(), 1, "the port followed the redirect");
}

#[test]
fn a_redirect_to_http_is_an_error_even_for_the_same_host() {
    let transport = RecordingTransport::new()
        .answering_redirect(format!("http://{FILE_URL_HOST}/knowledge.toml"));
    let err = port(&transport)
        .call(knowledge_request())
        .expect_err("a downgrade must fail");
    assert_four_fields(&err);
    assert_eq!(transport.sent().len(), 1);
}

#[test]
fn a_redirect_with_no_location_is_an_error() {
    assert_a_valid_call_goes_through();
    let transport = RecordingTransport::new().answering(Ok(Response {
        status: 302,
        location: None,
        body: Vec::new(),
    }));
    let err = port(&transport)
        .call(knowledge_request())
        .expect_err("a redirect with no target must fail");
    assert_four_fields(&err);
}

// ---- delegated runs ----

#[test]
fn the_delegated_installers_are_the_set_the_runner_port_will_use() {
    let names: Vec<&str> = DELEGATED_INSTALLERS.iter().map(|i| i.name).collect();
    assert_eq!(
        names,
        vec![
            "brew",
            "mise",
            "uv",
            "npm",
            "git",
            "sideshow",
            "vendor installers",
            "usher update"
        ]
    );
}

#[test]
fn every_delegated_installer_carries_the_fixed_note() {
    assert_eq!(
        DELEGATED_NOTE,
        "network access is the installer's, under its own settings"
    );
    for installer in DELEGATED_INSTALLERS {
        assert_eq!(installer.note, DELEGATED_NOTE, "{}", installer.name);
    }
}
