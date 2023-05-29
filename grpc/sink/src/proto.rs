use prost_types::Any;
pub use crate::link::*;

tonic::include_proto!("edger.hub.v1.sink");

impl SinkError {
    pub fn auth(link_req: &LinkAuthRequest, code: i32, message: &str, details: Option<Any>) -> Self {
        let req = link_req.app_req.as_ref().unwrap();
        Self {
            ord: req.ord,
            session_id: link_req.link_session.clone(),
            trace_id: link_req.trace_id.clone(),
            code: code,
            message: message.to_owned(),
            details,
        }
    }
    pub fn app(link_req: &LinkAppRequest, code: i32, message: &str, details: Option<Any>) -> Self {
        let req = link_req.app_req.as_ref().unwrap();
        Self {
            ord: req.ord,
            session_id: req.session_id.clone(),
            trace_id: link_req.trace_id.clone(),
            code: code,
            message: message.to_owned(),
            details,
        }
    }
    pub fn init() -> Self {
        Self {
            ord: 0,
            session_id: "".to_owned(),
            trace_id: "".to_owned(),
            code: 0,
            message: "".to_owned(),
            details: None,
        }
    }
}

impl AuthResult {
    pub fn ok(session_id: &str, res: AuthResponse) -> Self {
        Self {
            sink_session: session_id.to_owned(),
            result: Some(auth_result::Result::Ok(res)),
        }
    }
    pub fn err(session_id: &str, err: SinkError) -> Self {
        Self {
            sink_session: session_id.to_owned(),
            result: Some(auth_result::Result::Err(err)),
        }
    }
    pub fn init(res: &RegisterResponse) -> Self {
        Self::err(&res.session_id, SinkError::init())
    }
}

impl AuthResponse {
    pub fn new(link_req: &LinkAuthRequest) -> Self {
        let req = link_req.app_req.as_ref().unwrap();
        Self {
            ord: req.ord,
            session_id: link_req.link_session.clone(),
            trace_id: link_req.trace_id.clone(),
            extra: None,
        }
    }
}

impl HandleResult {
    pub fn ok(session_id: &str, res: AppResponse) -> Self {
        Self {
            sink_session: session_id.to_owned(),
            result: Some(handle_result::Result::Ok(res)),
        }
    }
    pub fn err(session_id: &str, err: SinkError) -> Self {
        Self {
            sink_session: session_id.to_owned(),
            result: Some(handle_result::Result::Err(err)),
        }
    }
    pub fn init(res: &RegisterResponse) -> Self {
        Self::err(&res.session_id, SinkError::init())
    }
}
