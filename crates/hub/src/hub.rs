//! The hub's shared state — the registry plus its discovery subscribers.
//!
//! Every mutation goes through here so the topology and the tail can never
//! disagree: the event a change implies is broadcast as part of applying it.

use std::sync::Arc;

use hub_protocol::{HubError, LinkSelector, NodeLink, Registration, TopologySnapshot};
use tokio::sync::Mutex;
use vox::Tx;

use crate::discovery::Subscribers;
use crate::events::EventSubscribers;
use crate::peers::Peers;
use crate::registry::Registry;

/// Shared hub state, cloneable across connection tasks.
///
/// `T` is the per-link handle used to call a node back (a `NodeService` client
/// in production), which keeps this testable without a transport.
pub struct Hub<T> {
    inner: Arc<Mutex<Inner<T>>>,
    /// Event subscribers, under a lock of their own: a slow subscriber must not
    /// hold up registration or routing.
    events: Arc<Mutex<EventSubscribers>>,
    /// This hub's name and its peer hubs.
    peers: Peers,
}

struct Inner<T> {
    registry: Registry<T>,
    subscribers: Subscribers,
}

impl<T> Clone for Hub<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            events: Arc::clone(&self.events),
            peers: self.peers.clone(),
        }
    }
}

impl<T: Clone> Default for Hub<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Hub<T> {
    /// A hub with nothing connected.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                registry: Registry::new(),
                subscribers: Subscribers::new(),
            })),
            events: Arc::new(Mutex::new(EventSubscribers::new())),
            peers: Peers::default(),
        }
    }

    /// This hub, named and peered by `peers`.
    pub fn with_peers(mut self, peers: Peers) -> Self {
        self.peers = peers;
        self
    }

    /// This hub's name and its peers.
    pub fn peers(&self) -> &Peers {
        &self.peers
    }

    /// Register a link and broadcast the topology change it implies (§2.1, §2.6).
    pub async fn register(&self, registration: Registration, handle: T) {
        let mut inner = self.inner.lock().await;
        if let Some(event) = inner.registry.register(registration, handle) {
            inner.subscribers.broadcast(event).await;
        }
    }

    /// Drop a link and broadcast the topology change it implies (§2.6).
    pub async fn disconnect(&self, target: &NodeLink) {
        let mut inner = self.inner.lock().await;
        if let Some(event) = inner.registry.remove(target) {
            inner.subscribers.broadcast(event).await;
        }
    }

    /// The live topology (§2.6).
    pub async fn snapshot(&self) -> TopologySnapshot {
        self.inner.lock().await.registry.snapshot()
    }

    /// Attach a consumer to the live topology tail (§2.6).
    pub async fn subscribe(&self, events: Tx<hub_protocol::TopologyEvent>) {
        self.inner.lock().await.subscribers.add(events);
    }

    /// Resolve a routing target to the handle that should answer it (§2.5).
    pub async fn resolve(&self, target: &LinkSelector) -> Result<T, HubError> {
        let inner = self.inner.lock().await;
        inner
            .registry
            .resolve(target)
            .map(|(_, entry)| entry.handle.clone())
    }

    /// Queue an event for every subscriber whose prefix matches its topic;
    /// how many it was queued for. Never waits on a subscriber.
    pub async fn publish(&self, event: hub_protocol::HubEvent) -> u32 {
        self.events.lock().await.publish(&event)
    }

    /// Attach a consumer to every event whose topic starts with `prefix`; its
    /// first event says the subscription is live. The returned receiver
    /// resolves once the consumer is gone or dropped for falling behind.
    pub async fn subscribe_events(
        &self,
        prefix: String,
        events: Tx<hub_protocol::HubEvent>,
    ) -> tokio::sync::oneshot::Receiver<()> {
        let hello = hub_protocol::HubEvent {
            topic: hub_protocol::SUBSCRIBED_TOPIC.to_string(),
            payload: prefix.clone().into_bytes(),
        };
        self.events.lock().await.add(prefix, events, hello)
    }

    /// As [`Hub::subscribe_events`], also handing back the subscriber's own
    /// queue, which a peer hub's events are carried into.
    pub async fn subscribe_events_with_queue(
        &self,
        prefix: String,
        events: Tx<hub_protocol::HubEvent>,
    ) -> (
        tokio::sync::oneshot::Receiver<()>,
        tokio::sync::mpsc::Sender<hub_protocol::HubEvent>,
    ) {
        let hello = hub_protocol::HubEvent {
            topic: hub_protocol::SUBSCRIBED_TOPIC.to_string(),
            payload: prefix.clone().into_bytes(),
        };
        self.events
            .lock()
            .await
            .add_with_queue(prefix, events, hello)
    }

    /// How many subscribers are attached (test/observability helper).
    pub async fn subscriber_count(&self) -> usize {
        self.inner.lock().await.subscribers.len()
    }
}
