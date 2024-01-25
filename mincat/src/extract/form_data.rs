use super::ExtractError;
use http::header;
use mincat_core::request::{FromRequest, Request, RequestExt};
use multer::{parse_boundary, Field};

use multer_derive::{FromMultipart, MultipartForm};

pub trait FromMultipartNull {
    fn is_null() -> bool;
    fn generate(multipart: multer::Multipart<'static>) -> Self;
}

pub struct Null(multer::Multipart<'static>);

impl FromMultipart for Null {
    fn from_multipart(
        _: &multer_derive::MultipartForm,
        _: multer_derive::FormContext<'_>,
    ) -> Result<Self, multer_derive::Error> {
        todo!()
    }
}

impl FromMultipartNull for Null {
    fn is_null() -> bool {
        true
    }

    fn generate(multipart: multer::Multipart<'static>) -> Self {
        Self(multipart)
    }
}

pub struct FormData<T = Null>(pub T)
where
    T: FromMultipart + FromMultipartNull;

#[async_trait::async_trait]
impl<T> FromRequest for FormData<T>
where
    T: FromMultipart + FromMultipartNull + 'static,
{
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

        if T::is_null() {
            let multipart = multer::Multipart::new(body, boundary);
            let res = T::generate(multipart);
            Ok(FormData(res))
        } else {
            let multipart = multer_derive::multer::Multipart::new(body, boundary);
            let form = MultipartForm::with_multipart(multipart)
                .await
                .map_err(ExtractError::from)?;
            let res = T::from_multipart(&form, Default::default()).map_err(ExtractError::from)?;
            Ok(FormData(res))
        }
    }
}

impl FormData<Null> {
    pub async fn next_field(&mut self) -> Result<Option<Field<'static>>, ExtractError> {
        self.0 .0.next_field().await.map_err(ExtractError::from)
    }
}
