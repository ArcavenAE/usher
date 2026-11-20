//! Running a harness with the network denied except localhost.

use std::{io, path::Path, process::Output};

use usher_testkit::ScratchHome;

use crate::fragment::Network;

const SANDBOX_EXEC: &str = "/usr/bin/sandbox-exec";

/// The sandbox profile: deny the network, allow localhost, and nothing else.
///
/// Later rules win, so the localhost allows follow the one deny. Nothing else
/// is changed: the run keeps every other permission it would have had.
pub fn profile() -> String {
    [
        "(version 1)",
        "(allow default)",
        "(deny network*)",
        r#"(allow network* (remote ip "localhost:*"))"#,
        r#"(allow network-bind (local ip "localhost:*"))"#,
        r#"(allow network-inbound (local ip "localhost:*"))"#,
    ]
    .join("\n")
        + "\n"
}

/// Runs `program` under the profile, in the scratch home, and returns its
/// output. Only hosts with `sandbox-exec` can do this.
pub fn run_denied(scratch: &ScratchHome, program: &str, args: &[&str]) -> io::Result<Output> {
    if !Path::new(SANDBOX_EXEC).exists() {
        return Err(io::Error::from(io::ErrorKind::Unsupported));
    }
    scratch
        .command(SANDBOX_EXEC)
        .arg("-p")
        .arg(profile())
        .arg(program)
        .args(args)
        .output()
}

/// The state a probe on this host records: denied where the sandbox exists,
/// not denied elsewhere.
pub fn host_state() -> Network {
    if cfg!(target_os = "macos") && Path::new(SANDBOX_EXEC).exists() {
        Network::Denied
    } else {
        Network::NotDenied
    }
}
