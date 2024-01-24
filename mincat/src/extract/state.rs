use std::any::type_name;

use mincat_core::request::{FromRequestParts, Parts};

use super::ExtractError;

pub struct State<T>(pub T);

#[async_trait::async_trait]
impl<T> FromRequestParts for State<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = ExtractError;

    async fn from_request_parts(parts: &mut Parts) -> Result<Self, Self::Error> {
        let state = parts
            .extensions
            .get::<T>()
            .ok_or(ExtractError(format!("missing state: {}", type_name::<T>())))?
            .clone();

        Ok(State(state))
    }
}
