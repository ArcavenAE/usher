//! Reds for the egress port (Story 1.15, part A): the registry, the descriptor
//! rules, the field and redirect checks, and the class policy, over a recording
//! transport that never touches a network.

use std::collections::BTreeSet;

use usher_core::{
    ALL, Class, DELEGATED_INSTALLERS, DELEGATED_NOTE, DenyAll, Descriptor, Egress, Method,
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
    assert_eq!(wire.method(), Method::Get);
    assert_eq!(wire.host(), FILE_URL_HOST);
    assert_eq!(wire.path(), "/knowledge.toml");
    assert!(wire.query().is_empty());
    assert!(wire.body().is_empty());
    assert_eq!(
        wire.headers(),
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
            "git clone",
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

#[test]
fn a_path_value_of_two_dots_alone_is_refused() {
    assert_a_valid_call_goes_through();
    for value in ["..", "."] {
        let transport = RecordingTransport::new().answering_ok("body");
        let err = port(&transport)
            .call(Request::new(Egress::KnowledgeUpdate).with("file", value))
            .expect_err("a dot segment must fail");
        assert_four_fields(&err);
        assert!(
            transport.sent().is_empty(),
            "value {value:?} reached the transport"
        );
    }
}

// ---- the test seam: descriptors the registry does not hold ----

use usher_core::Field;

const FILES_HOST: &str = "files.invalid";
const ASSETS_HOST: &str = "assets.invalid";

const UA: Field = Field {
    name: "user-agent",
    place: Place::Header,
    meaning: "the program name, fixed",
};

const REDIRECTING: Descriptor = Descriptor {
    name: "test-redirecting",
    method: Method::Get,
    host: FILES_HOST,
    path: "/{file}",
    purpose: "a test call that may be redirected",
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
};

const SHAPED: Descriptor = Descriptor {
    name: "test-shaped",
    method: Method::Post,
    host: FILES_HOST,
    path: "/report",
    purpose: "a test call with query and body fields",
    class: Class::PersonStarted,
    fields: &[
        UA,
        Field {
            name: "q2",
            place: Place::Query,
            meaning: "second query field, listed first",
        },
        Field {
            name: "b2",
            place: Place::Body,
            meaning: "second body field, listed second",
        },
        Field {
            name: "q1",
            place: Place::Query,
            meaning: "first query field, listed third",
        },
        Field {
            name: "b1",
            place: Place::Body,
            meaning: "first body field, listed fourth",
        },
    ],
    redirect_hosts: &[],
};

const VERSION_CHECK: Descriptor = Descriptor {
    name: "test-version-check",
    method: Method::Get,
    host: FILES_HOST,
    path: "/version",
    purpose: "a test version check",
    class: Class::VersionCheck,
    fields: &[UA],
    redirect_hosts: &[],
};

const PERSON_STARTED: Descriptor = Descriptor {
    class: Class::PersonStarted,
    ..VERSION_CHECK
};

fn described_request() -> Request {
    Request::new(Egress::KnowledgeUpdate)
}

#[test]
fn a_redirect_to_a_listed_host_is_followed_once_and_its_answer_returned() {
    let transport = RecordingTransport::new()
        .answering_redirect(format!("https://{ASSETS_HOST}/release/knowledge.toml"))
        .answering_ok("from the asset host");
    let response = port(&transport)
        .call_described(
            &REDIRECTING,
            described_request().with("file", "knowledge.toml"),
        )
        .expect("a listed redirect is followed");
    assert_eq!(response.body, b"from the asset host");

    let sent = transport.sent();
    assert_eq!(sent.len(), 2, "one call and one follow");
    assert_eq!(sent[0].host(), FILES_HOST);
    assert_eq!(sent[0].path(), "/knowledge.toml");
    assert_eq!(sent[1].host(), ASSETS_HOST);
    assert_eq!(sent[1].path(), "/release/knowledge.toml");
    assert_eq!(sent[1].method(), Method::Get);
    assert_eq!(
        sent[1].headers(),
        vec![("user-agent".to_string(), "usher".to_string())]
    );
}

#[test]
fn a_second_redirect_after_the_follow_is_refused_and_not_followed() {
    let transport = RecordingTransport::new()
        .answering_redirect(format!("https://{ASSETS_HOST}/one"))
        .answering_redirect(format!("https://{ASSETS_HOST}/two"))
        .answering_ok("never reached");
    let err = port(&transport)
        .call_described(
            &REDIRECTING,
            described_request().with("file", "knowledge.toml"),
        )
        .expect_err("a second redirect must fail");
    assert_four_fields(&err);
    assert_eq!(
        transport.sent().len(),
        2,
        "the port followed a second redirect"
    );
}

#[test]
fn a_listed_redirect_still_refuses_an_unlisted_host_and_a_downgrade() {
    assert_a_valid_call_goes_through();
    for target in [
        "https://elsewhere.invalid/x".to_string(),
        format!("http://{ASSETS_HOST}/x"),
    ] {
        let transport = RecordingTransport::new().answering_redirect(target.clone());
        let err = port(&transport)
            .call_described(&REDIRECTING, described_request().with("file", "k.toml"))
            .expect_err("an unlisted or downgraded redirect must fail");
        assert_four_fields(&err);
        assert_eq!(transport.sent().len(), 1, "followed {target}");
    }
}

#[test]
fn an_unlisted_query_or_body_field_is_refused_before_the_transport() {
    assert_a_valid_call_goes_through();
    for extra in ["extra-query", "extra-body"] {
        let transport = RecordingTransport::new().answering_ok("body");
        let err = port(&transport)
            .call_described(
                &SHAPED,
                described_request().with("q1", "1").with(extra, "x"),
            )
            .expect_err("an unlisted field must fail");
        assert_four_fields(&err);
        assert!(err.to_string().contains(extra), "{err}");
        assert!(transport.sent().is_empty(), "{extra} reached the transport");
    }
}

#[test]
fn listed_query_and_body_fields_go_in_descriptor_order() {
    let transport = RecordingTransport::new().answering_ok("body");
    port(&transport)
        .call_described(
            &SHAPED,
            described_request()
                .with("b1", "four")
                .with("q1", "three")
                .with("b2", "two")
                .with("q2", "one"),
        )
        .expect("listed fields go through");
    let sent = transport.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].method(), Method::Post);
    assert_eq!(sent[0].path(), "/report");
    assert_eq!(
        sent[0].query(),
        vec![
            ("q2".to_string(), "one".to_string()),
            ("q1".to_string(), "three".to_string())
        ]
    );
    assert_eq!(
        sent[0].body(),
        vec![
            ("b2".to_string(), "two".to_string()),
            ("b1".to_string(), "four".to_string())
        ]
    );
}

