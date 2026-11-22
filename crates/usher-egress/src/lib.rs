//! The one real transport for the egress port (story 1.15, part B).
//!
//! This crate is the only place usher opens a connection. The port in `usher-core`
//! checks every call against its descriptor first; this crate carries the checked
//! request to the host over https and hands the answer back. It follows no redirect,
//! sends no header the port did not list, and reads at most one byte past the limit
//! of every answer.

#![forbid(unsafe_code)]
// This crate is the one place that may name the http client and open a socket: the
// workspace clippy.toml refuses those types and calls everywhere else.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

mod map;

use std::io::{self, Read};

use usher_core::{Method, Response, Transport, UsherError, WireRequest};

/// Seconds the client waits for a connection.
pub const CONNECT_TIMEOUT_SECS: u64 = 10;

/// Seconds the client allows a whole call.
pub const CALL_TIMEOUT_SECS: u64 = 30;

/// What the client is configured to do and not to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSettings {
    /// Seconds to wait for a connection.
    pub connect_timeout_secs: u64,
    /// Seconds a whole call may take.
    pub call_timeout_secs: u64,
    /// Redirects the client follows itself.
    pub max_redirects: u32,
    /// Whether any scheme but https is refused.
    pub https_only: bool,
    /// Whether a 4xx or 5xx answer is turned into an error by the client.
    pub http_status_as_error: bool,
    /// Whether the client asks for or decodes gzip.
    pub gzip: bool,
    /// Whether the client keeps cookies.
    pub cookies: bool,
}

/// The settings the one agent is built from.
pub fn agent_settings() -> AgentSettings {
    AgentSettings {
        connect_timeout_secs: CONNECT_TIMEOUT_SECS,
        call_timeout_secs: CALL_TIMEOUT_SECS,
        max_redirects: 0,
        https_only: true,
        http_status_as_error: false,
        gzip: false,
        cookies: false,
    }
}

/// Which limit a timeout hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutKind {
    /// The connection was not made in time.
    Connect,
    /// The whole call took too long.
    Call,
}

/// The error for a call that did not finish in time.
pub fn timeout_error(host: &str, kind: TimeoutKind) -> UsherError {
    let (secs, which) = match kind {
        TimeoutKind::Connect => (CONNECT_TIMEOUT_SECS, "connect"),
        TimeoutKind::Call => (CALL_TIMEOUT_SECS, "whole call"),
    };
    UsherError::new(
        "send request",
        "the host answers within the limit",
        format!("{host}: no answer within {secs} s ({which})"),
        "check the connection and try again",
    )
}

/// A request as the client will send it, before any connection is made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestPlan {
    /// The method.
    pub method: Method,
    /// The full address, always https.
    pub uri: String,
    /// The headers, exactly the port's, in order.
    pub headers: Vec<(String, String)>,
}

/// Plans one request, or refuses one the client does not send. The port has already
/// percent-encoded the query, so it is joined as it stands.
pub fn plan_request(request: &WireRequest) -> Result<RequestPlan, UsherError> {
    if !request.body().is_empty() {
        return Err(UsherError::new(
            "send request",
            "the call sends no body",
            format!("{}: no registered call sends a body", request.host()),
            "no change is needed; a call that sends a body needs its own ruling",
        ));
    }
    if request.host().contains('@') {
        return Err(UsherError::new(
            "send request",
            "the host is a bare host name",
            "the host holds a user name or password".to_string(),
            "remove the user name and password from the host; a host that needs credentials needs its own ruling",
        ));
    }
    let mut uri = format!("https://{}{}", request.host(), request.path());
    for (index, (name, value)) in request.query().iter().enumerate() {
        uri.push(if index == 0 { '?' } else { '&' });
        uri.push_str(name);
        uri.push('=');
        uri.push_str(value);
    }
    Ok(RequestPlan {
        method: request.method(),
        uri,
        headers: request.headers().to_vec(),
    })
}

/// Reads at most `limit + 1` bytes: one more than the limit is enough for the port to
/// see that the answer is too large, and no more is read.
pub fn read_limited(reader: impl Read, limit: u64) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Builds the port's [`Response`] from what the client received, reading the body with
/// the limit of [`read_limited`] whatever the status.
pub fn response_from(
    status: u16,
    location: Option<&str>,
    body: impl Read,
    limit: u64,
) -> Result<Response, UsherError> {
    let body = read_limited(body, limit).map_err(|err| {
        UsherError::new(
            "send request",
            "the answer is readable",
            clean(&err.to_string()),
            "try again later",
        )
    })?;
    Ok(Response {
        status,
        location: location.map(str::to_string),
        body,
    })
}

