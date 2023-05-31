use chrono::prelude::{DateTime, Utc};
use indexmap::IndexMap;
use uuid::Uuid;

use crate::activity::HttpActivity;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct HttpSession {
    pub begin_time: DateTime<Utc>,
    pub activities: IndexMap<Uuid, HttpActivity>,
    pub end_time: Option<DateTime<Utc>>,
}