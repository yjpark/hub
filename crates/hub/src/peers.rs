//! Peer hubs: each hub dials a static list of named peers and reaches their
//! nodes one hop away, as `<peer>::<node>`.
//!
//! A peer is dialled as an ordinary consumer of the other hub: its own nodes
//! come from `local_topology`, calls go through `route` with the name
//! stripped, and events through `subscribe_local_events` — the two `local`
//! methods are what keep everything one hop.
//!
//! implements: a297d880b240fa86f9872753ebc3009c0d3af0dd572ec9eaa2a17a3f815f8d45@a297d880b240fa86f9872753ebc3009c0d3af0dd572ec9eaa2a17a3f815f8d45

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use hub_protocol::service::ConsumerApiClient;
use hub_protocol::{HubEvent, NodeInfo, PEER_SEPARATOR, SUBSCRIBED_TOPIC, TopologySnapshot};
use tokio::sync::mpsc;
use vox::ConnectionHandle;

/// Opens a connection to one peer hub.
pub type Dialer = Arc<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<ConnectionHandle, String>> + Send>>
        + Send
        + Sync,
>;

/// How long a peer's topology may take before its nodes are left out.
const TOPOLOGY_WAIT: Duration = Duration::from_secs(2);
/// The longest wait between redials of a peer that is down.
const REDIAL_MAX: Duration = Duration::from_secs(30);

struct Peer {
    name: String,
    client: Option<ConsumerApiClient>,
}

/// This hub's name and its peers, each redialled forever.
#[derive(Clone, Default)]
pub struct Peers {
    name: Option<String>,
    peers: Arc<Mutex<Vec<Peer>>>,
}

impl Peers {
    /// A hub named `name`, with no peers yet.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            peers: Arc::default(),
        }
    }

    /// This hub's name, when it has one.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn with<R>(&self, f: impl FnOnce(&mut Vec<Peer>) -> R) -> R {
        let mut peers = self
            .peers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(&mut peers)
    }

    fn set(&self, peer: &str, client: Option<ConsumerApiClient>) {
        self.with(|peers| {
            if let Some(p) = peers.iter_mut().find(|p| p.name == peer) {
                p.client = client;
            }
        });
    }

    /// Dial `peer` with `dialer`, and redial whenever it drops, backing off
    /// up to thirty seconds while it is down. Needs a tokio runtime.
    pub fn add(&self, peer: impl Into<String>, dialer: Dialer) -> Result<(), String> {
        let peer = peer.into();
        self.with(|peers| {
            peers.push(Peer {
                name: peer.clone(),
                client: None,
            })
        });
        let peers = self.clone();
        tokio::spawn(async move {
            let mut wait = Duration::from_secs(1);
            loop {
                if let Ok(handle) = dialer().await {
                    let lane: Result<ConsumerApiClient, _> = handle.open_lane().await;
                    if let Ok(client) = lane {
                        peers.set(&peer, Some(client));
                        wait = Duration::from_secs(1);
                        handle.closed().await;
                        peers.set(&peer, None);
                    }
                }
                tokio::time::sleep(wait).await;
                wait = (wait * 2).min(REDIAL_MAX);
            }
        });
        Ok(())
    }

    /// The peers' names, in the order they were added.
    pub fn names(&self) -> Vec<String> {
        self.with(|peers| peers.iter().map(|p| p.name.clone()).collect())
    }

    /// The live connection to `peer`, if it is up.
    pub async fn client(&self, peer: &str) -> Option<ConsumerApiClient> {
        self.with(|peers| {
            peers
                .iter()
                .find(|p| p.name == peer)
                .and_then(|p| p.client.clone())
        })
    }

    /// Every reachable peer's own nodes, qualified by the peer's name; a peer
    /// that is down or slower than two seconds is left out.
    pub async fn nodes(&self) -> Vec<NodeInfo> {
        let mut nodes = Vec::new();
        for peer in self.names() {
            let Some(client) = self.client(&peer).await else {
                continue;
            };
            if let Ok(Ok(TopologySnapshot { nodes: theirs })) =
                tokio::time::timeout(TOPOLOGY_WAIT, client.local_topology()).await
            {
                nodes.extend(theirs.into_iter().map(|n| NodeInfo {
                    node: hub_protocol::NodeId::qualified(&peer, &n.node),
                    links: n.links,
                }));
            }
        }
        nodes
    }

    /// Carry `peer`'s own events under `prefix` into one subscriber's
    /// `queue` while it lives, resubscribing whenever the peer drops. Events
    /// enter that queue only — never this hub's publish — so they cross one
    /// hop and are never sent on.
    pub async fn follow(&self, peer: String, prefix: String, queue: mpsc::Sender<HubEvent>) {
        loop {
            if let Some(client) = self.client(&peer).await {
                let (tx, mut rx) = vox::channel::<HubEvent>();
                let call = tokio::spawn({
                    let prefix = prefix.clone();
                    async move {
                        let _ = client.subscribe_local_events(prefix, tx).await;
                    }
                });
                loop {
                    tokio::select! {
                        got = rx.recv() => match got {
                            Ok(Some(event)) => {
                                let mut owned = None;
                                let _ = event.map(|e| owned = Some(e));
                                let Some(event) = owned else { continue };
                                if event.topic == SUBSCRIBED_TOPIC {
                                    continue;
                                }
                                if let Err(mpsc::error::TrySendError::Closed(_)) =
                                    queue.try_send(event)
                                {
                                    call.abort();
                                    return;
                                }
                            }
                            _ => break,
                        },
                        _ = queue.closed() => {
                            call.abort();
                            return;
                        }
                    }
                }
                call.abort();
            }
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                _ = queue.closed() => return,
            }
        }
    }
}

/// A dialer for a peer hub at `addr` over TCP (three seconds to connect).
pub fn tcp_dialer(addr: String) -> Dialer {
    Arc::new(move || {
        let addr = addr.clone();
        Box::pin(async move {
            let socket = tokio::time::timeout(
                Duration::from_secs(3),
                tokio::net::TcpStream::connect(addr.as_str()),
            )
            .await
            .map_err(|_| format!("{addr} did not answer"))?
            .map_err(|e| format!("{addr}: {e}"))?;
            vox::initiator_on(vox::transport::tcp::StreamLink::tcp(socket))
                .establish_connection()
                .await
                .map_err(|e| format!("{addr}: {e:?}"))
        })
    })
}

/// `HUB_PEERS`: `name=addr,…`. A name may not hold the peer separator.
pub fn parse_peers(spec: &str) -> Result<Vec<(String, String)>, String> {
    spec.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let (name, addr) = entry
                .split_once('=')
                .ok_or_else(|| format!("peer {entry:?}: want name=addr"))?;
            let (name, addr) = (name.trim(), addr.trim());
            if name.is_empty() || addr.is_empty() || name.contains(PEER_SEPARATOR) {
                return Err(format!(
                    "peer {entry:?}: want name=addr, a name without {PEER_SEPARATOR}"
                ));
            }
            Ok((name.to_string(), addr.to_string()))
        })
        .collect()
}
