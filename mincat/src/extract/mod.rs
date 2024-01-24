mod form_data;
mod form_urlencoded;
mod json;
mod path;
mod query;
mod state;

pub use form_data::FormData;
pub use form_urlencoded::FormUrlencoded;
pub use json::Json;
pub use path::Path;
pub use query::Query;
pub use state::State;

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
