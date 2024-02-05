#[cfg(feature = "cookie")]
pub mod cookie;

#[cfg(feature = "form")]
mod form_data;
#[cfg(feature = "form")]
mod form_urlencoded;
#[cfg(feature = "form")]
pub mod form {
    pub use super::form_data::{FormData, FromMultipartNull};
    pub use super::form_urlencoded::FormUrlencoded;
    pub use mincat_macro::Form;
    pub use multer::Multipart;
    pub use multer_derive::{Error, FormContext, FormFile, FromMultipart, MultipartForm};
}

use http::StatusCode;
use mincat_core::response::{IntoResponse, Response};
use std::{error::Error, fmt::Display};

#[cfg(feature = "json")]
mod json;
#[cfg(feature = "json")]
pub use json::Json;

#[cfg(feature = "path")]
mod path;
#[cfg(feature = "path")]
pub use path::Path;

#[cfg(feature = "query")]
mod query;
#[cfg(feature = "query")]
pub use query::Query;

#[cfg(feature = "session")]
mod session;
#[cfg(feature = "session")]
pub use session::Session;

#[cfg(feature = "state")]
mod state;
#[cfg(feature = "state")]
pub use state::State;

#[cfg(feature = "websocket")]
pub mod websocket;

#[derive(Debug)]
pub struct ExtractError(pub String);

impl Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ExtractError {}

impl serde::de::Error for ExtractError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        ExtractError(msg.to_string())
    }
}

impl ExtractError {
    pub fn from<T: std::error::Error>(e: T) -> Self {
        Self(e.to_string())
    }
}

impl IntoResponse for ExtractError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.0).into_response()
    }
}
