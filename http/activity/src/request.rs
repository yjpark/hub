use indexmap::IndexMap;

use crate::HttpBody;
use crate::HttpUri;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct HttpRequest {
    pub method: String,
    pub uri: HttpUri,
    pub version: String,
    pub headers: IndexMap<String, Vec<String>>,
    pub body: HttpBody,
    pub notes: Vec<String>,
}

impl HttpRequest {
    pub fn to_method(method: &http::Method) -> String {
        method.to_string()
    }

    pub fn to_version(version: http::Version) -> String {
        format!("{:?}", version)
    }

    pub fn to_headers(headers: &http::HeaderMap, notes: &mut Vec<String>) -> IndexMap<String, Vec<String>> {
        let mut result = IndexMap::new();
        for key in headers.keys() {
            let mut values = Vec::new();
            for value in headers.get_all(key) {
                match value.to_str() {
                    Ok(v) => {
                        values.push(v.to_owned());
                    },
                    Err(err) => {
                        notes.push(format!("INVALID_HEADER_VALUE: {:?} = {:?} -> {:?}", key, value, err));
                    }
                }
            }
            result.insert(key.to_string(), values);
        }
        result
    }
}

impl From<http::Request<()>> for HttpRequest {
    fn from(value: http::Request<()>) -> Self {
        let mut notes = Vec::new();
        Self {
            method: Self::to_method(value.method()),
            uri: value.uri().into(),
            version: Self::to_version(value.version()), 
            headers: Self::to_headers(value.headers(), &mut notes),
            body: HttpBody::None,
            notes,
        }
    }
}

impl From<http::Request<String>> for HttpRequest {
    fn from(value: http::Request<String>) -> Self {
        let mut notes = Vec::new();
        Self {
            method: Self::to_method(value.method()),
            uri: value.uri().into(),
            version: Self::to_version(value.version()), 
            headers: Self::to_headers(value.headers(), &mut notes),
            body: HttpBody::Text(value.body().clone()),
            notes,
        }
    }
}

#[cfg(feature = "rkyv")]
impl From<http::Request<Vec<u8>>> for HttpRequest {
    fn from(value: http::Request<Vec<u8>>) -> Self {
        let mut notes = Vec::new();
        Self {
            method: Self::to_method(value.method()),
            uri: value.uri().into(),
            version: Self::to_version(value.version()), 
            headers: Self::to_headers(value.headers(), &mut notes),
            body: HttpBody::Bytes(value.body().clone()),
            notes,
        }
    }
}

#[cfg(not(feature = "rkyv"))]
impl From<http::Request<bytes::Bytes>> for HttpRequest {
    fn from(value: http::Request<bytes::Bytes>) -> Self {
        let mut notes = Vec::new();
        Self {
            method: Self::to_method(value.method()),
            uri: value.uri().into(),
            version: Self::to_version(value.version()), 
            headers: Self::to_headers(value.headers(), &mut notes),
            body: HttpBody::Bytes(value.body().clone()),
            notes,
        }
    }
}