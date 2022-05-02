use std::{fmt::Display, sync::{RwLockReadGuard, RwLockWriteGuard, RwLock}};

use tonic::Status;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SinkId(pub String);

impl From<&str> for SinkId {
    fn from(v: &str) -> Self {
        Self(v.to_owned())
    }
}

pub type SessionId = link_hub::link_session::SessionId;
pub use link_hub::hub_app::AppId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppSink {
    pub app_id: AppId,
    pub sink_id: SinkId,
    pub session_id: SessionId,
}
impl Display for AppSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.app_id.0, self.sink_id.0, self.session_id.0)
    }
}
impl AppSink {
    pub fn new(app_id:&AppId, sink_id: &SinkId, session_id: &SessionId) -> Self {
        Self {
            app_id: app_id.clone(),
            sink_id: sink_id.clone(),
            session_id: session_id.clone(),
        }
    }
}

pub type LinkErrorMessage = link_hub::error::ErrorMessage;

#[derive(Debug)]
pub struct SinkSession {
    pub sink: AppSink,
    pub last_ord: u64,
}

impl Display for SinkSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<LinkSession>({}->{}", self.sink, self.last_ord)
    }
}

impl SinkSession {
    pub fn new(sink: AppSink, last_ord: u64) -> Self {
        Self {
            sink,
            last_ord,
        }
    }
    pub fn read_sink_last_ord(x: RwLockReadGuard<SinkSession>) -> (AppSink, u64) {
        (x.sink.clone(), x.last_ord)
    }
    pub fn update_last_ord(mut x: RwLockWriteGuard<SinkSession>, ord: u64) {
        x.last_ord = ord;
    }
    pub fn check_ord(session: &RwLock<SinkSession>, ord: u64) -> Result<AppSink, Status> {
        let (sink, last_ord) = SinkSession::read_sink_last_ord(session.read().unwrap());
        if ord <= last_ord {
            return Err(Status::aborted(LinkErrorMessage::INVALID_ORD));
        }
        SinkSession::update_last_ord(session.write().unwrap(), ord);
        Ok(sink)
    }
}