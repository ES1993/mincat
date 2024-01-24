use http::{header, StatusCode};
use http_body_util::BodyExt;
use mincat_core::{
    request::{FromRequest, Request, RequestExt},
    response::{IntoResponse, Response},
};
use serde::{de::DeserializeOwned, Serialize};

use super::ExtractError;

pub struct FormUrlencoded<T>(pub T);

#[async_trait::async_trait]
impl<T> FromRequest for FormUrlencoded<T>
where
    T: DeserializeOwned + Clone + Send + 'static,
{
    type Error = ExtractError;

    async fn from_request(request: Request) -> Result<Self, Self::Error> {
        let bytes = request
            .change_to_limited_body()
            .into_body()
            .collect()
            .await
            .map_err(|e| ExtractError(e.to_string()))?
            .to_bytes();

        let data: T = serde_urlencoded::from_bytes(&bytes).map_err(ExtractError::from)?;

        Ok(FormUrlencoded(data))
    }
}

impl<T> IntoResponse for FormUrlencoded<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match serde_urlencoded::to_string(&self.0) {
            Ok(body) => (
                StatusCode::OK,
                [(
                    header::CONTENT_TYPE,
                    mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                )],
                body,
            )
                .into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }
}
