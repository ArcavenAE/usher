//! The network-denied run: the profile's form, the host state a fragment
//! records, and, on macOS, the refusal and the localhost control measured.

use usher_probes::{
    fragment::Network,
    network::{denied_argv, host_state, profile, profile_with_ports},
};

// Meaning change: this was the_profile_denies_the_network_first_then_allows_localhost.
// A blanket localhost allow lets a probe reach any local service, a forwarding
// proxy included, so the profile now denies all network and allows nothing; a
// probe that needs a local port declares it.
#[test]
fn the_profile_denies_all_network_and_allows_nothing() {
    let text = profile();
    assert!(text.starts_with("(version 1)"), "{text}");
    assert_eq!(
        text.matches("(deny network*)").count(),
        1,
        "one network deny:\n{text}"
    );
    assert!(
        !text.contains("(allow network"),
        "the profile allows some network operation:\n{text}"
    );
    assert!(!text.contains("localhost"), "localhost is named:\n{text}");
}

#[test]
fn the_profile_has_one_network_deny_and_no_local_ip_allow_of_every_network() {
    let text = profile();
    assert_eq!(
        text.matches("(deny network*)").count(),
        1,
        "exactly one network deny, at the start:\n{text}"
    );
    assert!(
        !text.contains("(allow network* (local ip"),
        "a local-ip allow of every network operation lets other hosts through:\n{text}"
    );
}

