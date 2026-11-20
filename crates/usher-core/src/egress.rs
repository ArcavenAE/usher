//! The egress port (AD-14, Story 1.15): the one place an outbound call is
//! described and checked.
//!
//! Every call usher makes is one variant of [`Egress`]. Its [`Descriptor`] says
//! where it goes, why, in which class, and every field it sends. A [`Request`]
//! names an [`Egress`], so [`Port::call`] always has a descriptor to check it
//! against; the one way around that is [`Port::call_described`], which exists
//! only under the `test-seam` feature.
//!
//! The port holds no HTTP client: a [`Transport`] does, and the one real
//! transport lives outside this crate. A transport only ever receives a
//! [`WireRequest`], which nothing outside this module can build, and a
//! [`Policy`] only this crate can implement decides which classes of call go
//! out.
//!
//! The published behavioral profile is generated from [`ALL`] and
//! [`DELEGATED_INSTALLERS`].

use crate::UsherError;

/// Who starts a call, and so which switch governs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// A call the person asked for by name: an install, a clone, an update.
    PersonStarted,
    /// The behind-check that asks a source for its current version.
    VersionCheck,
    /// A report sent to a collector the person turned on.
    Reporting,
}

/// One outbound call usher can make. One variant per call; no wildcard match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Egress {
    /// Fetch the signed harness-knowledge file or its signature.
    KnowledgeUpdate,
}

/// The HTTP method of a call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// A read.
    Get,
    /// A write.
    Post,
}

/// Where in the request a field travels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// A `{name}` placeholder in the endpoint's path template.
    Path,
    /// A request header.
    Header,
    /// A query-string field.
    Query,
    /// A body field.
    Body,
}

/// One field a call sends, with a one-line meaning for the profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field {
    /// The field's name.
    pub name: &'static str,
    /// Where it travels.
    pub place: Place,
    /// What it carries, in one line.
    pub meaning: &'static str,
}

/// Everything the profile says about one call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Descriptor {
    /// The call's name in kebab case: the variant's name, and the row id in the profile.
    pub name: &'static str,
    /// The HTTP method.
    pub method: Method,
    /// The https host. Never a scheme, never a port.
    pub host: &'static str,
    /// The path template, with `{name}` placeholders.
    pub path: &'static str,
    /// Why usher makes this call, in one line.
    pub purpose: &'static str,
    /// Which switch governs it.
    pub class: Class,
    /// Every field the call sends. The port refuses any other.
    pub fields: &'static [Field],
    /// Hosts a redirect may name. Any other redirect is an error.
    pub redirect_hosts: &'static [&'static str],
    /// The most bytes of answer body the port passes on. Required, and never zero.
    pub max_body_bytes: u64,
}

/// Every call, for the profile. A test asserts it names every [`Egress`] variant.
pub const ALL: &[Egress] = &[Egress::KnowledgeUpdate];

/// The name of the header every call carries, set by the port and never by a caller.
const USER_AGENT: &str = "user-agent";

/// The program name the port sends as `user-agent`: no version, no host detail.
const USER_AGENT_VALUE: &str = "usher";

const USER_AGENT_FIELD: Field = Field {
    name: USER_AGENT,
    place: Place::Header,
    meaning: "the program name, fixed; no version and nothing about the person or host",
};

const KNOWLEDGE_UPDATE: Descriptor = Descriptor {
    name: "knowledge-update",
    method: Method::Get,
    host: "knowledge.usher.invalid",
    path: "/{file}",
    purpose: "fetch the signed harness-knowledge file or its signature, when the person asks for an update",
    class: Class::PersonStarted,
    fields: &[
        USER_AGENT_FIELD,
        Field {
            name: "file",
            place: Place::Path,
            meaning: "which published file: the knowledge file or its signature",
        },
    ],
    redirect_hosts: &[],
    max_body_bytes: 1_048_576,
};

/// The descriptor of one call.
pub fn descriptor(egress: Egress) -> &'static Descriptor {
    match egress {
        Egress::KnowledgeUpdate => &KNOWLEDGE_UPDATE,
    }
}

/// The text every delegated installer carries in the profile.
pub const DELEGATED_NOTE: &str = "network access is the installer's, under its own settings";

