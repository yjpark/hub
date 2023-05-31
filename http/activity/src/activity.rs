use chrono::prelude::{DateTime, Utc};
use uuid::Uuid;

use crate::{HttpRequest, HttpResponse, HttpError};

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct HttpActivity {
    pub uuid: Uuid,
    pub req_time: DateTime<Utc>,
    pub req: HttpRequest,
    pub res_time: Option<DateTime<Utc>>,
    pub res: Option<Result<HttpResponse, HttpError>>,
}
