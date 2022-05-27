pub use link_hub::proto::*;
use prost_types::Any;

use crate::sink_session::SessionId;

tonic::include_proto!("edger.hub.v1.sink");

impl RegisterResponse {
    pub fn new(req: &RegisterRequest, session_id: &SessionId, extra: Option<Any>) -> Self {
        Self {
            ord: req.ord,
            trace_id: "TODO".to_owned(),
            session_id: session_id.0.clone(),
            extra,
        }
    }
}

impl LinkAuthRequest {
    pub fn new(trace_id: &str, link_address: &str, link_session: &str, req: &AuthRequest) -> Self {
        Self {
            trace_id: trace_id.to_owned(), 
            link_address: link_address.to_owned(),
            link_session: link_session.to_owned(),
            req: Some(req.clone()),
        }
    }
}

impl LinkAppRequest {
    pub fn new(trace_id: &str, link_address: &str, req: &AppRequest) -> Self {
        Self {
            trace_id: trace_id.to_owned(),
            link_address: link_address.to_owned(),
            req: Some(req.clone()),
        }
    }
}