use tracing::warn;
use std::future::Future;

use hudsucker::{hyper, certificate_authority::RcgenAuthority, rustls::{PrivateKey, Certificate}};
use hudsucker::async_trait::async_trait;

use crate::Config;
use crate::dispatcher::DISPATCHER;

pub const CA_CACHE_SIZE: u64 = 1000;

pub fn get_ca(config: &Config) -> RcgenAuthority {
    use secrecy::ExposeSecret;
    use rustls_pemfile as permfile; 

    let mut private_key_bytes: &[u8] = config.private_key.expose_secret().as_bytes().clone();
    let mut certificate_bytes: &[u8] = config.certificate.expose_secret().as_bytes().clone();

    let private_key = PrivateKey(
        permfile::pkcs8_private_keys(&mut private_key_bytes)
            .expect("failed to parse private key")
            .remove(0)
    );
    let ca_cert = Certificate(
        permfile::certs(&mut certificate_bytes)
            .expect("failed to parse CA certificate")
            .remove(0)
    );
    RcgenAuthority::new(private_key, ca_cert, CA_CACHE_SIZE)
        .expect("failed to create CA")
}

#[derive(Clone)]
pub struct DispatchHandler;

// Copied form hudsucker/src/proxy/internal.rs
fn normalize_request<T>(mut req: hyper::Request<T>) -> hyper::Request<T> {
    use hyper::header::Entry;

    // Hyper will automatically add a Host header if needed.
    req.headers_mut().remove(hyper::header::HOST);

    // HTTP/2 supports multiple cookie headers, but HTTP/1.x only supports one.
    if let Entry::Occupied(mut cookies) = req.headers_mut().entry(hyper::header::COOKIE) {
        let joined_cookies = bstr::join(b"; ", cookies.iter());
        cookies.insert(joined_cookies.try_into().expect("Failed to join cookies"));
    }

    *req.version_mut() = hyper::Version::HTTP_11;
    req
}

#[async_trait]
impl hudsucker::HttpHandler for DispatchHandler {
    async fn handle_request(
        &mut self,
        _ctx: &hudsucker::HttpContext,
        req: hyper::Request<hyper::Body>,
    ) -> hudsucker::RequestOrResponse {
        if req.method() == hyper::Method::CONNECT {
            return req.into();
        }
        DISPATCHER.dispatch_hyper(normalize_request(req))
            .await
            .map(|x| x.into())
            .unwrap_or_else(|error| {
                warn!(error = error.to_string(), "dispatch_hyper failed");
                hyper::Response::builder()
                    .status(500)
                    .body(error.to_string().into())
                    .unwrap()
                    .into()
            })
    }
}

pub fn start<F: Send + Future<Output = ()> + 'static>(config: &Config, shutdown_signal: F) {
    let ca = get_ca(config);
    let address = config.proxy_address().clone();
    tokio::task::spawn(
        async move {
            warn!(address = address.to_string(), "starting hudsucker proxy server");
            let proxy = hudsucker::ProxyBuilder::new()
                .with_addr(address)
                .with_rustls_client()
                .with_ca(ca)
                .with_http_handler(DispatchHandler)
                .build();
            proxy.start(shutdown_signal)
                .await
                .expect("failed to start hudsucker proxy server")
        }
    );
}