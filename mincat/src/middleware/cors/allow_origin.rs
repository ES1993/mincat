use std::{fmt, sync::Arc};

use http::{
    header::{self, HeaderName, HeaderValue},
    request::Parts as RequestParts,
};

use super::{Any, WILDCARD};

#[derive(Clone, Default)]
#[must_use]
pub struct AllowOrigin(OriginInner);

impl AllowOrigin {
    pub fn any() -> Self {
        Self(OriginInner::Const(WILDCARD))
    }

    pub fn exact(origin: HeaderValue) -> Self {
        Self(OriginInner::Const(origin))
    }

    #[allow(clippy::borrow_interior_mutable_const)]
    pub fn list<I>(origins: I) -> Self
    where
        I: IntoIterator<Item = HeaderValue>,
    {
        let origins = origins.into_iter().collect::<Vec<_>>();
        if origins.iter().any(|o| o == WILDCARD) {
            panic!("Wildcard origin (`*`) cannot be passed to `AllowOrigin::list`. Use `AllowOrigin::any()` instead");
        } else {
            Self(OriginInner::List(origins))
        }
    }

    pub fn predicate<F>(f: F) -> Self
    where
        F: Fn(&HeaderValue, &RequestParts) -> bool + Send + Sync + 'static,
    {
        Self(OriginInner::Predicate(Arc::new(f)))
    }

    pub fn mirror_request() -> Self {
        Self::predicate(|_, _| true)
    }

    #[allow(clippy::borrow_interior_mutable_const)]
    pub(super) fn is_wildcard(&self) -> bool {
        matches!(&self.0, OriginInner::Const(v) if v == WILDCARD)
    }

    pub(super) fn to_header(
        &self,
        origin: Option<&HeaderValue>,
        parts: &RequestParts,
    ) -> Option<(HeaderName, HeaderValue)> {
        let allow_origin = match &self.0 {
            OriginInner::Const(v) => v.clone(),
            OriginInner::List(l) => origin.filter(|o| l.contains(o))?.clone(),
            OriginInner::Predicate(c) => origin.filter(|origin| c(origin, parts))?.clone(),
        };

        Some((header::ACCESS_CONTROL_ALLOW_ORIGIN, allow_origin))
    }
}

impl fmt::Debug for AllowOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            OriginInner::Const(inner) => f.debug_tuple("Const").field(inner).finish(),
            OriginInner::List(inner) => f.debug_tuple("List").field(inner).finish(),
            OriginInner::Predicate(_) => f.debug_tuple("Predicate").finish(),
        }
    }
}

impl From<Any> for AllowOrigin {
    fn from(_: Any) -> Self {
        Self::any()
    }
}

impl From<HeaderValue> for AllowOrigin {
    fn from(val: HeaderValue) -> Self {
        Self::exact(val)
    }
}

impl<const N: usize> From<[HeaderValue; N]> for AllowOrigin {
    fn from(arr: [HeaderValue; N]) -> Self {
        Self::list(IntoIterator::into_iter(arr))
    }
}

impl From<Vec<HeaderValue>> for AllowOrigin {
    fn from(vec: Vec<HeaderValue>) -> Self {
        Self::list(vec)
    }
}

type PredicateParam =
    Arc<dyn for<'a> Fn(&'a HeaderValue, &'a RequestParts) -> bool + Send + Sync + 'static>;

#[derive(Clone)]
enum OriginInner {
    Const(HeaderValue),
    List(Vec<HeaderValue>),
    Predicate(PredicateParam),
}

impl Default for OriginInner {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}
