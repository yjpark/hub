//! Peer hubs: each hub dials a static list of named peers and reaches their
//! nodes one hop away, as `<peer>::<node>`.
//!
//! implements: a297d880b240fa86f9872753ebc3009c0d3af0dd572ec9eaa2a17a3f815f8d45@a297d880b240fa86f9872753ebc3009c0d3af0dd572ec9eaa2a17a3f815f8d45

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use hub_protocol::service::ConsumerApiClient;
use vox::ConnectionHandle;

/// Opens a connection to one peer hub.
pub type Dialer = Arc<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<ConnectionHandle, String>> + Send>>
        + Send
        + Sync,
>;

/// This hub's name and its peers (stub).
#[derive(Clone, Default)]
pub struct Peers {
    name: Option<String>,
}

impl Peers {
    /// A hub named `name`, with no peers yet.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
        }
    }

    /// This hub's name, when it has one.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Dial `peer` with `dialer`, and redial whenever it drops (stub).
    pub fn add(&self, peer: impl Into<String>, dialer: Dialer) {
        let _ = (peer.into(), dialer);
    }

    /// The peers' names, configured order (stub).
    pub fn names(&self) -> Vec<String> {
        Vec::new()
    }

    /// The live connection to `peer`, if it is up (stub).
    pub async fn client(&self, peer: &str) -> Option<ConsumerApiClient> {
        let _ = peer;
        None
    }
}

/// A dialer for a peer hub at `addr` over TCP (stub).
pub fn tcp_dialer(addr: String) -> Dialer {
    Arc::new(move || {
        let addr = addr.clone();
        Box::pin(async move { Err(format!("not dialing {addr}")) })
    })
}

/// `HUB_PEERS`: `name=addr,…` (stub).
pub fn parse_peers(spec: &str) -> Result<Vec<(String, String)>, String> {
    let _ = spec;
    Ok(Vec::new())
}
