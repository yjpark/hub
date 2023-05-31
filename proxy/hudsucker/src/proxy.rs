use tracing::warn;
use crate::Config;

pub struct Proxy {
    pub config: Config,
}

impl Proxy {
    pub fn new(config: Config) -> Self {
        Self {
            config,
        }
    }

    async fn shutdown_signal() {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
    }

    pub fn start(self) {
        tokio::task::spawn(
            async move {
                let ca = self.config.get_ca();
                let proxy = hudsucker::ProxyBuilder::new()
                    .with_addr(self.config.address())
                    .with_rustls_client()
                    .with_ca(ca)
                    .build();
                warn!(host = self.config.host, port = self.config.port, "starting hudsucker proxy server");
                proxy.start(Self::shutdown_signal())
                    .await
                    .unwrap();
            }
        );
    }
}