#[test]
fn a_version_check_descriptor_never_reaches_the_transport_under_the_one_1_15_policy() {
    let allowed = RecordingTransport::new().answering_ok("control");
    port(&allowed)
        .call_described(&PERSON_STARTED, described_request())
        .expect("a person-started descriptor goes through");
    assert_eq!(allowed.sent().len(), 1);

    let transport = RecordingTransport::new().answering_ok("body");
    let err = port(&transport)
        .call_described(&VERSION_CHECK, described_request())
        .expect_err("a version check is refused until its switch exists");
    assert_four_fields(&err);
    assert!(transport.sent().is_empty());
}

#[test]
fn a_percent_sign_or_dot_segment_in_a_path_value_is_refused_naming_the_field_and_the_check() {
    assert_a_valid_call_goes_through();
    for value in ["a%2Fb", "%2e%2e", "..", ".", "50%"] {
        let transport = RecordingTransport::new().answering_ok("body");
        let err = port(&transport)
            .call(Request::new(Egress::KnowledgeUpdate).with("file", value))
            .expect_err("a percent sign or a dot segment must fail");
        assert!(
            err.to_string().contains("file"),
            "{value:?}: the error names the field: {err}"
        );
        assert_eq!(
            err.check(),
            "a path value holds no `/`, `?`, `#`, `%`, whitespace or control character, and is not `.`, `..` or empty",
            "{value:?}"
        );
        assert!(
            transport.sent().is_empty(),
            "value {value:?} reached the transport"
        );
    }
}

