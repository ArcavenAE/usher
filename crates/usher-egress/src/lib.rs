//! The one real transport for the egress port (story 1.15, part B).
//!
//! This crate is the only place usher opens a connection. The port in `usher-core`
//! checks every call against its descriptor first; this crate carries the checked
//! request to the host over https and hands the answer back. It follows no redirect,
//! sends no header the port did not list, and reads at most one byte past the limit
//! of every answer.

#![forbid(unsafe_code)]

use std::io::{self, Read};

use usher_core::{Method, Response, Transport, UsherError, WireRequest};

/// Seconds the client waits for a connection.
pub const CONNECT_TIMEOUT_SECS: u64 = 0;

/// Seconds the client allows a whole call.
pub const CALL_TIMEOUT_SECS: u64 = 0;

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
        connect_timeout_secs: 0,
        call_timeout_secs: 0,
        max_redirects: 0,
        https_only: false,
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
    let _ = (host, kind);
    UsherError::new("stub", "stub", "stub", "stub")
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

/// Plans one request, or refuses one the client does not send.
pub fn plan_request(request: &WireRequest) -> Result<RequestPlan, UsherError> {
    let _ = request;
    Ok(RequestPlan {
        method: Method::Get,
        uri: String::new(),
        headers: Vec::new(),
    })
}

/// Reads at most `limit + 1` bytes: one more than the limit is enough for the port to
/// see that the answer is too large, and no more is read.
pub fn read_limited(reader: impl Read, limit: u64) -> io::Result<Vec<u8>> {
    let _ = (reader, limit);
    Ok(Vec::new())
}

/// Builds the port's [`Response`] from what the client received, reading the body with
/// the limit of [`read_limited`] whatever the status.
pub fn response_from(
    status: u16,
    location: Option<&str>,
    body: impl Read,
    limit: u64,
) -> Result<Response, UsherError> {
    let _ = (location, body, limit);
    Ok(Response {
        status,
        location: None,
        body: Vec::new(),
    })
}

/// The real transport.
#[derive(Debug, Clone, Default)]
pub struct HttpTransport;

impl HttpTransport {
    /// A transport that dials only the hosts the port gives it.
    pub fn new() -> Self {
        Self
    }
}

impl Transport for HttpTransport {
    fn send(&self, request: &WireRequest) -> Result<Response, UsherError> {
        let _ = request;
        Ok(Response {
            status: 0,
            location: None,
            body: Vec::new(),
        })
    }
}