#[test]
fn a_declared_port_is_allowed_on_localhost_and_nothing_else_is() {
    assert_eq!(
        profile_with_ports(&[]),
        profile(),
        "no declared port means the plain profile"
    );
    let text = profile_with_ports(&[8080, 9090]);
    let deny = text.find("(deny network*)").expect("a network deny");
    for port in [8080, 9090] {
        for rule in [
            format!(r#"(allow network* (remote ip "localhost:{port}"))"#),
            format!(r#"(allow network-bind (local ip "localhost:{port}"))"#),
            format!(r#"(allow network-inbound (local ip "localhost:{port}"))"#),
        ] {
            let at = text
                .find(&rule)
                .unwrap_or_else(|| panic!("missing {rule}:\n{text}"));
            assert!(deny < at, "the deny must come first:\n{text}");
        }
    }
    assert_eq!(text.matches("(deny network*)").count(), 1, "{text}");
    assert!(
        !text.contains("localhost:*"),
        "a wildcard port is allowed:\n{text}"
    );
    assert!(
        !text.contains("localhost:7070"),
        "an undeclared port is allowed:\n{text}"
    );
}

// The crate has one spawn site, the launcher's run. A denied run is an
// argument vector the launcher runs: the sandbox tool, -p, the profile, the
// program, then its arguments. Building it starts nothing and needs no sandbox
// on the host.
#[test]
fn a_denied_run_is_an_exact_argv_that_starts_nothing() {
    let args = ["a", "b c", "--flag=1"];
    let plain: Vec<String> = [
        "/usr/bin/sandbox-exec",
        "-p",
        profile().as_str(),
        "/bin/echo",
    ]
    .into_iter()
    .chain(args)
    .map(str::to_string)
    .collect();
    assert_eq!(denied_argv(&[], "/bin/echo", &args), plain);

    let ported: Vec<String> = [
        "/usr/bin/sandbox-exec",
        "-p",
        profile_with_ports(&[8080, 9090]).as_str(),
        "/usr/bin/nc",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(denied_argv(&[8080, 9090], "/usr/bin/nc", &[]), ported);
}

#[cfg(target_os = "macos")]
mod macos {
    use std::{io, net::TcpListener, process::Output};

    use usher_probes::network::denied_argv;
    use usher_testkit::ScratchHome;

    use super::*;

    // Meaning change (Q8): run_denied and run_denied_with_ports no longer
    // start a process. The tests build the same argv and run it in the scratch
    // home themselves, so each run is the one the profile refuses or allows.
    fn run_argv(scratch: &ScratchHome, argv: &[String]) -> io::Result<Output> {
        scratch.command(&argv[0]).args(&argv[1..]).output()
    }

    fn run_denied(scratch: &ScratchHome, program: &str, args: &[&str]) -> io::Result<Output> {
        run_argv(scratch, &denied_argv(&[], program, args))
    }

    fn run_denied_with_ports(
        scratch: &ScratchHome,
        ports: &[u16],
        program: &str,
        args: &[&str],
    ) -> io::Result<Output> {
        run_argv(scratch, &denied_argv(ports, program, args))
    }

    #[test]
    fn a_macos_host_records_the_network_as_denied() {
        assert_eq!(host_state(), Network::Denied);
    }

    #[test]
    fn a_connect_to_a_non_local_address_is_refused_by_the_profile_not_the_network() {
        let scratch = ScratchHome::new().expect("scratch home");
        let args = ["-v", "-z", "-w", "1", "-G", "1", "192.0.2.1", "80"];

        // Outside half: the same command without the profile. TEST-NET-1 is
        // unroutable, so it times out or is unreachable; it is never refused
        // by a sandbox.
        let outside = scratch
            .command("/usr/bin/nc")
            .args(args)
            .output()
            .expect("nc runs without the profile");
        let outside_err = String::from_utf8_lossy(&outside.stderr);
        assert!(
            outside_err.contains("failed:"),
            "control: nc reports its failure, so an absent denial is a real absence: {outside_err}"
        );
        assert!(
            !outside_err.contains("Operation not permitted"),
            "control: without the profile the connect is not refused by a sandbox: {outside_err}"
        );

        // Inside: the profile's own denial, EPERM, and not any failure.
        let inside =
            run_denied(&scratch, "/usr/bin/nc", &args).expect("the run starts under the profile");
        let inside_err = String::from_utf8_lossy(&inside.stderr);
        assert!(!inside.status.success());
        assert!(
            inside_err.contains("Operation not permitted"),
            "{inside_err}"
        );
    }

    // Meaning change: this was a_connect_to_a_local_listener_still_works_under_the_profile.
    // Localhost is no longer allowed as a whole; a port the probe did not
    // declare is refused by the profile itself.
    #[test]
    fn a_connect_to_an_undeclared_local_port_is_refused_with_eperm() {
        let scratch = ScratchHome::new().expect("scratch home");
        let listener = TcpListener::bind("127.0.0.1:0").expect("a local listener");
        let port = listener
            .local_addr()
            .expect("its address")
            .port()
            .to_string();
        let args = ["-v", "-z", "-w", "1", "127.0.0.1", port.as_str()];

        // Outside the profile the listener accepts, so a refusal inside is the profile's.
        let outside = scratch
            .command("/usr/bin/nc")
            .args(args)
            .output()
            .expect("nc runs without the profile");
        assert!(
            outside.status.success(),
            "control: the listener must accept outside the profile: {}",
            String::from_utf8_lossy(&outside.stderr)
        );

        let inside =
            run_denied(&scratch, "/usr/bin/nc", &args).expect("the run starts under the profile");
        let stderr = String::from_utf8_lossy(&inside.stderr);
        assert!(!inside.status.success(), "an undeclared port was reachable");
        assert!(stderr.contains("Operation not permitted"), "{stderr}");
    }

    #[test]
    fn a_declared_local_port_is_reachable_and_another_is_not() {
        let scratch = ScratchHome::new().expect("scratch home");
        let declared = TcpListener::bind("127.0.0.1:0").expect("a local listener");
        let other = TcpListener::bind("127.0.0.1:0").expect("a second listener");
        let declared_port = declared.local_addr().expect("address").port();
        let other_port = other.local_addr().expect("address").port().to_string();

        let allowed = run_denied_with_ports(
            &scratch,
            &[declared_port],
            "/usr/bin/nc",
            &[
                "-v",
                "-z",
                "-w",
                "1",
                "127.0.0.1",
                &declared_port.to_string(),
            ],
        )
        .expect("the run starts under the profile");
        assert!(
            allowed.status.success(),
            "the declared port was refused: {}",
            String::from_utf8_lossy(&allowed.stderr)
        );

        let refused = run_denied_with_ports(
            &scratch,
            &[declared_port],
            "/usr/bin/nc",
            &["-v", "-z", "-w", "1", "127.0.0.1", other_port.as_str()],
        )
        .expect("the run starts under the profile");
        assert!(
            String::from_utf8_lossy(&refused.stderr).contains("Operation not permitted"),
            "{}",
            String::from_utf8_lossy(&refused.stderr)
        );
    }

    #[test]
    fn the_denied_run_uses_the_scratch_home() {
        let scratch = ScratchHome::new().expect("scratch home");
        let output = run_denied(&scratch, "/bin/sh", &["-c", "printf %s \"$HOME\""])
            .expect("the run starts under the profile");
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            scratch.home().to_string_lossy()
        );
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn a_host_without_the_sandbox_records_the_network_as_not_denied() {
    assert_eq!(host_state(), Network::NotDenied);
}
