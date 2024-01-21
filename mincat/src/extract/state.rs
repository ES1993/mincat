use std::any::type_name;

use mincat_core::request::{FromRequest, Request};

use super::ExtractError;

#[derive(Clone, Debug)]
pub struct State<T>(pub T);

#[async_trait::async_trait]
impl<T> FromRequest for State<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = ExtractError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        let state = request
            .extensions()
            .get::<T>()
            .ok_or(ExtractError(format!("missing state: {}", type_name::<T>())))?
            .clone();

        Ok(State(state))
    }
}
