use std::{collections::{HashMap, HashSet}, fmt::Display, sync::{RwLockReadGuard, RwLockWriteGuard, RwLock}};

use tonic::Status;

use crate::{proto, hub_app::{AppId, AppLink}, error::ErrorMessage};

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
    pub fn read_link_last_ord(x: RwLockReadGuard<LinkSession>) -> (AppLink, u64) {
        (x.link.clone(), x.last_ord)
    }
    pub fn update_last_ord(mut x: RwLockWriteGuard<LinkSession>, ord: u64) {
        x.last_ord = ord;
    }
    pub fn check_ord(session: &RwLock<LinkSession>, ord: u64) -> Result<AppLink, Status> {
        let (link, last_ord) = LinkSession::read_link_last_ord(session.read().unwrap());
        if ord <= last_ord {
            return Err(Status::aborted(ErrorMessage::INVALID_ORD));
        }
        LinkSession::update_last_ord(session.write().unwrap(), ord);
        Ok(link)
    }
}