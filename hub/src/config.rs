use tracing::{warn, info};
use secrecy::Secret;

#[derive(Debug, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub proxy_port: u16,
    #[serde(skip_serializing)]
    pub private_key: Secret<String>,
    #[serde(skip_serializing)]
    pub certificate: Secret<String>,
}

impl secrecy::SerializableSecret for Config {}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3300,
            proxy_port: 3301,
            private_key: "AA".to_string().into(),
            certificate: "BB".to_string().into(),
        }
    }
}

impl Config {
    pub const ENV_PREFIX: &'static str = "HUB_";
    pub const ENV_PRIVATE_KEY: &'static str = "HUB_PRIVATE_KEY";
    pub const ENV_CERTIFICATE: &'static str = "HUB_CERTIFICATE";

    pub fn address(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from((self.host.parse::<std::net::IpAddr>().expect("invalid host"), self.port))
    }

    pub fn proxy_address(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from((self.host.parse::<std::net::IpAddr>().expect("invalid host"), self.proxy_port))
    }

    pub fn load(args: &crate::Args) -> Self {
        use figment::Figment;
        use figment::providers::{Format, Toml, Env, Serialized};
        let path = args.config_path();
        if path.exists() {
            info!(path = path.to_str(), "loading config from path");
        } else {
            warn!(path = path.to_str(), "config file not exist");
        }
        args.inject_envs();
        Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Toml::file(&path))
            .merge(Env::prefixed(Self::ENV_PREFIX))
            .merge(Serialized::defaults(args))
            .extract()
            .unwrap()
    }
}