/// An installer usher delegates to; its network use is its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelegatedInstaller {
    /// The installer's name as the profile lists it.
    pub name: &'static str,
    /// The fixed text for the profile.
    pub note: &'static str,
}

impl DelegatedInstaller {
    const fn named(name: &'static str) -> Self {
        Self {
            name,
            note: DELEGATED_NOTE,
        }
    }
}

/// The installers usher delegates to.
pub const DELEGATED_INSTALLERS: &[DelegatedInstaller] = &[
    DelegatedInstaller::named("brew"),
    DelegatedInstaller::named("mise"),
    DelegatedInstaller::named("uv"),
    DelegatedInstaller::named("npm"),
    DelegatedInstaller::named("git clone"),
    DelegatedInstaller::named("sideshow"),
    DelegatedInstaller::named("vendor installers"),
    DelegatedInstaller::named("usher update"),
];

/// A call as the caller builds it: a call kind and values for its fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    egress: Egress,
    values: Vec<(String, String)>,
}

impl Request {
    /// Starts a request for one call.
    pub fn new(egress: Egress) -> Self {
        Self {
            egress,
            values: Vec::new(),
        }
    }

    /// Sets the value of one named field.
    #[must_use]
    pub fn with(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.values.push((name.into(), value.into()));
        self
    }

    /// The call this request is for.
    pub fn egress(&self) -> Egress {
        self.egress
    }

    /// The values set so far, in order.
    pub fn values(&self) -> &[(String, String)] {
        &self.values
    }
}

/// A request as it goes to a [`Transport`]: checked, with nothing left to add
/// but protocol headers the client sets itself.
///
/// Nothing outside this crate can build one, so every request a transport sees
/// came through the port:
///
/// ```compile_fail
/// use usher_core::{Method, WireRequest};
///
/// let _ = WireRequest {
///     method: Method::Get,
///     host: String::new(),
///     path: String::new(),
///     query: Vec::new(),
///     headers: Vec::new(),
///     body: Vec::new(),
/// };
/// ```
///
/// A transport reads the request through its accessors:
///
/// ```
/// use usher_core::{
///     Egress, PersonStartedOnly, Port, Request, Response, Transport, UsherError, WireRequest,
/// };
///
/// struct Reader;
/// impl Transport for Reader {
///     fn send(&self, request: &WireRequest) -> Result<Response, UsherError> {
///         assert_eq!(request.host(), "knowledge.usher.invalid");
///         assert_eq!(request.path(), "/f.toml");
///         assert!(request.query().is_empty() && request.body().is_empty());
///         assert_eq!(request.headers()[0].0, "user-agent");
///         Ok(Response { status: 200, location: None, body: Vec::new() })
///     }
/// }
///
/// let port = Port::new(Reader, PersonStartedOnly);
/// port.call(Request::new(Egress::KnowledgeUpdate).with("file", "f.toml"))
///     .expect("the call goes through");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireRequest {
    /// The HTTP method.
    method: Method,
    /// The https host.
    host: String,
    /// The path with placeholders filled.
    path: String,
    /// The query fields, in descriptor order.
    query: Vec<(String, String)>,
    /// The headers, lowercase names, `user-agent` first.
    headers: Vec<(String, String)>,
    /// The body fields, in descriptor order.
    body: Vec<(String, String)>,
    /// The most bytes of answer body the descriptor allows; a transport reads one more.
    max_response_bytes: u64,
}

impl WireRequest {
    /// The HTTP method.
    pub fn method(&self) -> Method {
        self.method
    }

    /// The https host.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// The path with placeholders filled.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The query fields, in descriptor order.
    pub fn query(&self) -> &[(String, String)] {
        &self.query
    }

    /// The headers, lowercase names, `user-agent` first.
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }

    /// The most bytes of answer body the descriptor allows.
    pub fn max_response_bytes(&self) -> u64 {
        self.max_response_bytes
    }

    /// The body fields, in descriptor order.
    pub fn body(&self) -> &[(String, String)] {
        &self.body
    }
}

/// What came back: the status, any redirect target, and the body bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// The HTTP status.
    pub status: u16,
    /// The `Location` of a 3xx answer.
    pub location: Option<String>,
    /// The body.
    pub body: Vec<u8>,
}

