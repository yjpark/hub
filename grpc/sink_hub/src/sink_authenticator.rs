use async_trait::async_trait;
use tonic::{Status};
use uuid::Uuid;

use hub_grpc_link_hub::link_session::SessionId;
use crate::error::ErrorMessage;
use crate::proto;

#[async_trait]
pub trait SinkAuthenticator {
    fn allow_create_app(&self) -> bool {
        true
    }
    fn allow_multiple_sessions(&self) -> bool {
        false
    }
    fn kick_old_sessions(&self) -> bool {
        true
    }
    async fn auth(&self, req: &proto::RegisterRequest) -> Result<SessionId, Status>;    
}

pub type PublicUuidAuthenticator = hub_grpc_link_hub::link_authenticator::PublicUuidAuthenticator;

#[async_trait]
impl SinkAuthenticator for PublicUuidAuthenticator {
    async fn auth(&self, req: &proto::RegisterRequest) -> Result<SessionId, Status> {
        match Uuid::parse_str(req.sink_id.as_str()) {
            Ok(uuid) => {
                let session_uuid = Uuid::new_v4();
                println!("PublicUuidAuthenticator.auth() passed: {} -> {}", uuid, session_uuid);
                Ok(SessionId(session_uuid.to_string()))
            },
            Err(err) => {
                println!("PublicUuidAuthenticator.auth() failed: {} -> {}", req.sink_id, err);
                Err(Status::permission_denied(ErrorMessage::INVALID_SINK_ID))
            }
        }
    }
}