#[test]
fn a_control_character_that_is_not_whitespace_in_a_path_value_is_refused() {
    assert_a_valid_call_goes_through();
    for value in ["a\u{0}b", "a\u{7f}b"] {
        let transport = RecordingTransport::new().answering_ok("body");
        let err = port(&transport)
            .call(Request::new(Egress::KnowledgeUpdate).with("file", value))
            .expect_err("a control character must fail");
        assert_four_fields(&err);
        assert!(err.to_string().contains("file"), "{value:?}: {err}");
        assert!(
            transport.sent().is_empty(),
            "value {value:?} reached the transport"
        );
    }
}

#[test]
fn a_204_answer_is_a_success() {
    let transport = RecordingTransport::new().answering(Ok(Response {
        status: 204,
        location: None,
        body: Vec::new(),
    }));
    let response = port(&transport)
        .call(knowledge_request())
        .expect("a 204 is a success");
    assert_eq!(response.status, 204);
}

#[test]
fn a_303_and_a_308_to_a_listed_host_are_each_followed_once() {
    for status in [303_u16, 308] {
        let transport = RecordingTransport::new()
            .answering(Ok(Response {
                status,
                location: Some(format!("https://{ASSETS_HOST}/moved")),
                body: Vec::new(),
            }))
            .answering_ok("moved body");
        let response = port(&transport)
            .call_described(&REDIRECTING, described_request().with("file", "k.toml"))
            .unwrap_or_else(|e| panic!("a {status} to a listed host is followed: {e}"));
        assert_eq!(response.body, b"moved body", "{status}");
        let sent = transport.sent();
        assert_eq!(sent.len(), 2, "{status}: one call and one follow");
        assert_eq!(sent[1].host(), ASSETS_HOST, "{status}");
        assert_eq!(sent[1].path(), "/moved", "{status}");
    }
}

#[test]
fn a_redirect_with_no_location_on_a_descriptor_that_lists_a_host_is_refused_with_one_send() {
    let transport = RecordingTransport::new()
        .answering(Ok(Response {
            status: 302,
            location: None,
            body: Vec::new(),
        }))
        .answering_ok("never reached");
    let err = port(&transport)
        .call_described(&REDIRECTING, described_request().with("file", "k.toml"))
        .expect_err("a redirect with no target must fail");
    assert_four_fields(&err);
    assert_eq!(transport.sent().len(), 1, "the port sent a second request");
}

const POSTING: Descriptor = Descriptor {
    name: "test-posting",
    method: Method::Post,
    host: FILES_HOST,
    path: "/report",
    purpose: "a test POST that lists a redirect host",
    class: Class::PersonStarted,
    fields: &[
        UA,
        Field {
            name: "b1",
            place: Place::Body,
            meaning: "a body field",
        },
    ],
    redirect_hosts: &[ASSETS_HOST],
};

const EXTRA_HEADER: Descriptor = Descriptor {
    name: "test-extra-header",
    method: Method::Get,
    host: FILES_HOST,
    path: "/{file}",
    purpose: "a test GET with a listed header beyond the user agent",
    class: Class::PersonStarted,
    fields: &[
        UA,
        Field {
            name: "x-note",
            place: Place::Header,
            meaning: "an extra listed header",
        },
        Field {
            name: "file",
            place: Place::Path,
            meaning: "which file",
        },
    ],
    redirect_hosts: &[ASSETS_HOST],
};