/// Carries one checked request to the network and returns the answer. It
/// follows no redirect and adds no header beyond the protocol's own.
pub trait Transport {
    /// Sends one request.
    fn send(&self, request: &WireRequest) -> Result<Response, UsherError>;
}

mod sealed {
    /// Names the types allowed to implement [`super::Policy`]: only this crate's.
    pub trait Sealed {}
}

/// Decides whether a class of call may go out now.
///
/// Only this crate implements it, so a shipping crate cannot send a call that
/// the person's switches keep off:
///
/// ```compile_fail
/// use usher_core::{Class, Policy};
///
/// struct Permissive;
/// impl Policy for Permissive {
///     fn allows(&self, _class: Class) -> bool {
///         true
///     }
/// }
/// ```
///
/// The policy this crate ships allows person-started calls and nothing else:
///
/// ```
/// use usher_core::{Class, PersonStartedOnly, Policy};
///
/// assert!(PersonStartedOnly.allows(Class::PersonStarted));
/// assert!(!PersonStartedOnly.allows(Class::Reporting));
/// ```
pub trait Policy: sealed::Sealed {
    /// True when a call of this class may reach the transport.
    fn allows(&self, class: Class) -> bool;
}

/// A policy that refuses every class. Tests only.
#[cfg(feature = "test-seam")]
#[derive(Debug, Clone, Copy, Default)]
pub struct DenyAll;

#[cfg(feature = "test-seam")]
impl sealed::Sealed for DenyAll {}

#[cfg(feature = "test-seam")]
impl Policy for DenyAll {
    fn allows(&self, _class: Class) -> bool {
        false
    }
}

/// A policy that allows every class. Tests only.
#[cfg(feature = "test-seam")]
#[derive(Debug, Clone, Copy, Default)]
pub struct AllowAll;

#[cfg(feature = "test-seam")]
impl sealed::Sealed for AllowAll {}

#[cfg(feature = "test-seam")]
impl Policy for AllowAll {
    fn allows(&self, _class: Class) -> bool {
        true
    }
}

/// The Story 1.15 policy: person-started calls only.
#[derive(Debug, Clone, Copy, Default)]
pub struct PersonStartedOnly;

impl sealed::Sealed for PersonStartedOnly {}

impl Policy for PersonStartedOnly {
    fn allows(&self, class: Class) -> bool {
        class == Class::PersonStarted
    }
}

/// The one door every outbound call goes through.
#[derive(Debug)]
pub struct Port<T, P> {
    transport: T,
    policy: P,
}

impl<T: Transport, P: Policy> Port<T, P> {
    /// Builds a port over a transport and a policy.
    pub fn new(transport: T, policy: P) -> Self {
        Self { transport, policy }
    }

    /// Checks a request against its descriptor and the policy, sends it, and
    /// follows a redirect only to a host the descriptor lists.
    pub fn call(&self, request: Request) -> Result<Response, UsherError> {
        self.send_described(descriptor(request.egress()), request)
    }

    /// The same as [`Port::call`] with a descriptor the caller builds. Compiled
    /// only for tests, so shipping code cannot send with a hand-built one.
    #[cfg(feature = "test-seam")]
    pub fn call_described(
        &self,
        descriptor: &Descriptor,
        request: Request,
    ) -> Result<Response, UsherError> {
        self.send_described(descriptor, request)
    }

