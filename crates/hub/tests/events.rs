#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The event channel over a real TCP socket: one consumer publishes, every
//! consumer subscribed to a matching prefix receives it, byte for byte.
//!
//! implements: 19983ca5288dc57fe00341ecd0dc00c1c97138a2312cd965a1b86acc9f2e01a8@19983ca5288dc57fe00341ecd0dc00c1c97138a2312cd965a1b86acc9f2e01a8

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

    let follows_c1 = consumer_for(&addr).await;
    let follows_c2 = consumer_for(&addr).await;
    let publisher = consumer_for(&addr).await;

    let (tx1, mut rx1) = vox::channel::<HubEvent>();
    follows_c1
        .subscribe_events("kudo/context/c1/".to_string(), tx1)
        .await
        .expect("subscribe c1");
    let (tx2, mut rx2) = vox::channel::<HubEvent>();
    follows_c2
        .subscribe_events("kudo/context/c2/".to_string(), tx2)
        .await
        .expect("subscribe c2");

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
