//! Running a harness with the network denied except localhost.

use std::{io, process::Output};

use usher_testkit::ScratchHome;

use crate::fragment::Network;

/// The sandbox profile: deny the network, allow localhost, and nothing else.
pub fn profile() -> String {
    String::new()
}

/// Runs `program` under the profile, in the scratch home, and returns its
/// output. Only hosts with `sandbox-exec` can do this.
pub fn run_denied(_scratch: &ScratchHome, _program: &str, _args: &[&str]) -> io::Result<Output> {
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

/// The state a probe on this host records: denied where the sandbox exists,
/// not denied elsewhere.
pub fn host_state() -> Network {
    Network::NotDenied
}