    fn send_described(
        &self,
        descriptor: &Descriptor,
        request: Request,
    ) -> Result<Response, UsherError> {
        if !self.policy.allows(descriptor.class) {
            return Err(refuse(
                descriptor,
                "the call's class may go out now",
                format!(
                    "{:?} calls are switched off, so nothing was sent to {}",
                    descriptor.class, descriptor.host
                ),
                "leave it off, or turn on the feature that owns this class",
            ));
        }
        let wire = build_wire(descriptor, &request)?;
        let first = self.transport.send(&wire)?;
        if !is_redirect(first.status) {
            return finish(descriptor, first);
        }
        if descriptor.method != Method::Get {
            return Err(refuse(
                descriptor,
                "a redirect is followed only for a read",
                format!(
                    "{} answered {} to a call that is not a GET; nothing was resent",
                    descriptor.host, first.status
                ),
                "ask the publisher for a direct address",
            ));
        }
        let target = redirect_target(descriptor, &first)?;
        let follow = WireRequest {
            method: Method::Get,
            host: target.0,
            path: target.1,
            query: Vec::new(),
            headers: vec![(USER_AGENT.to_string(), USER_AGENT_VALUE.to_string())],
            body: Vec::new(),
            max_response_bytes: 0,
        };
        let second = self.transport.send(&follow)?;
        if is_redirect(second.status) {
            return Err(refuse(
                descriptor,
                "an answer from a redirect host is a final answer",
                format!(
                    "{} answered {} with another redirect; the port follows one",
                    follow.host, second.status
                ),
                "ask the publisher for a direct address",
            ));
        }
        finish(descriptor, second)
    }
}

/// A refusal of a described call: the step names the call, so a person learns
/// which one failed from the step alone.
fn refuse(
    descriptor: &Descriptor,
    check: impl Into<String>,
    status: impl Into<String>,
    next: &str,
) -> UsherError {
    UsherError::new(format!("send {}", descriptor.name), check, status, next)
}

fn is_redirect(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

fn finish(descriptor: &Descriptor, response: Response) -> Result<Response, UsherError> {
    if response.body.len() as u64 > descriptor.max_body_bytes {
        return Err(refuse(
            descriptor,
            "the answer fits the descriptor's body limit",
            format!(
                "{} sent an answer larger than the limit of {} bytes",
                descriptor.host, descriptor.max_body_bytes
            ),
            "ask the publisher for a smaller file",
        ));
    }
    if (200..300).contains(&response.status) {
        return Ok(response);
    }
    Err(refuse(
        descriptor,
        "the host answers with a success status",
        format!("{} answered status {}", descriptor.host, response.status),
        "try again later, or check the address with the publisher",
    ))
}

const PATH_VALUE_CHECK: &str = "a path value holds no `/`, `?`, `#`, `%`, whitespace or control character, and is not `.` or `..`";

fn path_value_ok(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && !value
            .chars()
            .any(|c| matches!(c, '/' | '?' | '#' | '%') || c.is_whitespace() || c.is_control())
}

fn place_word(place: Place) -> &'static str {
    match place {
        Place::Path => "path",
        Place::Query => "query",
        Place::Header => "header",
        Place::Body => "body",
    }
}

/// Printable ASCII only (0x20 to 0x7E), with no space at either end.
fn header_value_problem(name: &str, value: &str) -> Option<String> {
    if !value.bytes().all(|b| (0x20..=0x7e).contains(&b)) {
        return Some(format!(
            "header {name} holds a character that is not printable ASCII"
        ));
    }
    if value.starts_with(' ') || value.ends_with(' ') {
        return Some(format!("header {name} starts or ends with a space"));
    }
    None
}

