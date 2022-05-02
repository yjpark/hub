use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use dashmap::DashMap;
use link_hub::link_session::SessionId;
use tonic::{Status, Code};
use tokio::sync::broadcast::Receiver;

use link_hub::hub_app::{AppId, HubApp, AppLink};

use link_hub::proto;
use crate::proto::{auth_result, LinkAuthRequest};

use crate::error::ErrorMessage;

#[derive(Debug)]
pub struct SinkApp {
    pub app_id: AppId,
    auth_request_sender: RwLock<Option<Arc<tokio::sync::mpsc::Sender<Result<LinkAuthRequest, Status>>>>>,
    auth_notifiers: DashMap<SessionId, tokio::sync::oneshot::Sender<auth_result::Result>>,
}

impl SinkApp {
    pub fn new(app_id: &AppId) -> Self {
        Self {
            app_id: app_id.clone(),
            auth_request_sender: RwLock::new(None),
            auth_notifiers: DashMap::new(),
        }
    }
    pub fn is_sink_app(app: &Box<dyn HubApp>) -> bool {
        app.as_any().downcast_ref::<SinkApp>().is_some()
    }
    pub fn as_sink_app<'a>(app: &'a Box<(dyn HubApp + 'static)>) -> &'a SinkApp {
        app.as_any().downcast_ref::<SinkApp>().unwrap()
    }
    pub fn set_auth_request_sender(&self, sender: Arc<tokio::sync::mpsc::Sender<Result<LinkAuthRequest, Status>>>) {
        let mut guard = self.auth_request_sender.write().unwrap();
        *guard = Some(sender);
    }
    pub fn notify_auth(&self, result: auth_result::Result) -> bool {
        let session_id = match &result {
            auth_result::Result::Ok(res) => SessionId(res.session_id.clone()),
            auth_result::Result::Err(err) => SessionId(err.session_id.clone()),
        };
        match self.auth_notifiers.remove(&session_id) {
            Some((_, notifier)) => {
                if let Err(err) = notifier.send(result) {
                    println!("SinkApp::notify_auth() failed: {} -> {:?}", session_id.0, err);
                    false
                } else {
                    println!("SinkApp::notify_auth() succeed: {}", session_id.0);
                    true
                }
            },
            None => {
                println!("SinkApp::notify_auth() failed: {} -> notifier not found", session_id.0);
                false
            }
        }
    }
}

#[async_trait]
impl HubApp for SinkApp {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    async fn handle(&self, link: &AppLink, req: &proto::AppRequest) -> Result<proto::AppResponse, Status> {
        println!("DummyApp.handle({:?} -> {:?})", link, req);
        let data = req.data.as_ref().map(|x| x.clone());
        Ok(proto::AppResponse::new(req, data))
    }
    async fn subscribe(&self, link: &AppLink, sub: &proto::AppSubscribe) -> Result<Receiver<proto::AppEvent>, Status> {
        println!("DummyApp.subscribe({:?} -> {:?})", link, sub);
        Err(Status::unimplemented(ErrorMessage::UNDER_CONSTRUCTION))
    }
    async fn auth(&self, trace_id: &str, link_address: &str, link_session: &str, req: &proto::AuthRequest) -> Result<SessionId, Status> {
        let mut sender = None;
        {
            sender = self.auth_request_sender.read().unwrap().as_ref().map(|x| x.clone());
        }
        if sender.is_none() {
            return Err(Status::internal("TODO"));
        }
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.auth_notifiers.insert(SessionId(link_session.to_owned()), tx);
        if let Err(err) = sender.unwrap().send(Ok(LinkAuthRequest::new(trace_id, link_address, link_session, req))).await {
            return Err(Status::internal(err.to_string()));
        }
        match rx.await {
            Ok(result) => {
                match result {
                    auth_result::Result::Ok(res) => {
                        Ok(SessionId(res.session_id.clone()))
                    },
                    auth_result::Result::Err(status) => {
                        Err(Status::new(Code::from_i32(status.code), status.message.clone()))
                    },
                }
            },
            Err(err) => {
                Err(Status::internal(err.to_string()))
            }
        }
    }
}