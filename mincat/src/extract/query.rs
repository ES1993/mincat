use mincat_core::request::{FromRequestParts, Parts};
use serde::de::DeserializeOwned;

use super::ExtractError;

pub struct Query<T>(pub T);

#[async_trait::async_trait]
impl<T> FromRequestParts for Query<T>
where
    T: DeserializeOwned + Clone + Send + 'static,
{
    type Error = ExtractError;

    async fn from_request_parts(parts: &mut Parts) -> Result<Self, Self::Error> {
        let query = parts.uri.query().unwrap_or("");
        let res: T = serde_qs::from_str(query).map_err(ExtractError::from)?;
        Ok(Query(res))
    }
}
