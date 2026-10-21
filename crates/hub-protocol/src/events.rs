//! Events the hub carries and never reads.
//!
//! implements: 19983ca5288dc57fe00341ecd0dc00c1c97138a2312cd965a1b86acc9f2e01a8@c80bd1aa79624551fdf589040d97c790dc5c651c88db9a771399a2038aa0b9e6

use facet::Facet;

/// The topic of the first event every subscription receives: the hub's word
/// that the subscription is live, so nothing published after it can be missed.
/// Its payload is the prefix subscribed to.
pub const SUBSCRIBED_TOPIC: &str = "hub/subscribed";

/// The largest payload one event may carry.
pub const MAX_EVENT_PAYLOAD: usize = 64 * 1024;

/// One event: a topic subscribers match by prefix, and bytes only the
/// publisher and its readers understand.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct HubEvent {
    pub topic: String,
    pub payload: Vec<u8>,
}