#[test]
fn a_redirect_on_a_non_get_call_is_refused_naming_the_status_with_one_send() {
    for status in [301_u16, 307, 308] {
        let transport = RecordingTransport::new()
            .answering(Ok(Response {
                status,
                location: Some(format!("https://{ASSETS_HOST}/moved")),
                body: Vec::new(),
            }))
            .answering_ok("never reached");
        let err = port(&transport)
            .call_described(&POSTING, described_request().with("b1", "one"))
            .expect_err("a redirect on a POST must fail");
        assert_four_fields(&err);
        assert!(err.to_string().contains(&status.to_string()), "{err}");
        assert_eq!(transport.sent().len(), 1, "{status}: the port followed");
    }
}

#[test]
fn a_get_with_an_extra_listed_header_follows_with_the_user_agent_only() {
    let transport = RecordingTransport::new()
        .answering_redirect(format!("https://{ASSETS_HOST}/moved"))
        .answering_ok("moved body");
    port(&transport)
        .call_described(
            &EXTRA_HEADER,
            described_request()
                .with("file", "k.toml")
                .with("x-note", "private"),
        )
        .expect("the follow goes through");
    let sent = transport.sent();
    assert_eq!(sent.len(), 2);
    assert_eq!(
        sent[0].headers(),
        vec![
            ("user-agent".to_string(), "usher".to_string()),
            ("x-note".to_string(), "private".to_string())
        ],
        "the first request carries the listed header"
    );
    assert_eq!(
        sent[1].headers(),
        vec![("user-agent".to_string(), "usher".to_string())],
        "the follow carries the user agent and nothing else"
    );
    assert!(sent[1].query().is_empty() && sent[1].body().is_empty());
}

#[test]
fn a_redirect_path_holding_a_space_a_control_character_or_a_fragment_is_refused_with_one_send() {
    assert_a_valid_call_goes_through();
    for path in ["/a b", "/a\u{0}b", "/a#b"] {
        let transport = RecordingTransport::new()
            .answering_redirect(format!("https://{ASSETS_HOST}{path}"))
            .answering_ok("never reached");
        let err = port(&transport)
            .call_described(&REDIRECTING, described_request().with("file", "k.toml"))
            .expect_err("a refused path must fail");
        assert_four_fields(&err);
        assert_eq!(transport.sent().len(), 1, "{path:?}: the port followed");
    }
}

#[test]
fn the_test_seam_method_is_compiled_only_under_its_feature() {
    let source = include_str!("../src/egress.rs");
    let lines: Vec<&str> = source.lines().map(str::trim).collect();
    let at = lines
        .iter()
        .position(|l| l.starts_with("pub fn call_described"))
        .expect("call_described is in egress.rs");
    let attribute = lines[..at]
        .iter()
        .rev()
        .find(|l| !l.starts_with("///") && !l.is_empty());
    assert_eq!(
        attribute,
        Some(&"#[cfg(feature = \"test-seam\")]"),
        "call_described must sit directly under its cfg"
    );
}

const QUERYING: Descriptor = Descriptor {
    name: "test-querying",
    method: Method::Get,
    host: FILES_HOST,
    path: "/{file}",
    purpose: "a test GET with a listed query field",
    class: Class::PersonStarted,
    fields: &[
        UA,
        Field {
            name: "q",
            place: Place::Query,
            meaning: "a listed query field",
        },
        Field {
            name: "file",
            place: Place::Path,
            meaning: "which file",
        },
    ],
    redirect_hosts: &[ASSETS_HOST],
};

const BODIED_GET: Descriptor = Descriptor {
    name: "test-bodied-get",
    method: Method::Get,
    host: FILES_HOST,
    path: "/{file}",
    purpose: "a test GET with a listed body field",
    class: Class::PersonStarted,
    fields: &[
        UA,
        Field {
            name: "b",
            place: Place::Body,
            meaning: "a listed body field",
        },
        Field {
            name: "file",
            place: Place::Path,
            meaning: "which file",
        },
    ],
    redirect_hosts: &[ASSETS_HOST],
};