/// The port's [`Response`] for what the client received: the status, the `Location`
/// header, and a body read with the limit of [`read_limited`]. A read that fails is an
/// error naming the host; one the client itself ended (a timeout, for one) reads as
/// the same error a failed exchange gives.
pub fn answer_from(
    host: &str,
    status: u16,
    headers: &ureq::http::HeaderMap,
    body: impl Read,
    limit: u64,
) -> Result<Response, UsherError> {
    let location = headers
        .get("location")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let body = read_limited(body, limit).map_err(|err| {
        match err
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<ureq::Error>())
        {
            Some(client) => transport_error(host, client),
            None => UsherError::new(
                "send request",
                "the answer is readable",
                format!("{host}: {}", clean(&err.to_string())),
                "try again later",
            ),
        }
    })?;
    Ok(Response {
        status,
        location,
        body,
    })
}

/// The port's [`Response`] for the answer to one request. Placeholder values until the
/// wiring is pinned: the status, the headers, the host and the limit are not yet taken
/// from the request and the parts.
pub fn answer_for(
    _request: &WireRequest,
    _parts: &ureq::http::response::Parts,
    body: impl Read,
) -> Result<Response, UsherError> {
    answer_from("", 0, &ureq::http::HeaderMap::new(), body, u64::MAX)
}

/// A cause text with control characters replaced, since part of it can come from the
/// remote end.
fn clean(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { '?' } else { c })
        .collect()
}

/// The error for a failure of the connection or the exchange.
fn transport_error(host: &str, err: &ureq::Error) -> UsherError {
    match err {
        ureq::Error::Timeout(ureq::Timeout::Connect) => timeout_error(host, TimeoutKind::Connect),
        ureq::Error::Timeout(_) => timeout_error(host, TimeoutKind::Call),
        other => UsherError::new(
            "send request",
            "reach the host over https",
            format!("{host}: {}", clean(&other.to_string())),
            "check the connection and try again",
        ),
    }
}

/// The real transport.
#[derive(Debug, Clone)]
pub struct HttpTransport {
    agent: ureq::Agent,
}

impl Default for HttpTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpTransport {
    /// A transport that dials only the hosts the port gives it.
    pub fn new() -> Self {
        Self {
            agent: ureq::Agent::new_with_config(map::to_config(&agent_settings())),
        }
    }
}

impl Transport for HttpTransport {
    fn send(&self, request: &WireRequest) -> Result<Response, UsherError> {
        let host = request.host();
        let plan = plan_request(request)?;
        let outgoing = map::to_request(&plan)?;
        let mut answer = self
            .agent
            .run(outgoing)
            .map_err(|err| transport_error(host, &err))?;
        let status = answer.status().as_u16();
        let headers = answer.headers().clone();
        answer_from(
            host,
            status,
            &headers,
            answer.body_mut().as_reader(),
            request.max_response_bytes(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cause_carrying_control_characters_is_cleaned_in_the_transport_error() {
        let err = transport_error(
            "h.usher.invalid",
            &ureq::Error::BadUri("a\u{1b}b".to_string()),
        );
        assert!(err.status().contains("a?b"), "{err}");
        assert!(!err.status().contains('\u{1b}'), "{err}");
    }

    #[test]
    fn a_connect_timeout_from_the_client_reads_as_the_connect_text() {
        let err = transport_error(
            "h.usher.invalid",
            &ureq::Error::Timeout(ureq::Timeout::Connect),
        );
        assert_eq!(
            err.status(),
            "h.usher.invalid: no answer within 10 s (connect)"
        );
    }

    #[test]
    fn any_other_timeout_from_the_client_reads_as_the_whole_call_text() {
        for kind in [
            ureq::Timeout::Global,
            ureq::Timeout::PerCall,
            ureq::Timeout::RecvBody,
        ] {
            let err = transport_error("h.usher.invalid", &ureq::Error::Timeout(kind));
            assert_eq!(
                err.status(),
                "h.usher.invalid: no answer within 30 s (whole call)",
                "{kind:?}"
            );
        }
    }

    #[test]
    fn another_client_error_names_the_host_first_under_the_transports_own_check() {
        let err = transport_error("h.usher.invalid", &ureq::Error::TooManyRedirects);
        assert_eq!(err.step(), "send request");
        assert_eq!(err.check(), "reach the host over https");
        assert!(err.status().starts_with("h.usher.invalid: "), "{err}");
    }

    #[test]
    fn a_cause_text_loses_its_control_characters() {
        assert_eq!(clean("a\u{1b}[31mb\n"), "a?[31mb?");
    }

    #[test]
    fn a_read_limit_of_the_largest_number_does_not_wrap() {
        assert_eq!(read_limited(&b"abc"[..], u64::MAX).expect("a read"), b"abc");
    }
}
