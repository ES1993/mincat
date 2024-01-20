use mincat_core::request::{FromRequest, Request};
use serde::de::DeserializeOwned;

use super::ExtractError;

#[derive(Clone, Debug)]
pub struct Query<T>(pub T);

#[async_trait::async_trait]
impl<T> FromRequest for Query<T>
where
    T: DeserializeOwned + Clone + Send + 'static,
{
    type Error = ExtractError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        let query = request.uri().query().unwrap_or("");
        let res: T = serde_qs::from_str(query).map_err(ExtractError::from)?;
        Ok(Query(res))
    }
}
