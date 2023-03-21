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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SinkId(pub String);

impl From<&str> for SinkId {
    fn from(v: &str) -> Self {
        Self(v.to_owned())
    }
}