#[test]
fn a_follow_carries_no_query_and_no_body_field_to_the_second_host() {
    for (descriptor, name) in [(&QUERYING, "q"), (&BODIED_GET, "b")] {
        let transport = RecordingTransport::new()
            .answering_redirect(format!("https://{ASSETS_HOST}/moved"))
            .answering_ok("moved body");
        port(&transport)
            .call_described(
                descriptor,
                described_request()
                    .with("file", "k.toml")
                    .with(name, "private"),
            )
            .expect("the follow goes through");
        let sent = transport.sent();
        assert_eq!(sent.len(), 2, "{name}");
        let first = if name == "q" {
            &sent[0].query()
        } else {
            &sent[0].body()
        };
        assert_eq!(
            first,
            &vec![(name.to_string(), "private".to_string())],
            "the first request carries the listed {name} field"
        );
        assert!(
            sent[1].query().is_empty() && sent[1].body().is_empty(),
            "{name}: the follow carried a query or body field"
        );
    }
}

#[test]
fn a_redirect_on_a_post_with_no_body_field_is_refused_with_one_send() {
    let transport = RecordingTransport::new()
        .answering(Ok(Response {
            status: 307,
            location: Some(format!("https://{ASSETS_HOST}/moved")),
            body: Vec::new(),
        }))
        .answering_ok("never reached");
    let err = port(&transport)
        .call_described(&POSTING, described_request())
        .expect_err("a redirect on a POST must fail");
    assert_four_fields(&err);
    assert!(err.to_string().contains("307"), "{err}");
    assert_eq!(transport.sent().len(), 1, "the port followed");
}

// ---- the request cannot be built outside the port ----

fn block<'a>(source: &'a str, start: &str) -> &'a str {
    let from = source
        .find(start)
        .unwrap_or_else(|| panic!("`{start}` is in egress.rs"));
    let rest = &source[from..];
    let to = rest.find("\n}\n").expect("the block ends");
    &rest[..to]
}

#[test]
fn wire_request_has_no_public_field() {
    let source = include_str!("../src/egress.rs");
    let body = block(source, "pub struct WireRequest {");
    for line in body.lines().skip(1) {
        let line = line.trim();
        assert!(
            line.starts_with("///") || !line.starts_with("pub "),
            "a WireRequest field is public: {line}"
        );
    }
}

#[test]
fn wire_request_has_no_public_function_that_returns_one() {
    let source = include_str!("../src/egress.rs");
    let body = block(source, "impl WireRequest {");
    for line in body.lines() {
        let line = line.trim();
        if line.starts_with("pub fn") {
            assert!(
                !line.contains("-> Self") && !line.contains("-> WireRequest"),
                "a public function returns a WireRequest: {line}"
            );
        }
    }
    assert!(
        body.contains("pub fn host("),
        "the accessors are in this block"
    );
}

// ---- the policy is sealed, and the test policies are test-only ----

#[test]
fn deny_all_refuses_every_class_and_allow_all_allows_every_class() {
    use usher_core::AllowAll;
    for class in [Class::PersonStarted, Class::VersionCheck, Class::Reporting] {
        assert!(!DenyAll.allows(class), "DenyAll allowed {class:?}");
        assert!(AllowAll.allows(class), "AllowAll refused {class:?}");
    }
}

#[test]
fn the_test_policies_are_compiled_only_under_the_test_seam_feature() {
    let source = include_str!("../src/egress.rs");
    let lines: Vec<&str> = source.lines().map(str::trim).collect();
    for name in ["pub struct DenyAll;", "pub struct AllowAll;"] {
        let at = lines
            .iter()
            .position(|l| *l == name)
            .unwrap_or_else(|| panic!("`{name}` is in egress.rs"));
        let attributes: Vec<&&str> = lines[..at]
            .iter()
            .rev()
            .take_while(|l| l.starts_with("#[") || l.starts_with("///"))
            .collect();
        assert!(
            attributes
                .iter()
                .any(|l| **l == "#[cfg(feature = \"test-seam\")]"),
            "{name} must sit under its cfg"
        );
    }
}

