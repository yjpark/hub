use snafu::prelude::*;
use dashmap::DashMap;
use reqwest::Client;
use reqwest::ClientBuilder;

use crate::Config;
use crate::config::Action;
use crate::config::Proxy;
use crate::config::ProxyKind;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("no action for uri: {}", uri))]
    NoActionUri { uri: http::Uri },
    #[snafu(display("no action for url: {}", url))]
    NoActionUrl { url: reqwest::Url },
    #[snafu(display("parse url failed: {} -> {}", str, source))]
    ParseUrlFailed { str: String, source: url::ParseError },
    #[snafu(display("reqwest failed for uri: {} -> {}", uri, source))]
    ReqwestFailedUri { uri: http::Uri, source: reqwest::Error },
    #[snafu(display("reqwest failed for url: {} -> {}", url, source))]
    ReqwestFailedUrl { url: reqwest::Url, source: reqwest::Error },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Dispatcher {
    pub config: Config,
    pub clients: DashMap<Action, Client>,
}

fn create_proxy_client(proxy: &Proxy) -> Client {
    let reqwest_proxy = reqwest::Proxy::all(proxy.proxy_scheme())
        .expect(format!("invalid proxy: {:?}", proxy).as_str());
    let reqwest_proxy =  if let (Some(username), Some(password)) = (&proxy.username, &proxy.password) {
        if proxy.kind == ProxyKind::Socks5 {
            tracing::error!(proxy = proxy.proxy_scheme(), "proxy auth not supported for ProxyKind::Socks5");
            reqwest_proxy
        } else {
            use secrecy::ExposeSecret;
            reqwest_proxy.basic_auth(username.expose_secret(), password.expose_secret())
        }
    } else {
        reqwest_proxy
    };
    ClientBuilder::new()
        .proxy(reqwest_proxy)
        .build()
        .expect(format!("failed to create reqwest client with proxy: {:?}", proxy).as_str())
}

fn create_clients(config: &Config) -> DashMap<Action, Client> {
    let clients = DashMap::new();
    clients.insert(Action::Pass,
        ClientBuilder::new()
            .no_proxy()
            .build()
            .expect("failet to create reqwest client with no proxy")
    );
    for proxy in config.proxies.iter() {
        clients.insert(Action::Proxy(proxy.clone()), create_proxy_client(proxy));
    }
    clients
}

impl Dispatcher {
    pub fn new(config: Config) -> Dispatcher {
        let clients = create_clients(&config);
        Dispatcher {
            config,
            clients,
        }
    }

    pub fn get_client_uri(&self, uri: &http::Uri) -> Option<Client> {
        let action = self.config.action_uri(uri);
        tracing::warn!(action = format!("{:?}", action), "USING ACTION");
        self.clients.get(action)
            .map(|x| x.value().clone())
    }

    pub fn get_client_url(&self, url: &reqwest::Url) -> Option<Client> {
        let action = self.config.action_url(url);
        self.clients.get(action)
            .map(|x| x.value().clone())
    }

    pub async fn dispatch(&self, request: reqwest::Request) -> Result<reqwest::Response> {
        let url = request.url().clone();
        let Some(client) = self.get_client_url(&url) else {
            return Err(Error::NoActionUrl { url })
        };
        client
            .execute(request)
            .await
            .context(ReqwestFailedUrlSnafu { url })
    }

    #[cfg(feature = "hyper")]
    pub async fn dispatch_hyper(&self, request: hyper::Request<hyper::Body>) -> Result<hyper::Response<hyper::Body>> {
        let uri = request.uri().clone();
        let Some(client) = self.get_client_uri(&uri) else {
            return Err(Error::NoActionUri { uri });
        };
        let str = uri.to_string();
        let url = reqwest::Url::parse(&str)
            .context(ParseUrlFailedSnafu { str })?;
        let method = request.method();
        let mut req = reqwest::Request::new(method.clone(), url);
        *req.headers_mut() = request.headers().clone();
        *req.version_mut() = request.version().clone();
        let (_parts, body) = request.into_parts();
        *req.body_mut() = Some(body.into());
        let res = client
            .execute(req)
            .await
            .context(ReqwestFailedUriSnafu { uri: uri.clone() })?;
        let status = res.status();
        let version = res.version();
        let headers = res.headers().clone();
        let body = res.bytes()
            .await
            .context(ReqwestFailedUriSnafu { uri })?;
        let mut response = hyper::Response::new(body.into());
        *response.status_mut() = status;
        *response.version_mut() = version;
        *response.headers_mut() = headers;
        Ok(response)
    }
}
