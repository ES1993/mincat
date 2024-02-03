use std::fmt;

use http::{
    header::{self, HeaderName, HeaderValue},
    request::Parts as RequestParts,
};

use super::{separated_by_commas, Any, WILDCARD};

#[derive(Clone, Default)]
#[must_use]
pub struct ExposeHeaders(ExposeHeadersInner);

impl ExposeHeaders {
    pub fn any() -> Self {
        Self(ExposeHeadersInner::Const(Some(WILDCARD)))
    }

    pub fn list<I>(headers: I) -> Self
    where
        I: IntoIterator<Item = HeaderName>,
    {
        Self(ExposeHeadersInner::Const(separated_by_commas(
            headers.into_iter().map(Into::into),
        )))
    }

    #[allow(clippy::borrow_interior_mutable_const)]
    pub(super) fn is_wildcard(&self) -> bool {
        matches!(&self.0, ExposeHeadersInner::Const(Some(v)) if v == WILDCARD)
    }

    pub(super) fn to_header(&self, _parts: &RequestParts) -> Option<(HeaderName, HeaderValue)> {
        let expose_headers = match &self.0 {
            ExposeHeadersInner::Const(v) => v.clone()?,
        };

        Some((header::ACCESS_CONTROL_EXPOSE_HEADERS, expose_headers))
    }
}

impl fmt::Debug for ExposeHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ExposeHeadersInner::Const(inner) => f.debug_tuple("Const").field(inner).finish(),
        }
    }
}

impl From<Any> for ExposeHeaders {
    fn from(_: Any) -> Self {
        Self::any()
    }
}

impl<const N: usize> From<[HeaderName; N]> for ExposeHeaders {
    fn from(arr: [HeaderName; N]) -> Self {
        Self::list(IntoIterator::into_iter(arr))
    }
}

impl From<Vec<HeaderName>> for ExposeHeaders {
    fn from(vec: Vec<HeaderName>) -> Self {
        Self::list(vec)
    }
}

#[derive(Clone)]
enum ExposeHeadersInner {
    Const(Option<HeaderValue>),
}

impl Default for ExposeHeadersInner {
    fn default() -> Self {
        ExposeHeadersInner::Const(None)
    }
}
