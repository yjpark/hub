use async_trait::async_trait;
use prost_types::Any;
use tonic::{Status};
use uuid::Uuid;

use crate::{proto, link_session::SessionId};
use crate::error::ErrorMessage;

#[async_trait]
pub trait LinkAuthenticator {
    fn allow_multiple_sessions(&self) -> bool {
        false
    }
    fn kick_old_sessions(&self) -> bool {
        true
    }
    async fn auth(&self, req: &proto::AuthRequest) -> Result<SessionId, Status>;    
}

pub struct PublicUuidAuthenticator {}

#[async_trait]
impl LinkAuthenticator for PublicUuidAuthenticator {
    async fn auth(&self, req: &proto::AuthRequest) -> Result<SessionId, Status> {
        match Uuid::parse_str(req.link_id.as_str()) {
            Ok(uuid) => {
                let session_uuid = Uuid::new_v4();
                println!("PublicUuidAuthenticator.auth() passed: {} -> {}", uuid, session_uuid);
                Ok(SessionId(session_uuid.to_string()))
            },
            Err(err) => {
                println!("PublicUuidAuthenticator.auth() failed: {} -> {}", req.link_id, err);
                Err(Status::permission_denied(ErrorMessage::INVALID_LINK_ID))
            }
        }
    }
}

impl proto::AuthResponse {
    pub fn new(req: &proto::AuthRequest, session_id: &SessionId) -> Self {
        Self {
            ord: req.ord,
            session_id: session_id.0.clone(),
            trace_id: "TODO".to_owned(),
            extra: None,
        }
    }
}

impl proto::AppResponse {
    pub fn new(req: &proto::AppRequest, data: Option<Any>) -> Self {
        Self {
            ord: req.ord,
            session_id: req.session_id.clone(),
            trace_id: "TODO".to_owned(),
            data,
        }
    }
}
