use echo_sink::echo::EchoResponse;
use tonic::transport::Channel;
use uuid::Uuid;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use hub_grpc_sink::proto::{sink_hub_client::SinkHubClient, RegisterRequest, AuthResult, AuthResponse, RegisterResponse, HandleResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut client = SinkHubClient::connect("http://127.0.0.1:1234").await.unwrap();
        let mut kinds = Vec::new();
        kinds.push(1);
        let res = client.register(RegisterRequest{
            ord: 1,
            app_id: "b1b03254-e344-4e48-9ca4-e7062f219b95".to_owned(),
            sink_id: Uuid::new_v4().to_string(),
            token: Uuid::new_v4().to_string(),
            kinds: kinds,
            extra: None,
        }).await?;
        println!("register succeed: {:?}", res);
        let _ = tokio::join!(
            auth(&mut client, &res.get_ref()).await,
            handle(&mut client, &res.get_ref()).await);
    }
    Ok(())
}

pub const AUTH_CHANNEL_BUFFER_SIZE: usize = 16;

async fn auth(client: &mut SinkHubClient<Channel>, register_res: &RegisterResponse) -> tokio::task::JoinHandle<()> {
    let (tx, rx) = tokio::sync::mpsc::channel(AUTH_CHANNEL_BUFFER_SIZE);
    let res_stream = ReceiverStream::new(rx);
    let response = client.auth(res_stream).await.unwrap();
    let mut req_stream = response.into_inner();
    if let Err(err) = tx.send(AuthResult::init(register_res)).await {
        println!("Auth init failed: {:#?}", err);
    }
    let sink_session = register_res.session_id.clone();
    tokio::spawn(async move {
        tokio::pin!(tx);
        let mut count = 0;
        while let Some(req) = req_stream.next().await {
            count += 1;
            match req {
                Ok(link_req) => {
                    println!("auth() got request: {:?}", link_req);
                    let res = AuthResponse::new(&link_req);
                    let send_result = tx.send(AuthResult::ok(&sink_session, res)).await;
                    println!("auth() send: {:?}", send_result);
                },
                Err(status) => {
                    println!("auth() got error: {:?}", status);
                    break;
                }
            }
        }
        println!("auth finished: {:?}", count);
    })
}

pub const HANDLE_CHANNEL_BUFFER_SIZE: usize = 16;

async fn handle(client: &mut SinkHubClient<Channel>, register_res: &RegisterResponse) -> tokio::task::JoinHandle<()> {
    let (tx, rx) = tokio::sync::mpsc::channel(HANDLE_CHANNEL_BUFFER_SIZE);
    let res_stream = ReceiverStream::new(rx);
    let response = client.handle(res_stream).await.unwrap();
    let mut req_stream = response.into_inner();
    if let Err(err) = tx.send(HandleResult::init(register_res)).await {
        println!("Auth init failed: {:#?}", err);
    }
    let sink_session = register_res.session_id.clone();
    tokio::spawn(async move {
        tokio::pin!(tx);
        let mut count = 0;
        while let Some(req) = req_stream.next().await {
            count += 1;
            match req {
                Ok(link_req) => {
                    println!("handle() got request: {:?}", link_req);
                    //let req = link_req.data
                    let res = EchoResponse::from_app(&link_req).to_app(&link_req);
                    let send_result = tx.send(HandleResult::ok(&sink_session, res)).await;
                    println!("handle() send: {:?}", send_result);
                },
                Err(status) => {
                    println!("handle() got error: {:?}", status);
                    break;
                }
            }
        }
        println!("handle finished: {:?}", count);
    })
}