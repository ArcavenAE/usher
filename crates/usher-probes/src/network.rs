//! Running a harness with the network denied: the profile, the host state a
//! fragment records, and the argument vector a denied start runs with. This
//! file starts nothing; the launcher's one spawn site runs the vector.

use std::path::Path;

use crate::fragment::Network;

const SANDBOX_EXEC: &str = "/usr/bin/sandbox-exec";

/// The sandbox profile: deny the network and allow nothing.
///
/// Every other permission the run would have had is kept. A probe that needs
/// a local port declares it, and [`profile_with_ports`] allows that port only.
pub fn profile() -> String {
    ["(version 1)", "(allow default)", "(deny network*)"].join("\n") + "\n"
}

/// The sandbox profile for a run that may reach exactly `ports` on localhost.
///
/// Later rules win, so each allow follows the one deny.
pub fn profile_with_ports(ports: &[u16]) -> String {
    let mut text = profile();
    for port in ports {
        text.push_str(&format!(
            "(allow network* (remote ip \"localhost:{port}\"))\n\
             (allow network-bind (local ip \"localhost:{port}\"))\n\
             (allow network-inbound (local ip \"localhost:{port}\"))\n"
        ));
    }
    text
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

/// The argument vector that runs `program` with `args` under the profile for
/// `ports`: the sandbox tool, `-p`, the profile, the program, then its
/// arguments. It starts nothing; the launcher runs it.
pub fn denied_argv(ports: &[u16], program: &str, args: &[&str]) -> Vec<String> {
    [SANDBOX_EXEC, "-p"]
        .into_iter()
        .map(str::to_string)
        .chain(std::iter::once(profile_with_ports(ports)))
        .chain(std::iter::once(program.to_string()))
        .chain(args.iter().map(|arg| (*arg).to_string()))
        .collect()
}
