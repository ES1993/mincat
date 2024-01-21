use bytes::{BufMut, BytesMut};
use http::{header, HeaderValue, StatusCode};
use mincat_core::{
    request::{FromRequest, Request},
    response::{IntoResponse, Response},
};
use serde::{de::DeserializeOwned, Serialize};

use super::ExtractError;

#[derive(Clone, Debug)]
pub struct Json<T>(pub T);

#[async_trait::async_trait]
impl<T> FromRequest for Json<T>
where
    T: DeserializeOwned + Clone + Send + 'static,
{
    type Error = ExtractError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        let bytes = request
            .body_mut()
            .bytes()
            .await
            .map_err(|e| ExtractError(e.to_string()))?;

        let data: T = serde_json::from_slice(&bytes).map_err(ExtractError::from)?;

        Ok(Json(data))
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize + Clone + Send + 'static,
{
    fn into_response(self) -> Response {
        let mut buf = BytesMut::with_capacity(128).writer();
        match serde_json::to_writer(&mut buf, &self.0) {
            Ok(_) => (
                StatusCode::OK,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}
