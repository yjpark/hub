use async_trait::async_trait;
use std::fmt::{Debug, Display};
use tonic::{Status};
use tokio::sync::broadcast::Receiver;

use crate::link_authenticator::LinkAuthenticator;
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
    pub link_address: String,
}
impl Display for AppLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}-{}", self.app_id.0, self.link_id.0, self.session_id.0, self.link_address)
    }
}
impl AppLink {
    pub fn new(app_id:&AppId, link_id: &LinkId, session_id: &SessionId, link_address: &str) -> Self {
        Self {
            app_id: app_id.clone(),
            link_id: link_id.clone(),
            session_id: session_id.clone(),
            link_address: link_address.to_owned(),
        }
    }
}

#[async_trait]
pub trait HubApp : Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn std::any::Any;
    async fn handle(&self, trace_id: &str, link: &AppLink, req: &proto::AppRequest) -> Result<proto::AppResponse, Status>;    
    async fn subscribe(&self, trace_id: &str, link: &AppLink, sub: &proto::AppSubscribe) -> Result<Receiver<proto::AppEvent>, Status>;    
    async fn auth(&self, _trace_id: &str, _link_address: &str, link_session: &str, _req: &proto::AuthRequest) -> Result<SessionId, Status> {
        Ok(SessionId(link_session.to_owned()))
    }
}

#[derive(Debug)]
pub struct DummyApp {}

#[async_trait]
impl HubApp for DummyApp {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    async fn handle(&self, trace_id: &str, link: &AppLink, req: &proto::AppRequest) -> Result<proto::AppResponse, Status> {
        println!("DummyApp.handle({:?} {:?} -> {:?})", trace_id, link, req);
        let data = req.data.as_ref().map(|x| x.clone());
        Ok(proto::AppResponse::new(req, data))
    }
    async fn subscribe(&self, trace_id: &str, link: &AppLink, sub: &proto::AppSubscribe) -> Result<Receiver<proto::AppEvent>, Status> {
        println!("DummyApp.subscribe({:?} {:?} -> {:?})", trace_id, link, sub);
        Err(Status::unimplemented(ErrorMessage::UNDER_CONSTRUCTION))
    }
}