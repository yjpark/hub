use prost::Message;
use prost_types::Any;

use hub_grpc_link::{proto::AppRequest, hub_session::SessionId};

use crate::echo::{EchoKinds, EchoRequest};

impl EchoRequest {
    pub const TYPE_URL: &'static str = "/edger.hub.v1.echo.EchoRequest";

    pub fn to_app(&self, ord: u64, session_id: &SessionId) -> AppRequest {
        let mut buffer = Vec::new();
        buffer.reserve(self.encoded_len());
        self.encode(&mut buffer).unwrap();
        AppRequest {
            ord: ord, 
            session_id: session_id.0.clone(), 
            kind: EchoKinds::Echo as i32,
            data: Some(Any{
                type_url: Self::TYPE_URL.to_owned(),
                value: buffer,
            }),
        }
    }
}