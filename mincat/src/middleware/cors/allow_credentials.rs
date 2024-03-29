use std::{fmt, sync::Arc};

use http::{
    header::{self, HeaderName, HeaderValue},
    request::Parts as RequestParts,
};

#[derive(Clone, Default)]
#[must_use]
pub struct AllowCredentials(AllowCredentialsInner);

impl AllowCredentials {
    pub fn yes() -> Self {
        Self(AllowCredentialsInner::Yes)
    }

    pub fn predicate<F>(f: F) -> Self
    where
        F: Fn(&HeaderValue, &RequestParts) -> bool + Send + Sync + 'static,
    {
        Self(AllowCredentialsInner::Predicate(Arc::new(f)))
    }

    pub(super) fn is_true(&self) -> bool {
        matches!(&self.0, AllowCredentialsInner::Yes)
    }

    pub(super) fn to_header(
        &self,
        origin: Option<&HeaderValue>,
        parts: &RequestParts,
    ) -> Option<(HeaderName, HeaderValue)> {
        #[allow(clippy::declare_interior_mutable_const)]
        const TRUE: HeaderValue = HeaderValue::from_static("true");

        let allow_creds = match &self.0 {
            AllowCredentialsInner::Yes => true,
            AllowCredentialsInner::No => false,
            AllowCredentialsInner::Predicate(c) => c(origin?, parts),
        };

        allow_creds.then_some((header::ACCESS_CONTROL_ALLOW_CREDENTIALS, TRUE))
    }
}

impl From<bool> for AllowCredentials {
    fn from(v: bool) -> Self {
        match v {
            true => Self(AllowCredentialsInner::Yes),
            false => Self(AllowCredentialsInner::No),
        }
    }
}

impl fmt::Debug for AllowCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            AllowCredentialsInner::Yes => f.debug_tuple("Yes").finish(),
            AllowCredentialsInner::No => f.debug_tuple("No").finish(),
            AllowCredentialsInner::Predicate(_) => f.debug_tuple("Predicate").finish(),
        }
    }
}

type PredicateParam =
    Arc<dyn for<'a> Fn(&'a HeaderValue, &'a RequestParts) -> bool + Send + Sync + 'static>;

#[derive(Clone)]
enum AllowCredentialsInner {
    Yes,
    No,
    Predicate(PredicateParam),
}

impl Default for AllowCredentialsInner {
    fn default() -> Self {
        Self::No
    }
}
