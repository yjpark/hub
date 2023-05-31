use indexmap::IndexMap;

use crate::body::HttpBody;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct HttpResponse {
    pub status: u16,
    pub version: String,
    pub headers: IndexMap<String, String>,
    pub body: HttpBody,
}