use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use dashmap::DashMap;
use link_hub::link_session::SessionId;
use tonic::{Status, Code};
use tokio::sync::broadcast::Receiver;

use link_hub::hub_app::{AppId, HubApp, AppLink};

use link_hub::proto;
use crate::proto::{auth_result, LinkAuthRequest, handle_result, LinkAppRequest};

use crate::error::ErrorMessage;

#[derive(Debug)]
pub struct SinkApp {
    pub app_id: AppId,
    pub kinds: Vec<i32>,
    auth_request_sender: RwLock<Option<Arc<tokio::sync::mpsc::Sender<Result<LinkAuthRequest, Status>>>>>,
    auth_notifiers: DashMap<SessionId, tokio::sync::oneshot::Sender<auth_result::Result>>,
    handle_request_sender: RwLock<Option<Arc<tokio::sync::mpsc::Sender<Result<LinkAppRequest, Status>>>>>,
    handle_notifiers: DashMap<SessionId, tokio::sync::oneshot::Sender<handle_result::Result>>,
}

impl SinkApp {
    pub fn new(app_id: &AppId, kinds: &Vec<i32>) -> Self {
        Self {
            app_id: app_id.clone(),
            kinds: kinds.clone(),
            auth_request_sender: RwLock::new(None),
            auth_notifiers: DashMap::new(),
            handle_request_sender: RwLock::new(None),
            handle_notifiers: DashMap::new(),
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
    pub fn set_handle_request_sender(&self, sender: Arc<tokio::sync::mpsc::Sender<Result<LinkAppRequest, Status>>>) {
        let mut guard = self.handle_request_sender.write().unwrap();
        *guard = Some(sender);
    }
    pub fn notify_handle(&self, result: handle_result::Result) -> bool {
        let session_id = match &result {
            handle_result::Result::Ok(res) => SessionId(res.session_id.clone()),
            handle_result::Result::Err(err) => SessionId(err.session_id.clone()),
        };
        match self.handle_notifiers.remove(&session_id) {
            Some((_, notifier)) => {
                if let Err(err) = notifier.send(result) {
                    println!("SinkApp::notify_handle() failed: {} -> {:?}", session_id.0, err);
                    false
                } else {
                    println!("SinkApp::notify_handle() succeed: {}", session_id.0);
                    true
                }
            },
            None => {
                println!("SinkApp::notify_handle() failed: {} -> notifier not found", session_id.0);
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
    async fn auth(&self, trace_id: &str, link_address: &str, link_session: &str, req: &proto::AuthRequest) -> Result<SessionId, Status> {
        println!("SinkApp.auth({:?} {:?} {:?} -> {:?})", trace_id, link_address, link_session, req);
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
    async fn handle(&self, trace_id: &str, link: &AppLink, req: &proto::AppRequest) -> Result<proto::AppResponse, Status> {
        println!("SinkApp.handle({:?} {:?} -> {:?})", trace_id, link, req);
        let mut sender = None;
        {
            sender = self.handle_request_sender.read().unwrap().as_ref().map(|x| x.clone());
        }
        if sender.is_none() {
            return Err(Status::internal("TODO"));
        }
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.handle_notifiers.insert(link.session_id.clone(), tx);
        if let Err(err) = sender.unwrap().send(Ok(LinkAppRequest::new(trace_id, &link.link_address, req))).await {
            return Err(Status::internal(err.to_string()));
        }
        match rx.await {
            Ok(result) => {
                match result {
                    handle_result::Result::Ok(res) => {
                        Ok(res)
                    },
                    handle_result::Result::Err(status) => {
                        Err(Status::new(Code::from_i32(status.code), status.message.clone()))
                    },
                }
            },
            Err(err) => {
                Err(Status::internal(err.to_string()))
            }
        }
    }
    async fn subscribe(&self, trace_id: &str, link: &AppLink, sub: &proto::AppSubscribe) -> Result<Receiver<proto::AppEvent>, Status> {
        println!("SinkApp.subscribe({:?} {:?} -> {:?})", trace_id, link, sub);
        Err(Status::unimplemented(ErrorMessage::UNDER_CONSTRUCTION))
    }
}