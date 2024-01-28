#[cfg(feature = "cookie")]
pub mod cookie;
mod form_data;
mod form_urlencoded;
mod json;
mod path;
mod query;
mod session;
mod state;

pub use json::Json;
pub use path::Path;
pub use query::Query;
#[cfg(feature = "session")]
pub use session::Session;
pub(crate) use session::SessionStore;
pub use state::State;

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
