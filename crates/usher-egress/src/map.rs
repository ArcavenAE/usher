//! The two functions that touch the http client's own types: the plan to its request,
//! and the settings to its config. Nothing else in the crate names them.

use std::{sync::Arc, time::Duration};

use ureq::{
    config::Config,
    http::{self, Request},
    tls::{RootCerts, TlsConfig, TlsProvider},
};
use usher_core::{Method, UsherError};

use crate::{AgentSettings, RequestPlan};

/// The client's request for a plan: its method, address and headers as planned, in
/// order, and no body.
pub(crate) fn to_request(plan: &RequestPlan) -> Result<Request<()>, UsherError> {
    let method = match plan.method {
        Method::Get => http::Method::GET,
        Method::Post => http::Method::POST,
    };
    let mut builder = Request::builder().method(method).uri(plan.uri.as_str());
    for (name, value) in &plan.headers {
        builder = builder.header(name.as_str(), value.as_str());
    }
    builder.body(()).map_err(|err| {
        UsherError::new(
            "send request",
            "the request is well formed",
            err.to_string(),
            "report this; the call was not sent",
        )
    })
}

/// The client's config for the settings. The client's own conveniences that would send
/// something the port did not list are turned off: its automatic headers and the
/// proxy it would read from the environment.
pub(crate) fn to_config(settings: &AgentSettings) -> Config {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let tls = TlsConfig::builder()
        .provider(TlsProvider::Rustls)
        .root_certs(RootCerts::PlatformVerifier)
        .unversioned_rustls_crypto_provider(provider)
        .build();
    Config::builder()
        .timeout_connect(Some(Duration::from_secs(settings.connect_timeout_secs)))
        .timeout_global(Some(Duration::from_secs(settings.call_timeout_secs)))
        .max_redirects(settings.max_redirects)
        .https_only(settings.https_only)
        .http_status_as_error(settings.http_status_as_error)
        .user_agent(ureq::config::AutoHeaderValue::None)
        .accept_encoding(ureq::config::AutoHeaderValue::None)
        .proxy(None)
        .tls_config(tls)
        .build()
}

/// The settings the config carries, read back from it where the client has a getter.
#[cfg(test)]
fn read_back(config: &Config) -> AgentSettings {
    AgentSettings {
        connect_timeout_secs: config.timeouts().connect.map_or(0, |limit| limit.as_secs()),
        call_timeout_secs: config.timeouts().global.map_or(0, |limit| limit.as_secs()),
        max_redirects: config.max_redirects(),
        https_only: config.https_only(),
        http_status_as_error: config.http_status_as_error(),
        gzip: false,
        cookies: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_settings;

    fn plan() -> RequestPlan {
        RequestPlan {
            method: Method::Get,
            uri: "https://files.usher.invalid/list?name=a".to_string(),
            headers: vec![
                ("user-agent".to_string(), "usher".to_string()),
                ("x-trace".to_string(), "t1".to_string()),
                ("accept".to_string(), "text/plain".to_string()),
            ],
        }
    }

    #[test]
    fn the_mapped_request_has_the_plans_method_uri_and_headers_in_order_and_no_more() {
        let request = to_request(&plan()).expect("a request");
        assert_eq!(request.method(), http::Method::GET);
        assert_eq!(
            request.uri().to_string(),
            "https://files.usher.invalid/list?name=a"
        );
        let headers: Vec<(String, String)> = request
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_string(),
                    value.to_str().expect("text").to_string(),
                )
            })
            .collect();
        assert_eq!(headers, plan().headers);
    }

    #[test]
    fn a_post_plan_maps_to_a_post_with_no_body() {
        let mut posting = plan();
        posting.method = Method::Post;
        let request = to_request(&posting).expect("a request");
        assert_eq!(request.method(), http::Method::POST);
    }

    #[test]
    fn the_mapped_config_carries_each_setting_the_client_lets_it_read_back() {
        let config = to_config(&agent_settings());
        assert_eq!(read_back(&config), agent_settings());
    }

    #[test]
    fn the_mapped_config_adds_no_header_of_the_clients_own_and_reads_no_proxy() {
        let config = to_config(&agent_settings());
        assert!(matches!(
            config.user_agent(),
            ureq::config::AutoHeaderValue::None
        ));
        assert!(matches!(
            config.accept_encoding(),
            ureq::config::AutoHeaderValue::None
        ));
        assert!(config.proxy().is_none(), "no proxy from the environment");
    }

    #[test]
    fn the_mapped_config_verifies_with_the_platform_and_carries_the_ring_provider() {
        let config = to_config(&agent_settings());
        let tls = config.tls_config();
        assert!(matches!(tls.root_certs(), RootCerts::PlatformVerifier));
        assert_eq!(tls.provider(), TlsProvider::Rustls);
        assert!(
            tls.unversioned_rustls_crypto_provider().is_some(),
            "the provider is set on this agent, not process wide"
        );
    }
}
