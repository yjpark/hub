use snafu::{prelude::*, Whatever};
use std::io::Cursor;

use prost::Message;
use prost_types::Any;

use hub_sink::proto::{LinkAppRequest, AppResponse};

use crate::echo::{EchoKinds, EchoRequest, EchoResponse};

impl EchoRequest {
    pub fn from_app(req: &LinkAppRequest) -> Result<Self, Whatever> {
        let app_req = req.app_req.as_ref().unwrap();
        if let Some(data) = &app_req.data {
            return EchoRequest::decode(&mut Cursor::new(data.value.clone()))
                .with_whatever_context(|_| format!("decode failed, [{:?}]", app_req.kind));
        }
        whatever!("no data")
    }
}

impl EchoResponse {
    pub const TYPE_URL: &'static str = "/edger.hub.v1.echo.EchoResponse";

    pub fn from_app(req: &LinkAppRequest) -> Self {
        let echo_req = EchoRequest::from_app(req);
        let echo_req = echo_req.unwrap_or_else(|e|{
            EchoRequest {
                time: None,
                message: e.to_string(),
            }
        });
        Self {
            time: None,
            message: echo_req.message,
        }
    }

    pub fn to_app(&self, req: &LinkAppRequest) -> AppResponse {
        let app_req = req.app_req.as_ref().unwrap();
        let mut buffer = Vec::new();
        buffer.reserve(self.encoded_len());
        self.encode(&mut buffer).unwrap();
        AppResponse {
            ord: app_req.ord, 
            session_id: app_req.session_id.clone(), 
            trace_id: req.trace_id.clone(),
            kind: EchoKinds::Echo as i32,
            data: Some(Any{
                type_url: Self::TYPE_URL.to_owned(),
                value: buffer,
            }),
        }
    }
}

