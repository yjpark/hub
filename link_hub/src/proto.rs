tonic::include_proto!("edger.hub.v1.link");
use crate::link_session::SessionId;
use prost_types::Any;

impl AuthResponse {
    pub fn new(req: &AuthRequest, session_id: &SessionId) -> Self {
        Self {
            ord: req.ord,
            session_id: session_id.0.clone(),
            trace_id: "TODO".to_owned(),
            extra: None,
        }
    }
}

impl AppResponse {
    pub fn new(req: &AppRequest, data: Option<Any>) -> Self {
        Self {
            ord: req.ord,
            session_id: req.session_id.clone(),
            trace_id: "TODO".to_owned(),
            kind: req.kind,
            data,
        }
    }
}
