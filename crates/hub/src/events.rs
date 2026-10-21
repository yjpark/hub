//! Event fan-out — events the hub carries and never reads.
//!
//! A subscriber names a topic prefix; every published event whose topic starts
//! with it is delivered, in the order the hub received them. Each subscriber
//! has a bounded queue of its own and a task that forwards from it, so a
//! publisher never waits on any subscriber: one that falls `QUEUE` events
//! behind (it stopped reading, its link stalled) is dropped, and re-reads the
//! store it was being told about. A subscriber that is gone is dropped too —
//! whether a send to it fails or its subscribe call was cancelled.
//!
//! implements: 19983ca5288dc57fe00341ecd0dc00c1c97138a2312cd965a1b86acc9f2e01a8@c80bd1aa79624551fdf589040d97c790dc5c651c88db9a771399a2038aa0b9e6

use hub_protocol::HubEvent;
use tokio::sync::{mpsc, oneshot};
use vox::Tx;

/// Events one subscriber may fall behind by before it is dropped.
const QUEUE: usize = 256;

/// One subscriber: its prefix, its queue, and the token whose drop evicts it
/// (stopping its forwarding task).
struct Subscriber {
    prefix: String,
    queue: mpsc::Sender<HubEvent>,
    _evict: oneshot::Sender<()>,
}

/// The live event subscribers.
#[derive(Default)]
pub struct EventSubscribers {
    subscribers: Vec<Subscriber>,
}

impl EventSubscribers {
    /// No subscribers yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a subscriber for every topic starting with `prefix`; `hello` is
    /// queued first, so nothing published after it can be missed. The
    /// returned receiver resolves when the subscriber's forwarding stops — the
    /// call holding its channel open then returns.
    pub fn add(
        &mut self,
        prefix: String,
        events: Tx<HubEvent>,
        hello: HubEvent,
    ) -> oneshot::Receiver<()> {
        self.subscribers.retain(|s| !s.queue.is_closed());
        let (queue, mut queued) = mpsc::channel::<HubEvent>(QUEUE);
        let (alive, gone) = oneshot::channel::<()>();
        let (evict, mut evicted) = oneshot::channel::<()>();
        // A fresh queue has room: the hello is always first.
        let _ = queue.try_send(hello);
        tokio::spawn(async move {
            let mut alive = alive;
            loop {
                let event = tokio::select! {
                    event = queued.recv() => match event {
                        Some(event) => event,
                        None => break,
                    },
                    _ = alive.closed() => break,
                    _ = &mut evicted => break,
                };
                tokio::select! {
                    sent = events.send(event) => if sent.is_err() { break },
                    _ = alive.closed() => break,
                    _ = &mut evicted => break,
                }
            }
            // Dropping `alive` ends the subscriber's call.
        });
        self.subscribers.push(Subscriber {
            prefix,
            queue,
            _evict: evict,
        });
        gone
    }

    /// How many subscribers are attached.
    pub fn len(&self) -> usize {
        self.subscribers.len()
    }

    /// Whether anyone is listening.
    pub fn is_empty(&self) -> bool {
        self.subscribers.is_empty()
    }

    /// Queue `event` for every subscriber whose prefix matches its topic,
    /// without waiting on any; one whose queue is full or closed is dropped.
    /// Returns how many it was queued for.
    pub fn publish(&mut self, event: &HubEvent) -> u32 {
        let mut queued = 0u32;
        self.subscribers.retain(|s| {
            if !event.topic.starts_with(&s.prefix) {
                return !s.queue.is_closed();
            }
            match s.queue.try_send(event.clone()) {
                Ok(()) => {
                    queued = queued.saturating_add(1);
                    true
                }
                // Full: it stopped keeping up. Closed: it is gone.
                Err(_) => false,
            }
        });
        queued
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
