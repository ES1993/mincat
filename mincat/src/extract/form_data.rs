use http::header;
use mincat_core::request::{FromRequest, Request, RequestExt};
use multer::{parse_boundary, Field};

use super::ExtractError;

pub struct FormData(multer::Multipart<'static>);

#[async_trait::async_trait]
impl FromRequest for FormData {
    type Error = ExtractError;

    async fn from_request(request: Request) -> Result<Self, Self::Error> {
        let boundary = parse_boundary(
            request
                .headers()
                .get(header::CONTENT_TYPE)
                .ok_or(ExtractError("missing content type".to_string()))?
                .to_str()
                .ok()
                .ok_or(ExtractError("content type ".to_string()))?,
        )
        .map_err(ExtractError::from)?;

        let body = request.change_to_limited_body().into_body();

        let multipart = multer::Multipart::new(body, boundary);

        Ok(FormData(multipart))
    }
}

impl FormData {
    pub async fn next_field(&mut self) -> Result<Option<Field<'static>>, ExtractError> {
        self.0.next_field().await.map_err(ExtractError::from)
    }
}
