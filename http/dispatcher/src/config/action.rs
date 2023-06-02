use std::fmt::Display;

use secrecy::Secret;
use derivative::Derivative;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ProxyKind {
    Http,
    Https,
    Socks5,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Proxy {
    pub kind: ProxyKind,
    pub host: String,
    pub port: u16,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    pub username: Option<Secret<String>>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    pub password: Option<Secret<String>>, 
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Action {
    Drop,
    Pass,
    Proxy(Proxy),
}

impl ProxyKind {
    pub fn protocol(&self) -> String {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::Socks5 => "socks5",
        }.to_string()
    }

}

impl Display for ProxyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.protocol())
    }
}

impl Proxy {
    pub fn http(host: &str, port: u16) -> Proxy {
        Proxy {
            kind: ProxyKind::Http,
            host: host.to_string(),
            port,
            username: None,
            password: None,
        }
    }

    pub fn https(host: &str, port: u16) -> Proxy {
        Proxy {
            kind: ProxyKind::Https,
            host: host.to_string(),
            port,
            username: None,
            password: None,
        }
    }

    pub fn socks5(host: &str, port: u16) -> Proxy {
        Proxy {
            kind: ProxyKind::Socks5,
            host: host.to_string(),
            port,
            username: None,
            password: None,
        }
    }

    pub fn proxy_scheme(&self) -> String {
        format!("{}://{}:{}", self.kind.protocol(), self.host, self.port)
    }
}