use std::{collections::{HashMap, HashSet}, fmt::Display};

use crate::{proto, hub_app::{AppId, AppLink}};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinkId(pub String);

impl From<&str> for LinkId {
    fn from(v: &str) -> Self {
        Self(v.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(pub String);

impl From<&str> for SessionId {
    fn from(v: &str) -> Self {
        Self(v.to_owned())
    }
}

#[derive(Debug)]
pub struct LinkSession {
    pub link: AppLink,
    pub last_ord: u64,
    pub subscriptions: HashSet<String>,
    pub pending_requests: HashMap<u64, proto::AppRequest>,
}

impl Display for LinkSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<LinkSession>({}->{}:{}-{})", self.link, self.last_ord, self.subscriptions.len(), self.pending_requests.len())
    }
}

impl LinkSession {
    pub fn new(link: AppLink, last_ord: u64) -> Self {
        Self {
            link,
            last_ord,
            subscriptions: HashSet::new(),
            pending_requests: HashMap::new(),
        }
    }
}