#[test]
fn the_policy_trait_has_a_private_supertrait() {
    let source = include_str!("../src/egress.rs");
    assert!(
        source.contains("pub trait Policy: sealed::Sealed"),
        "Policy must name a private supertrait"
    );
    assert!(source.contains("mod sealed {"), "the sealing module exists");
}

// ---- header values ----

fn header_call(value: &str) -> (Result<Response, UsherError>, RecordingTransport) {
    let transport = RecordingTransport::new().answering_ok("body");
    let result = port(&transport).call_described(
        &EXTRA_HEADER,
        described_request()
            .with("file", "k.toml")
            .with("x-note", value),
    );
    (result, transport)
}

#[test]
fn a_header_value_of_printable_ascii_is_sent() {
    let (result, transport) = header_call("a printable value ~");
    result.expect("a printable value goes out");
    assert_eq!(
        transport.sent()[0].headers()[1],
        ("x-note".to_string(), "a printable value ~".to_string())
    );
}

#[test]
fn a_header_value_with_a_control_or_non_ascii_byte_or_edge_space_is_refused_without_echo() {
    let (control, _) = header_call("a printable value");
    control.expect("the control call goes through");
    for value in [
        "secret\r\nvalue",
        "secret\tvalue",
        "secretcaf\u{e9}",
        "secret\u{7f}",
        "secret\u{0}",
    ] {
        let (result, transport) = header_call(value);
        let err = result.expect_err("an unprintable header value must fail");
        assert_eq!(err.step(), "send test-extra-header", "{value:?}");
        assert_eq!(err.check(), "a header value holds only printable ASCII");
        assert_eq!(
            err.status(),
            "header x-note holds a character that is not printable ASCII"
        );
        assert_eq!(
            err.next_action(),
            "send a header value of printable ASCII only"
        );
        assert!(
            !err.to_string().contains("secret"),
            "{value:?}: the error echoes the value: {err}"
        );
        assert!(
            transport.sent().is_empty(),
            "{value:?} reached the transport"
        );
    }
}

// ---- query values ----

fn query_call(value: &str) -> (Result<Response, UsherError>, RecordingTransport) {
    let transport = RecordingTransport::new().answering_ok("body");
    let result = port(&transport).call_described(&SHAPED, described_request().with("q1", value));
    (result, transport)
}

#[test]
fn query_values_are_percent_encoded_outside_the_unreserved_set() {
    for (value, wanted) in [
        ("a b&c=d", "a%20b%26c%3Dd"),
        ("50%", "50%25"),
        ("\u{e9}", "%C3%A9"),
        ("A-Za-z0-9._~", "A-Za-z0-9._~"),
        ("a/b?c#d+e", "a%2Fb%3Fc%23d%2Be"),
    ] {
        let (result, transport) = query_call(value);
        result.unwrap_or_else(|e| panic!("{value:?} goes out: {e}"));
        assert_eq!(
            transport.sent()[0].query(),
            [("q1".to_string(), wanted.to_string())],
            "{value:?}"
        );
    }
}

#[test]
fn a_control_character_in_a_query_value_is_refused_without_echo() {
    let (control, _) = query_call("fine");
    control.expect("the control call goes through");
    for value in ["secret\nvalue", "secret\u{0}", "secret\u{7f}", "secret\r"] {
        let (result, transport) = query_call(value);
        let err = result.expect_err("a control character must fail");
        assert_eq!(err.status(), "query q1 holds a control character");
        assert!(!err.to_string().contains("secret"), "{err}");
        assert!(
            transport.sent().is_empty(),
            "{value:?} reached the transport"
        );
    }
}

#[test]
fn encoded_query_fields_keep_the_descriptor_order() {
    let transport = RecordingTransport::new().answering_ok("body");
    port(&transport)
        .call_described(
            &SHAPED,
            described_request().with("q1", "b c").with("q2", "a b"),
        )
        .expect("goes out");
    assert_eq!(
        transport.sent()[0].query(),
        [
            ("q2".to_string(), "a%20b".to_string()),
            ("q1".to_string(), "b%20c".to_string())
        ]
    );
}

