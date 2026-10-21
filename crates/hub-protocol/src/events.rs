//! Events the hub carries and never reads.
//!
//! implements: 19983ca5288dc57fe00341ecd0dc00c1c97138a2312cd965a1b86acc9f2e01a8@19983ca5288dc57fe00341ecd0dc00c1c97138a2312cd965a1b86acc9f2e01a8

use facet::Facet;

/// One event: a topic subscribers match by prefix, and bytes only the
/// publisher and its readers understand.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct HubEvent {
    pub topic: String,
    pub payload: Vec<u8>,
}
