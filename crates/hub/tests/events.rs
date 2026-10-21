#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The event channel over a real TCP socket: one consumer publishes, every
//! consumer subscribed to a matching prefix receives it, byte for byte.
//!
//! implements: 19983ca5288dc57fe00341ecd0dc00c1c97138a2312cd965a1b86acc9f2e01a8@c80bd1aa79624551fdf589040d97c790dc5c651c88db9a771399a2038aa0b9e6

use std::time::Duration;

use hub::{HubListener, ServedHub};
use hub_protocol::HubEvent;
use hub_protocol::service::ConsumerApiClient;

async fn consumer_for(addr: &str) -> ConsumerApiClient {
    let socket = tokio::net::TcpStream::connect(addr).await.expect("connect");
    vox::initiator_on(vox::transport::tcp::StreamLink::tcp(socket))
        .establish_connection()
        .await
        .expect("consumer establish")
        .open_lane()
        .await
        .expect("open ConsumerApi lane")
}

/// The event out of the frame it was decoded in (it owns its strings and
/// bytes, so nothing borrows the frame).
fn owned(got: vox::SelfRef<HubEvent>) -> HubEvent {
    let mut out = None;
    let _ = got.map(|e| out = Some(e));
    out.expect("an event")
}

/// Subscribe to `prefix` on a connection of its own, and wait until the hub
/// says the subscription is live.
async fn subscribed(addr: &str, prefix: &str) -> vox::Rx<HubEvent> {
    let consumer = consumer_for(addr).await;
    let (tx, mut rx) = vox::channel::<HubEvent>();
    let prefix = prefix.to_string();
    tokio::spawn(async move {
        let _ = consumer.subscribe_events(prefix, tx).await;
    });
    let hello = owned(
        tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("the hub answers in time")
            .expect("a value")
            .expect("an event"),
    );
    assert_eq!(hello.topic, hub_protocol::SUBSCRIBED_TOPIC);
    rx
}

fn event(topic: &str, payload: &[u8]) -> HubEvent {
    HubEvent {
        topic: topic.to_string(),
        payload: payload.to_vec(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn an_event_reaches_every_matching_subscriber_and_no_other() {
    let listener = HubListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().to_string();
    tokio::spawn(async move { listener.serve(ServedHub::new()).await });

    let publisher = consumer_for(&addr).await;
    // A subscription streams within its call: the call stays open while the
    // subscriber listens, so it runs on a task of its own. The hub's first
    // event says the subscription is live.
    let mut rx1 = subscribed(&addr, "kudo/context/c1/").await;
    let mut rx2 = subscribed(&addr, "kudo/context/c2/").await;

    // Opaque bytes, including ones that are not text.
    let payload = [0u8, 1, 2, 255, b'\n'];
    let reached = publisher
        .publish(event("kudo/context/c1/submitted", &payload))
        .await
        .expect("publish");
    assert_eq!(reached, 1, "one subscriber matches");

    let got = owned(
        tokio::time::timeout(Duration::from_secs(5), rx1.recv())
            .await
            .expect("delivered in time")
            .expect("a value")
            .expect("an event"),
    );
    assert_eq!(got.topic, "kudo/context/c1/submitted");
    assert_eq!(got.payload, payload);

    // The c2 subscriber's first event is the one meant for it: it never saw c1's.
    publisher
        .publish(event("kudo/context/c2/claimed", b"two"))
        .await
        .expect("publish c2");
    let first = owned(
        tokio::time::timeout(Duration::from_secs(5), rx2.recv())
            .await
            .expect("delivered in time")
            .expect("a value")
            .expect("an event"),
    );
    assert_eq!(first.topic, "kudo/context/c2/claimed");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_event_nobody_listens_for_reaches_no_one() {
    let listener = HubListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().to_string();
    tokio::spawn(async move { listener.serve(ServedHub::new()).await });
    let publisher = consumer_for(&addr).await;
    let reached = publisher
        .publish(event("kudo/context/nobody/x", b""))
        .await
        .expect("publish");
    assert_eq!(reached, 0);
}

/// A subscriber that stops reading cannot hold up anyone else: publishers keep
/// publishing, and a new subscriber is still answered. The stalled one is
/// dropped once it falls too far behind (it re-reads the store).
#[tokio::test(flavor = "multi_thread")]
async fn a_stalled_subscriber_holds_up_no_one() {
    let listener = HubListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().to_string();
    tokio::spawn(async move { listener.serve(ServedHub::new()).await });

    // Subscribed, live, and then never read again.
    let _stalled = subscribed(&addr, "kudo/").await;
    let publisher = consumer_for(&addr).await;
    let all = async {
        for n in 0..2000u32 {
            publisher
                .publish(event("kudo/context/c1/x", &n.to_be_bytes()))
                .await
                .expect("publish");
        }
    };
    tokio::time::timeout(Duration::from_secs(20), all)
        .await
        .expect("publishing never waits on a subscriber that stopped reading");
    let mut late = tokio::time::timeout(Duration::from_secs(5), subscribed(&addr, "kudo/"))
        .await
        .expect("a new subscriber is still answered");
    publisher
        .publish(event("kudo/after", b""))
        .await
        .expect("publish");
    let got = owned(
        tokio::time::timeout(Duration::from_secs(5), late.recv())
            .await
            .expect("delivered")
            .expect("a value")
            .expect("an event"),
    );
    assert_eq!(got.topic, "kudo/after");
}

#[tokio::test(flavor = "multi_thread")]
async fn every_matching_subscriber_is_counted() {
    let listener = HubListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().to_string();
    tokio::spawn(async move { listener.serve(ServedHub::new()).await });
    let _a = subscribed(&addr, "kudo/").await;
    let _b = subscribed(&addr, "kudo/context/c1/").await;
    let _c = subscribed(&addr, "kudo/context/c10/").await;
    let publisher = consumer_for(&addr).await;
    let reached = publisher
        .publish(event("kudo/context/c1/x", b""))
        .await
        .expect("publish");
    assert_eq!(reached, 2, "kudo/ and kudo/context/c1/, not c10");
}

/// `hub/` topics are the hub's own; and a payload has a ceiling.
#[tokio::test(flavor = "multi_thread")]
async fn the_hubs_topics_and_oversized_payloads_are_refused() {
    let listener = HubListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().to_string();
    tokio::spawn(async move { listener.serve(ServedHub::new()).await });
    let publisher = consumer_for(&addr).await;
    assert!(
        publisher
            .publish(event(hub_protocol::SUBSCRIBED_TOPIC, b"kudo/"))
            .await
            .is_err()
    );
    let big = vec![0u8; hub_protocol::MAX_EVENT_PAYLOAD + 1];
    assert!(publisher.publish(event("kudo/x", &big)).await.is_err());
}
