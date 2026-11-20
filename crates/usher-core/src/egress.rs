//! The egress port (AD-14, Story 1.15): the one place an outbound call is
//! described and checked.
//!
//! Every call usher makes is one variant of [`Egress`]. Its [`Descriptor`] says
//! where it goes, why, in which class, and every field it sends. The
//! [`Port`] takes only an [`Egress`] value, so a call with no descriptor does
//! not compile. The port holds no HTTP client: a [`Transport`] does, and the
//! one real transport lives outside this crate.
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
}

/// Every call, for the profile. A test asserts it names every [`Egress`] variant.
pub const ALL: &[Egress] = &[];

/// The descriptor of one call.
pub fn descriptor(egress: Egress) -> &'static Descriptor {
    match egress {
        Egress::KnowledgeUpdate => &PLACEHOLDER,
    }
}

const PLACEHOLDER: Descriptor = Descriptor {
    method: Method::Get,
    host: "",
    path: "",
    purpose: "",
    class: Class::PersonStarted,
    fields: &[],
    redirect_hosts: &[],
};

/// The text every delegated installer carries in the profile.
pub const DELEGATED_NOTE: &str = "";

/// An installer usher delegates to; its network use is its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelegatedInstaller {
    /// The installer's name as the profile lists it.
    pub name: &'static str,
    /// The fixed text for the profile.
    pub note: &'static str,
}

/// The installers usher delegates to.
pub const DELEGATED_INSTALLERS: &[DelegatedInstaller] = &[];

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireRequest {
    /// The HTTP method.
    pub method: Method,
    /// The https host.
    pub host: String,
    /// The path with placeholders filled.
    pub path: String,
    /// The query fields, in descriptor order.
    pub query: Vec<(String, String)>,
    /// The headers, lowercase names, `user-agent` first.
    pub headers: Vec<(String, String)>,
    /// The body fields, in descriptor order.
    pub body: Vec<(String, String)>,
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

/// Decides whether a class of call may go out now.
pub trait Policy {
    /// True when a call of this class may reach the transport.
    fn allows(&self, class: Class) -> bool;
}

/// The Story 1.15 policy: person-started calls only.
#[derive(Debug, Clone, Copy, Default)]
pub struct PersonStartedOnly;

impl Policy for PersonStartedOnly {
    fn allows(&self, _class: Class) -> bool {
        false
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
        _descriptor: &Descriptor,
        request: Request,
    ) -> Result<Response, UsherError> {
        let _ = (&self.transport, &self.policy, request);
        Err(UsherError::new(
            "egress",
            "the port is built",
            "not built",
            "build the egress port",
        ))
    }
}
