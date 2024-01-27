use std::{error::Error as StdError, fmt};

use http::StatusCode;

use crate::response::IntoResponse;

pub type BoxError = Box<dyn StdError + Send + Sync>;

#[derive(Debug)]
pub struct Error {
    inner: BoxError,
}

impl Error {
    pub fn new(error: impl Into<BoxError>) -> Self {
        Self {
            inner: error.into(),
        }
    }

    pub fn into_inner(self) -> BoxError {
        self.inner
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.inner)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> crate::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.inner.to_string()).into_response()
    }
}
