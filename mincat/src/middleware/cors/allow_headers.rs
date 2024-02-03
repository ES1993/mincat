use std::fmt;

use http::{
    header::{self, HeaderName, HeaderValue},
    request::Parts as RequestParts,
};

use super::{separated_by_commas, Any, WILDCARD};

#[derive(Clone, Default)]
#[must_use]
pub struct AllowHeaders(AllowHeadersInner);

impl AllowHeaders {
    pub fn any() -> Self {
        Self(AllowHeadersInner::Const(Some(WILDCARD)))
    }

    pub fn list<I>(headers: I) -> Self
    where
        I: IntoIterator<Item = HeaderName>,
    {
        Self(AllowHeadersInner::Const(separated_by_commas(
            headers.into_iter().map(Into::into),
        )))
    }

    pub fn mirror_request() -> Self {
        Self(AllowHeadersInner::MirrorRequest)
    }

    #[allow(clippy::borrow_interior_mutable_const)]
    pub(super) fn is_wildcard(&self) -> bool {
        matches!(&self.0, AllowHeadersInner::Const(Some(v)) if v == WILDCARD)
    }

    pub(super) fn to_header(&self, parts: &RequestParts) -> Option<(HeaderName, HeaderValue)> {
        let allow_headers = match &self.0 {
            AllowHeadersInner::Const(v) => v.clone()?,
            AllowHeadersInner::MirrorRequest => parts
                .headers
                .get(header::ACCESS_CONTROL_REQUEST_HEADERS)?
                .clone(),
        };

        Some((header::ACCESS_CONTROL_ALLOW_HEADERS, allow_headers))
    }
}

impl fmt::Debug for AllowHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            AllowHeadersInner::Const(inner) => f.debug_tuple("Const").field(inner).finish(),
            AllowHeadersInner::MirrorRequest => f.debug_tuple("MirrorRequest").finish(),
        }
    }
}

impl From<Any> for AllowHeaders {
    fn from(_: Any) -> Self {
        Self::any()
    }
}

impl<const N: usize> From<[HeaderName; N]> for AllowHeaders {
    fn from(arr: [HeaderName; N]) -> Self {
        Self::list(IntoIterator::into_iter(arr))
    }
}

impl From<Vec<HeaderName>> for AllowHeaders {
    fn from(vec: Vec<HeaderName>) -> Self {
        Self::list(vec)
    }
}

#[derive(Clone)]
enum AllowHeadersInner {
    Const(Option<HeaderValue>),
    MirrorRequest,
}

impl Default for AllowHeadersInner {
    fn default() -> Self {
        Self::Const(None)
    }
}
