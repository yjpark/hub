//! Event fan-out — events the hub carries and never reads.
//!
//! A subscriber names a topic prefix; every published event whose topic starts
//! with it is delivered, in the order the hub received them. A subscriber
//! whose channel has gone away is dropped silently, as a topology subscriber
//! is: a console tab closing is ordinary.
//!
//! implements: 19983ca5288dc57fe00341ecd0dc00c1c97138a2312cd965a1b86acc9f2e01a8@19983ca5288dc57fe00341ecd0dc00c1c97138a2312cd965a1b86acc9f2e01a8

use hub_protocol::HubEvent;
use vox::Tx;

/// One subscriber: its prefix, its channel, and the token whose drop ends
/// the subscribe call that is holding the channel open.
type Subscriber = (String, Tx<HubEvent>, tokio::sync::oneshot::Sender<()>);

/// The live event subscribers, each with its topic prefix.
#[derive(Default)]
pub struct EventSubscribers {
    channels: Vec<Subscriber>,
}

impl EventSubscribers {
    /// No subscribers yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a subscriber for every topic starting with `prefix`. The returned
    /// receiver resolves when the subscriber is dropped (its channel gone), so
    /// the call holding the channel open knows to return.
    pub fn add(
        &mut self,
        prefix: String,
        events: Tx<HubEvent>,
    ) -> tokio::sync::oneshot::Receiver<()> {
        let (alive, gone) = tokio::sync::oneshot::channel();
        self.channels.push((prefix, events, alive));
        gone
    }

    /// How many subscribers are attached.
    pub fn len(&self) -> usize {
        self.channels.len()
    }

    /// Whether anyone is listening.
    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }

    /// Deliver `event` to every subscriber whose prefix matches its topic,
    /// dropping those that have gone away. Returns how many received it.
    pub async fn publish(&mut self, event: HubEvent) -> u32 {
        let mut live = Vec::with_capacity(self.channels.len());
        let mut delivered = 0u32;
        for (prefix, channel, alive) in std::mem::take(&mut self.channels) {
            if !event.topic.starts_with(&prefix) {
                live.push((prefix, channel, alive));
                continue;
            }
            if channel.send(event.clone()).await.is_ok() {
                delivered = delivered.saturating_add(1);
                live.push((prefix, channel, alive));
            }
            // A send error means the consumer is gone — drop the channel, and
            // with it the token, which ends its subscribe call.
        }
        self.channels = live;
        delivered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Delivery is proven end to end (tests/events.rs): a local Tx/Rx pair is
    // never bound, so it cannot carry a value here.
    #[test]
    fn event_subscribers_start_empty() {
        let subs = EventSubscribers::new();
        assert!(subs.is_empty());
        assert_eq!(subs.len(), 0);
    }
}
