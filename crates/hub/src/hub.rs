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
        }
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

    /// Publish an event to every subscriber whose prefix matches its topic;
    /// how many it reached.
    pub async fn publish(&self, event: hub_protocol::HubEvent) -> u32 {
        self.events.lock().await.publish(event).await
    }

    /// Attach a consumer to every event whose topic starts with `prefix`,
    /// after telling it the subscription is live. The returned receiver
    /// resolves once the consumer is gone; `None` if it was gone already.
    pub async fn subscribe_events(
        &self,
        prefix: String,
        events: Tx<hub_protocol::HubEvent>,
    ) -> Option<tokio::sync::oneshot::Receiver<()>> {
        let hello = hub_protocol::HubEvent {
            topic: hub_protocol::SUBSCRIBED_TOPIC.to_string(),
            payload: prefix.clone().into_bytes(),
        };
        // Held across the hello, so no event can slip between it and joining.
        let mut subscribers = self.events.lock().await;
        events.send(hello).await.ok()?;
        Some(subscribers.add(prefix, events))
    }

    /// How many subscribers are attached (test/observability helper).
    pub async fn subscriber_count(&self) -> usize {
        self.inner.lock().await.subscribers.len()
    }
}
