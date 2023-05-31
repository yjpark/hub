#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub enum HttpError {
    Timeout,
    // Http { source: http::Error },
    Http { source: String },
}

impl HttpError {
    /// Returns `true` if the http error is [`Timeout`].
    ///
    /// [`Timeout`]: HttpError::Timeout
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout)
    }

    /// Returns `true` if the http error is [`Http`].
    ///
    /// [`Http`]: HttpError::Http
    #[must_use]
    pub fn is_http(&self) -> bool {
        matches!(self, Self::Http { .. })
    }

    pub fn as_http(&self) -> Option<&String> {
        if let Self::Http { source } = self {
            Some(source)
        } else {
            None
        }
    }
}

impl From<http::Error> for HttpError {
    fn from(value: http::Error) -> Self {
        Self::Http {
            source: value.to_string(),
        }
    }
}