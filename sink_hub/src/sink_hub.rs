use tonic::{Request, Response, Status, Code};

use futures::Stream;
use std::{pin::Pin, sync::{Arc, RwLock}, net::SocketAddr};
use dashmap::DashMap;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use link_hub::{hub_app::{HubApp, AppId}, link_hub::LinkHub};
use link_hub::link_session::SessionId;
use crate::{proto, sink_app::SinkApp, sink_session};
use crate::sink_session::{AppSink, SinkId, SinkSession};
use crate::error::{ErrorMessage, LinkErrorMessage};
use crate::sink_authenticator::SinkAuthenticator;

#[derive(Debug)]
pub struct SinkHub<TA>
    where TA: SinkAuthenticator
{
    sinks: DashMap<SinkId, Vec<SessionId>>,
    sessions: Arc<DashMap<SessionId, Arc<RwLock<SinkSession>>>>,
    authenticator: TA,
    apps: Arc<DashMap<AppId, Arc<Box<dyn HubApp>>>>,
}

impl<TA: SinkAuthenticator> SinkHub<TA> {
    pub const AUTH_CHANNEL_BUFFER_SIZE: usize = 32;
    pub const HANDLE_CHANNEL_BUFFER_SIZE: usize = 32;
    pub const EVENT_CHANNEL_BUFFER_SIZE: usize = 32;

    pub fn new(authenticator: TA, apps: Arc<DashMap<AppId, Arc<Box<dyn HubApp>>>>) -> Self {
        Self {
            sinks: DashMap::new(),
            sessions: Arc::new(DashMap::new()),
            authenticator,
            apps,
        }
    }
    fn add_app(&self, app_id: &str, app: Arc<Box<dyn HubApp>>) {
        self.apps.insert(AppId(app_id.to_owned()), app.clone());
    }
    fn get_app(&self, app_id: &AppId) -> Result<Arc<Box<dyn HubApp>>, Status> {
        match self.apps.get(app_id) {
            Some(app) => Ok(app.clone()),
            None => {
                Err(Status::permission_denied(ErrorMessage::INVALID_APP_ID))
            },
        }
    }
    fn get_or_add_app(&self, app_id: &AppId, kinds: &Vec<i32>) -> Result<Arc<Box<dyn HubApp>>, Status> {
        match self.apps.get(app_id) {
            Some(app) => Ok(app.clone()),
            None => {
                if self.authenticator.allow_create_app() {
                    let app = Arc::new(Box::new(SinkApp::new(app_id, kinds)) as Box<dyn HubApp>);
                    self.add_app(app_id.0.as_str(), app.clone());
                    Ok(app)
                } else {
                    Err(Status::permission_denied(ErrorMessage::INVALID_APP_ID))
                }
            },
        }
    }
    fn get_session(&self, session_id: &String) -> Result<Arc<RwLock<SinkSession>>, Status> {
        match self.sessions.get(&SessionId(session_id.clone())) {
            Some(session) => Ok(session.clone()),
            None => Err(Status::permission_denied(LinkErrorMessage::INVALID_SESSION_ID)),
        }
    }
    fn get_sink_session_ids(&self, sink_id: &SinkId) -> Option<Vec<SessionId>> {
        self.sinks
            .get(sink_id)
            .map(|x| x.value().clone())
    }
    async fn auth(&self, req: &proto::RegisterRequest) -> Result<SessionId, Status> {
        self.authenticator.auth(req).await
    }
    fn kick_session(&self, sink_id: &SinkId, session_id: &SessionId) {
        println!("LinkHub.kick_session() {} -> {}", sink_id.0, session_id.0);
        self.sessions.remove(session_id);
    }
    fn add_session(&self, app_id:&AppId, sink_id: &SinkId, session_id: &SessionId, last_ord: u64) {
        match self.sinks.get_mut(sink_id) {
            None => {
                self.sinks.insert(sink_id.clone(), vec![session_id.clone()]);
            },
            Some(mut kv) => {
                if !self.authenticator.allow_multiple_sessions() {
                    for session_id in kv.value() {
                        self.kick_session(sink_id, session_id);
                    }
                    kv.value_mut().clear();
                }
                kv.value_mut().push(session_id.clone());
            },
        }
        let link = AppSink::new(app_id, sink_id, session_id);
        let session = SinkSession::new(link, last_ord);
        self.sessions.insert(session_id.clone(), Arc::new(RwLock::new(session)));
    }
}

type AuthRequestStream = Pin<Box<dyn Stream<Item = Result<proto::LinkAuthRequest, Status>> + Send>>;

type AppRequestStream = Pin<Box<dyn Stream<Item = Result<proto::LinkAppRequest, Status>> + Send>>;

type AppSubscribeStream = Pin<Box<dyn Stream<Item = Result<proto::LinkAppSubscribe, Status>> + Send>>;