#[test]
fn every_registered_query_name_is_a_plain_token() {
    assert!(!ALL.is_empty(), "ALL names no call");
    for egress in ALL {
        for field in descriptor(*egress)
            .fields
            .iter()
            .filter(|f| f.place == Place::Query)
        {
            assert!(
                !field.name.is_empty()
                    && field
                        .name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')),
                "query name `{}` is not [A-Za-z0-9_.-]",
                field.name
            );
        }
    }
}

// ---- what a redirect's text may put in an error ----

fn redirect_error(location: &str) -> UsherError {
    let transport = RecordingTransport::new().answering_redirect(location);
    port(&transport)
        .call(knowledge_request())
        .expect_err("an unlisted redirect must fail")
}

#[test]
fn a_valid_host_name_in_a_refused_redirect_is_echoed_and_no_path_or_query_is() {
    let err = redirect_error("https://elsewhere.invalid/secret?token=1");
    let text = err.to_string();
    assert!(text.contains("elsewhere.invalid"), "{text}");
    assert!(!text.contains("secret"), "the path is echoed: {text}");
    assert!(!text.contains("token"), "the query is echoed: {text}");
}

#[test]
fn a_host_that_is_not_a_valid_name_is_never_echoed() {
    let long = "a".repeat(254);
    for host in [
        "ev\u{1b}il.invalid".to_string(),
        "user@assets.invalid".to_string(),
        "assets.invalid:443".to_string(),
        "caf\u{e9}.invalid".to_string(),
        "a b.invalid".to_string(),
        String::new(),
        long,
    ] {
        let err = redirect_error(&format!("https://{host}/x"));
        let text = err.to_string();
        assert!(
            text.contains("a host that is not a valid name"),
            "{host:?}: {text}"
        );
        assert!(!text.contains('\u{1b}'), "{host:?}: the error holds ESC");
        if !host.is_empty() {
            assert!(!text.contains(&host), "{host:?}: the host is echoed");
        }
        assert_four_fields(&err);
    }
}

#[test]
fn a_refused_redirect_path_on_a_listed_host_is_not_echoed() {
    let transport =
        RecordingTransport::new().answering_redirect(format!("https://{ASSETS_HOST}/secret path"));
    let err = port(&transport)
        .call_described(&REDIRECTING, described_request().with("file", "k.toml"))
        .expect_err("a path with a space must fail");
    assert!(!err.to_string().contains("secret"), "{err}");
}

// ---- the call's name ----

#[test]
fn every_descriptor_name_is_unique_kebab_case_and_the_variants_own() {
    assert!(!ALL.is_empty(), "ALL names no call");
    let mut seen = BTreeSet::new();
    for egress in ALL {
        let name = descriptor(*egress).name;
        assert_eq!(
            name,
            every_variant_named(*egress),
            "the name is the variant's kebab form"
        );
        assert!(
            (1..=64).contains(&name.len())
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "name `{name}` is not 1 to 64 of [a-z0-9-]"
        );
        assert!(seen.insert(name), "name `{name}` is used twice");
    }
}

#[test]
fn a_header_value_with_a_leading_or_trailing_space_has_its_own_status_and_no_echo() {
    let (control, _) = header_call("a printable value");
    control.expect("the control call goes through");
    for value in [" secret", "secret ", " secret ", " "] {
        let (result, transport) = header_call(value);
        let err = result.expect_err("an edge space must fail");
        assert_eq!(err.step(), "send test-extra-header", "{value:?}");
        assert_eq!(
            err.status(),
            "header x-note starts or ends with a space",
            "{value:?}"
        );
        assert!(!err.to_string().contains("secret"), "{err}");
        assert!(
            transport.sent().is_empty(),
            "{value:?} reached the transport"
        );
    }
}
