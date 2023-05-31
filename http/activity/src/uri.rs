#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct HttpUri {
    pub schema: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
}

impl From<&http::Uri> for HttpUri {
    fn from(value: &http::Uri) -> Self {
        Self {
            schema: value.scheme_str().map(|x| x.to_string()),
            host: value.host().map(|x| x.to_string()),
            port: value.port().map(|x| x.as_u16()),
            path: value.path().to_string(),
            query: value.query().map(|x| x.to_string()),
        }
    }
}