#[tonic::async_trait]
impl<TA: SinkAuthenticator + Send + Sync + 'static> proto::sink_hub_server::SinkHub for SinkHub<TA> {
    async fn register(&self, request: Request<proto::RegisterRequest>) ->  Result<Response<proto::RegisterResponse> ,Status> {
        let app_id = AppId(request.get_ref().app_id.clone());
        let kinds = request.get_ref().kinds.clone();
        let _app = self.get_or_add_app(&app_id, &kinds)?;
        let sink_id = SinkId(request.get_ref().sink_id.clone());
        let old_session_ids = self.get_sink_session_ids(&sink_id);
        if old_session_ids.is_some() {
            if !self.authenticator.allow_multiple_sessions() {
                if !self.authenticator.kick_old_sessions() {
                    return Err(Status::permission_denied(ErrorMessage::ALREADY_REGISTERED))
                }
            }
        }
        let session_id = self.auth(request.get_ref()).await?;
        self.add_session(&app_id, &sink_id, &session_id, request.get_ref().ord);
        Ok(Response::new(proto::RegisterResponse::new(request.get_ref(), &session_id, None)))
    }

    type AuthStream = AuthRequestStream;
    async fn auth(&self, request: Request<tonic::Streaming<proto::AuthResult>>) ->  Result<Response<Self::AuthStream> , Status> {
        let mut res_stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(Self::AUTH_CHANNEL_BUFFER_SIZE);
        let tx = Arc::new(tx);
        let sessions = self.sessions.clone();
        let apps = self.apps.clone();
        tokio::spawn(async move {
            let mut is_init = true;
            let mut sink_session = None;
            let mut app = None;
            while let Some(result) = res_stream.next().await {
                match result {
                    Ok(res) => {
                        if is_init {
                            is_init = false;
                            sink_session = Some(SessionId(res.sink_session.clone()));
                            if let Some(session) = sessions.get(&sink_session.as_ref().unwrap()) {
                                let app_id = session.read().unwrap().sink.app_id.clone();
                                app = apps.get(&app_id).map(|x| x.clone());
                                if app.is_some() {
                                    SinkApp::as_sink_app(app.as_ref().unwrap()).set_auth_request_sender(tx.clone());
                                }
                            }
                            if app.is_none() {
                                println!("SinkHub::auth(), init failed");
                                break;
                            }
                        } else {
                            if sink_session.as_ref().unwrap().0 != res.sink_session.clone() {
                                println!("SinkHub::auth(), bad sink_session: {} -> {}", sink_session.unwrap().0, res.sink_session);
                                break;
                            }
                        }
                        if let Some(app) = app.as_ref() {
                            match res.result {
                                Some(result) => {
                                    SinkApp::as_sink_app(&app).notify_auth(result);
                                },
                                None => {
                                    println!("SinkHub::auth(), got nothing");
                                    break;
                                },
                            }
                        }
                    },
                    Err(status) => {
                        println!("SinkHub::auth(), got error: {:?}", status);
                        break;
                    }
                }
            }
        });
        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(out_stream) as Self::AuthStream
        ))
    }

    type HandleStream = AppRequestStream;
    async fn handle(&self, request: Request<tonic::Streaming<proto::HandleResult>>) ->  Result<Response<Self::HandleStream> , Status> {
        let mut res_stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(Self::HANDLE_CHANNEL_BUFFER_SIZE);
        let tx = Arc::new(tx);
        let sessions = self.sessions.clone();
        let apps = self.apps.clone();
        tokio::spawn(async move {
            let mut is_init = true;
            let mut sink_session = None;
            let mut app = None;
            while let Some(result) = res_stream.next().await {
                match result {
                    Ok(res) => {
                        if is_init {
                            is_init = false;
                            sink_session = Some(SessionId(res.sink_session.clone()));
                            if let Some(session) = sessions.get(&sink_session.as_ref().unwrap()) {
                                let app_id = session.read().unwrap().sink.app_id.clone();
                                app = apps.get(&app_id).map(|x| x.clone());
                                if app.is_some() {
                                    SinkApp::as_sink_app(app.as_ref().unwrap()).set_handle_request_sender(tx.clone());
                                }
                            }
                            if app.is_none() {
                                println!("SinkHub::handle(), init failed");
                                break;
                            }
                        } else {
                            if sink_session.as_ref().unwrap().0 != res.sink_session.clone() {
                                println!("SinkHub::handle(), bad sink_session: {} -> {}", sink_session.unwrap().0, res.sink_session);
                                break;
                            }
                        }
                        if let Some(app) = app.as_ref() {
                            match res.result {
                                Some(result) => {
                                    SinkApp::as_sink_app(&app).notify_handle(result);
                                },
                                None => {
                                    println!("SinkHub::handle(), got nothing");
                                    break;
                                },
                            }
                        }
                    },
                    Err(status) => {
                        println!("SinkHub::handle(), got error: {:?}", status);
                        break;
                    }
                }
            }
        });
        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(out_stream) as Self::HandleStream
        ))
    }

    type PublishStream = AppSubscribeStream;
    async fn publish(&self, request: Request<tonic::Streaming<proto::AppEvent>>) ->  Result<Response<Self::PublishStream> , Status> {
        Err(Status::unimplemented(ErrorMessage::UNDER_CONSTRUCTION))
    }
}