/// Percent-encodes every byte outside the RFC 3986 unreserved set, as upper-case
/// `%XX` of the value's UTF-8 bytes.
fn encode_query_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Checks every value against the descriptor and builds the request that goes out.
fn build_wire(descriptor: &Descriptor, request: &Request) -> Result<WireRequest, UsherError> {
    let mut seen: Vec<&str> = Vec::new();
    for (name, value) in request.values() {
        if name.eq_ignore_ascii_case(USER_AGENT) {
            return Err(refuse(
                descriptor,
                "the caller leaves the user-agent to the port",
                format!("field `{name}` is set by the port and cannot be given"),
                "remove the field from the request",
            ));
        }
        let Some(field) = descriptor.fields.iter().find(|f| f.name == name) else {
            return Err(refuse(
                descriptor,
                "every field sent is listed in the call's descriptor",
                format!("field `{name}` is not listed for {}", descriptor.host),
                "send only listed fields, or list the field in the registry and the profile",
            ));
        };
        if seen.contains(&name.as_str()) {
            return Err(refuse(
                descriptor,
                "each field is given once",
                format!("field `{name}` was given twice"),
                "give each field one value",
            ));
        }
        seen.push(name);
        if value.is_empty() {
            return Err(refuse(
                descriptor,
                "every field value is not empty",
                format!("{} {name} is empty", place_word(field.place)),
                "give the field a value",
            ));
        }
        match field.place {
            Place::Header => {
                if let Some(status) = header_value_problem(name, value) {
                    return Err(refuse(
                        descriptor,
                        "a header value holds only printable ASCII",
                        status,
                        "send a header value of printable ASCII only",
                    ));
                }
            }
            Place::Query if value.chars().any(char::is_control) => {
                return Err(refuse(
                    descriptor,
                    "a query value holds no control character",
                    format!("query {name} holds a control character"),
                    "send a query value without control characters",
                ));
            }
            _ => {}
        }
    }
    let value_of = |name: &str| {
        request
            .values()
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    };

    let mut path = descriptor.path.to_string();
    for field in descriptor.fields.iter().filter(|f| f.place == Place::Path) {
        let Some(value) = value_of(field.name) else {
            return Err(refuse(
                descriptor,
                "every path placeholder has a value",
                format!("field `{}` has no value", field.name),
                "give the field a value",
            ));
        };
        if !path_value_ok(value) {
            return Err(refuse(
                descriptor,
                PATH_VALUE_CHECK,
                format!("field `{}` holds a value the port refuses", field.name),
                "give the field a plain name with none of those characters",
            ));
        }
        path = path.replace(&format!("{{{}}}", field.name), value);
    }

    let pairs = |place: Place| -> Vec<(String, String)> {
        descriptor
            .fields
            .iter()
            .filter(|f| f.place == place && f.name != USER_AGENT)
            .filter_map(|f| {
                value_of(f.name).map(|v| {
                    let v = if place == Place::Query {
                        encode_query_value(v)
                    } else {
                        v.to_string()
                    };
                    (f.name.to_string(), v)
                })
            })
            .collect()
    };
    let mut headers = vec![(USER_AGENT.to_string(), USER_AGENT_VALUE.to_string())];
    headers.extend(pairs(Place::Header));
    Ok(WireRequest {
        method: descriptor.method,
        host: descriptor.host.to_string(),
        path,
        query: pairs(Place::Query),
        headers,
        body: pairs(Place::Body),
        max_response_bytes: 0,
    })
}

/// A host name is echoed in an error only when it is 1 to 253 characters of
/// letters, digits, dots and hyphens; anything else could carry terminal
/// escapes or log injection from a remote answer.
fn is_valid_host_name(host: &str) -> bool {
    (1..=253).contains(&host.len())
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
}

/// The host and path a redirect names, once the descriptor lists the host.
fn redirect_target(
    descriptor: &Descriptor,
    response: &Response,
) -> Result<(String, String), UsherError> {
    let Some(location) = response.location.as_deref() else {
        return Err(refuse(
            descriptor,
            "a redirect names where it goes",
            format!(
                "{} answered {} with no location",
                descriptor.host, response.status
            ),
            "ask the publisher for a direct address",
        ));
    };
    let Some(rest) = location.strip_prefix("https://") else {
        return Err(refuse(
            descriptor,
            "a redirect stays on https",
            format!("{} redirected to a non-https address", descriptor.host),
            "ask the publisher for an https address",
        ));
    };
    let (host, path) = rest.find('/').map_or((rest, "/"), |i| rest.split_at(i));
    if !is_valid_host_name(host) {
        return Err(refuse(
            descriptor,
            "a redirect goes only to a host the descriptor lists",
            format!(
                "{} redirected to a host that is not a valid name",
                descriptor.host
            ),
            "ask the publisher for a direct address",
        ));
    }
    if !descriptor.redirect_hosts.contains(&host) {
        return Err(refuse(
            descriptor,
            "a redirect goes only to a host the descriptor lists",
            format!(
                "{} redirected to host `{host}`, which is not listed",
                descriptor.host
            ),
            "ask the publisher for a direct address, or list the host in the registry and the profile",
        ));
    }
    if path
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '#')
    {
        return Err(refuse(
            descriptor,
            "a redirect path holds no whitespace, control character or fragment",
            format!("{host} gave a redirect path the port refuses"),
            "ask the publisher for a plain address",
        ));
    }
    Ok((host.to_string(), path.to_string()))
}
