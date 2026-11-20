//! The network-denied run: the profile's form, the host state a fragment
//! records, and, on macOS, the refusal and the localhost control measured.

use usher_probes::{
    fragment::Network,
    network::{host_state, profile},
};

#[test]
fn the_profile_denies_the_network_first_then_allows_localhost() {
    let text = profile();
    assert!(text.starts_with("(version 1)"), "{text}");
    let deny = text.find("(deny network*)").expect("a network deny");
    let allow = text
        .find(r#"(allow network* (remote ip "localhost:*"))"#)
        .expect("a localhost allow");
    assert!(deny < allow, "the deny must come first:\n{text}");
    assert!(text.contains(r#"(allow network-bind (local ip "localhost:*"))"#));
    assert!(text.contains(r#"(allow network-inbound (local ip "localhost:*"))"#));
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

#[cfg(target_os = "macos")]
mod macos {
    use std::net::TcpListener;

    use usher_probes::network::run_denied;
    use usher_testkit::ScratchHome;

    use super::*;

    #[test]
    fn a_macos_host_records_the_network_as_denied() {
        assert_eq!(host_state(), Network::Denied);
    }

    #[test]
    fn a_connect_to_a_non_local_address_is_refused_by_the_profile_not_the_network() {
        let scratch = ScratchHome::new().expect("scratch home");
        let args = ["-z", "-w", "1", "-G", "1", "192.0.2.1", "80"];

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

    #[test]
    fn a_connect_to_a_local_listener_still_works_under_the_profile() {
        let scratch = ScratchHome::new().expect("scratch home");
        let listener = TcpListener::bind("127.0.0.1:0").expect("a local listener");
        let port = listener
            .local_addr()
            .expect("its address")
            .port()
            .to_string();
        let output = run_denied(
            &scratch,
            "/usr/bin/nc",
            &["-z", "-w", "1", "127.0.0.1", &port],
        )
        .expect("the run starts under the profile");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
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
