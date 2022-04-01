use async_trait::async_trait;
use prost_types::Any;
use std::fmt::{Debug, Display};
use tonic::{Status};
use tokio::sync::broadcast::Receiver;

use crate::link_session::{LinkId, SessionId};
use crate::proto;
use crate::error::ErrorMessage;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppLink {
    pub app_id: AppId,
    pub link_id: LinkId,
    pub session_id: SessionId,
}
impl Display for AppLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.app_id.0, self.link_id.0, self.session_id.0)
    }
}
impl AppLink {
    pub fn new(app_id:&AppId, link_id: &LinkId, session_id: &SessionId) -> Self {
        Self {
            app_id: app_id.clone(),
            link_id: link_id.clone(),
            session_id: session_id.clone(),
        }
    }
}

#[async_trait]
pub trait HubApp : Debug + Send + Sync + 'static {
    async fn handle(&self, link: &AppLink, req: &proto::AppRequest) -> Result<proto::AppResponse, Status>;    
    async fn subscribe(&self, link: &AppLink, sub: &proto::AppSubscribe) -> Result<Receiver<proto::AppEvent>, Status>;    
}

#[derive(Debug)]
pub struct DummyApp {}

#[async_trait]
impl HubApp for DummyApp {
    async fn handle(&self, link: &AppLink, req: &proto::AppRequest) -> Result<proto::AppResponse, Status> {
        println!("DummyApp.handle({:?} -> {:?})", link, req);
        let data = req.data.as_ref().map(|x| x.clone());
        Ok(proto::AppResponse::new(req, data))
    }
    async fn subscribe(&self, link: &AppLink, sub: &proto::AppSubscribe) -> Result<Receiver<proto::AppEvent>, Status> {
        println!("DummyApp.subscribe({:?} -> {:?})", link, sub);
        Err(Status::unimplemented(ErrorMessage::UNDER_CONSTRUCTION))
    }
}