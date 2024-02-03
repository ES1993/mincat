use std::{fmt, sync::Arc, time::Duration};

use http::{
    header::{self, HeaderName, HeaderValue},
    request::Parts as RequestParts,
};

#[derive(Clone, Default)]
#[must_use]
pub struct MaxAge(MaxAgeInner);

impl MaxAge {
    pub fn exact(max_age: Duration) -> Self {
        Self(MaxAgeInner::Exact(Some(max_age.as_secs().into())))
    }

    pub fn dynamic<F>(f: F) -> Self
    where
        F: Fn(&HeaderValue, &RequestParts) -> Duration + Send + Sync + 'static,
    {
        Self(MaxAgeInner::Fn(Arc::new(f)))
    }

    pub(super) fn to_header(
        &self,
        origin: Option<&HeaderValue>,
        parts: &RequestParts,
    ) -> Option<(HeaderName, HeaderValue)> {
        let max_age = match &self.0 {
            MaxAgeInner::Exact(v) => v.clone()?,
            MaxAgeInner::Fn(c) => c(origin?, parts).as_secs().into(),
        };

        Some((header::ACCESS_CONTROL_MAX_AGE, max_age))
    }
}

impl fmt::Debug for MaxAge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            MaxAgeInner::Exact(inner) => f.debug_tuple("Exact").field(inner).finish(),
            MaxAgeInner::Fn(_) => f.debug_tuple("Fn").finish(),
        }
    }
}

impl From<Duration> for MaxAge {
    fn from(max_age: Duration) -> Self {
        Self::exact(max_age)
    }
}
type FnParam =
    Arc<dyn for<'a> Fn(&'a HeaderValue, &'a RequestParts) -> Duration + Send + Sync + 'static>;

#[derive(Clone)]
enum MaxAgeInner {
    Exact(Option<HeaderValue>),
    Fn(FnParam),
}

impl Default for MaxAgeInner {
    fn default() -> Self {
        Self::Exact(None)
    }
}
