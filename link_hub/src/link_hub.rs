use tonic::{Request, Response, Status};

use futures::Stream;
use std::{pin::Pin, sync::{Arc, RwLock}};
use dashmap::DashMap;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use crate::{proto, hub_app::AppLink};
use crate::link_session::{LinkId, SessionId, LinkSession};
use crate::hub_app::{AppId, HubApp};
use crate::error::ErrorMessage;
use crate::link_authenticator::LinkAuthenticator;

#[derive(Debug)]
pub struct LinkHub<TA>
    where TA: LinkAuthenticator
{
    links: DashMap<LinkId, Vec<SessionId>>,
    sessions: DashMap<SessionId, Arc<RwLock<LinkSession>>>,
    authenticator: TA,
    apps: Arc<DashMap<AppId, Arc<Box<dyn HubApp>>>>,
    fallback_app: Option<Arc<Box<dyn HubApp>>>,
}

impl<TA: LinkAuthenticator> LinkHub<TA> {
    pub const EVENT_CHANNEL_BUFFER_SIZE: usize = 16;

    pub fn new(authenticator: TA, fallback_app: Option<Arc<Box<dyn HubApp>>>) -> Self {
        Self {
            links: DashMap::new(),
            sessions: DashMap::new(),
            authenticator,
            apps: Arc::new(DashMap::new()),
            fallback_app,
        }
    }
    pub fn get_apps(&self) -> Arc<DashMap<AppId, Arc<Box<dyn HubApp>>>> {
        self.apps.clone()
    }
    pub fn add_app(&self, app_id: &str, app: Arc<Box<dyn HubApp>>) {
        self.apps.insert(AppId(app_id.to_owned()), app);
    }
    fn get_app(&self, app_id: &AppId) -> Result<Arc<Box<dyn HubApp>>, Status> {
        match self.apps.get(app_id) {
            Some(app) => {
                println!("LinkHub found app: {} -> {:?}", app_id.0, app.clone());
                Ok(app.clone())
            },
            None => {
                match self.fallback_app.as_ref() {
                    Some(app) => {
                        println!("LinkHub use fallback app: {} -> {:?}", app_id.0, app);
                        Ok(app.clone())
                    },
                    None => {
                        Err(Status::permission_denied(ErrorMessage::INVALID_APP_ID))
                    },
                }
            },
        }
    }
    fn get_session(&self, session_id: &String) -> Result<Arc<RwLock<LinkSession>>, Status> {
        match self.sessions.get(&SessionId(session_id.clone())) {
            Some(session) => Ok(session.clone()),
            None => Err(Status::permission_denied(ErrorMessage::INVALID_SESSION_ID)),
        }
    }
    fn get_link_session_ids(&self, link_id: &LinkId) -> Option<Vec<SessionId>> {
        self.links
            .get(link_id)
            .map(|x| x.value().clone())
    }
    async fn auth(&self, req: &proto::AuthRequest) -> Result<SessionId, Status> {
        self.authenticator.auth(req).await
    }
    fn kick_session(&self, link_id: &LinkId, session_id: &SessionId) {
        println!("LinkHub.kick_session() {} -> {}", link_id.0, session_id.0);
        self.sessions.remove(session_id);
    }
    fn add_session(&self, app_id:&AppId, link_id: &LinkId, session_id: &SessionId, link_address: &str, last_ord: u64) {
        match self.links.get_mut(link_id) {
            None => {
                self.links.insert(link_id.clone(), vec![session_id.clone()]);
            },
            Some(mut kv) => {
                if !self.authenticator.allow_multiple_sessions() {
                    for session_id in kv.value() {
                        self.kick_session(link_id, session_id);
                    }
                    kv.value_mut().clear();
                }
                kv.value_mut().push(session_id.clone());
            },
        }
        let link = AppLink::new(app_id, link_id, session_id, link_address);
        let session = LinkSession::new(link, last_ord);
        self.sessions.insert(session_id.clone(), Arc::new(RwLock::new(session)));
    }
}

type EventStream = Pin<Box<dyn Stream<Item = Result<proto::AppEvent, Status>> + Send>>;

#[tonic::async_trait]
impl<TA: LinkAuthenticator + Send + Sync + 'static> proto::link_hub_server::LinkHub for LinkHub<TA> {
    async fn auth(&self, request: Request<proto::AuthRequest>) ->  Result<Response<proto::AuthResponse>, Status> {
        let app_id = AppId(request.get_ref().app_id.clone());
        let app = self.get_app(&app_id)?;
        let link_id = LinkId(request.get_ref().link_id.clone());
        let old_session_ids = self.get_link_session_ids(&link_id);
        if old_session_ids.is_some() {
            if !self.authenticator.allow_multiple_sessions() {
                if !self.authenticator.kick_old_sessions() {
                    return Err(Status::permission_denied(ErrorMessage::ALREADY_AUTHENTICATED))
                }
            }
        }
        let trace_id = "TODO";
        let link_address = "TODO";
        let session_id = self.auth(request.get_ref()).await?;
        let session_id = app.auth(trace_id, link_address, &session_id.0, request.get_ref()).await?;
        self.add_session(&app_id, &link_id, &session_id, link_address, request.get_ref().ord);
        Ok(Response::new(proto::AuthResponse::new(request.get_ref(), &session_id)))
    }


    async fn handle(&self, request: Request<proto::AppRequest>) ->  Result<Response<proto::AppResponse>, Status> {
        let session = self.get_session(&request.get_ref().session_id)?;
        let link = LinkSession::check_ord(session.as_ref(), request.get_ref().ord)?;
        let app = self.get_app(&link.app_id)?;
        let res = app.handle("TODO", &link, request.get_ref()).await;
        res.map(move |res| {
            Response::new(res)
        })
    }

    type SubscribeStream = EventStream;

    async fn subscribe(&self, request: Request<proto::AppSubscribe>) -> Result<Response<Self::SubscribeStream>, Status> {
        let session = self.get_session(&request.get_ref().session_id)?;
        let link = LinkSession::check_ord(session.as_ref(), request.get_ref().ord)?;
        let app = self.get_app(&link.app_id)?;
        let mut receiver = app.subscribe("TODO", &link, request.get_ref()).await?;
        let (tx, rx) = tokio::sync::mpsc::channel(Self::EVENT_CHANNEL_BUFFER_SIZE);
        tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        println!("Event Receiver got event: {:?}", event);
                        match tx.send(Result::<_, Status>::Ok(event)).await {
                            Ok(_) => {},
                            Err(err) => {
                                println!("Forward Event got error: {:?}", err);
                                break;
                            }
                        }
                    },
                    Err(err) => {
                        println!("Event Receiver got error: {:?}", err);
                        break;
                    }
                }
            }
        });
        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(out_stream) as Self::SubscribeStream
        ))
    }
}