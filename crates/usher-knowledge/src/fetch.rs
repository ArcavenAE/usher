//! The first caller of the egress port: fetch a signed knowledge update.
//!
//! The file and its signature come through the port, and then
//! [`Knowledge::update`] decides, offline, whether to take them. Fetching adds
//! no trust of its own: a file that did not verify leaves the knowledge in use
//! unchanged.

use usher_core::{Policy, Port, Transport, UsherError};

use crate::{Knowledge, KnowledgeVersion, TrustedKeys};

/// Fetches the signed knowledge file and its signature through the port and
/// takes the update if it verifies.
pub fn fetch_update<T: Transport, P: Policy>(
    port: &Port<T, P>,
    knowledge: &mut Knowledge,
    trusted: &TrustedKeys,
) -> Result<KnowledgeVersion, UsherError> {
    let _ = (port, knowledge, trusted);
    Err(UsherError::new(
        "fetch harness knowledge",
        "the fetch is built",
        "not built",
        "build the knowledge fetch",
    ))
}
