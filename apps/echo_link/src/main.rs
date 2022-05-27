use echo_link::echo::EchoRequest;
use tonic::transport::Channel;
use uuid::Uuid;
use futures::stream::Stream;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use hub_link::proto::{link_hub_client::LinkHubClient, AuthRequest, AppRequest};
use hub_link::hub_session::SessionId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = LinkHubClient::connect("http://127.0.0.1:1234").await.unwrap();
    let res = client.auth(AuthRequest{
        ord: 1,
        app_id: "b1b03254-e344-4e48-9ca4-e7062f219b95".to_owned(),
        link_id: Uuid::new_v4().to_string(),
        token: Uuid::new_v4().to_string(),
        extra: None,
    }).await;
    println!("auth succeed: {:#?}", res);
    let session_id = SessionId(res.unwrap().get_ref().session_id.clone());
    let req = EchoRequest {
        time: None,
        message: "LINK".to_owned(),
    };
    let res = client.handle(req.to_app(2, &session_id)).await;
    println!("handle succeed: {:#?}", res);
    Ok(())
}