use std::fmt;

use http::{
    header::{self, HeaderName, HeaderValue},
    request::Parts as RequestParts,
    Method,
};

use super::{separated_by_commas, Any, WILDCARD};

#[derive(Clone, Default)]
#[must_use]
pub struct AllowMethods(AllowMethodsInner);

impl AllowMethods {
    pub fn any() -> Self {
        Self(AllowMethodsInner::Const(Some(WILDCARD)))
    }

    pub fn exact(method: Method) -> Self {
        Self(AllowMethodsInner::Const(Some(
            HeaderValue::from_str(method.as_str()).unwrap(),
        )))
    }

    pub fn list<I>(methods: I) -> Self
    where
        I: IntoIterator<Item = Method>,
    {
        Self(AllowMethodsInner::Const(separated_by_commas(
            methods
                .into_iter()
                .map(|m| HeaderValue::from_str(m.as_str()).unwrap()),
        )))
    }

    pub fn mirror_request() -> Self {
        Self(AllowMethodsInner::MirrorRequest)
    }

    #[allow(clippy::borrow_interior_mutable_const)]
    pub(super) fn is_wildcard(&self) -> bool {
        matches!(&self.0, AllowMethodsInner::Const(Some(v)) if v == WILDCARD)
    }

    pub(super) fn to_header(&self, parts: &RequestParts) -> Option<(HeaderName, HeaderValue)> {
        let allow_methods = match &self.0 {
            AllowMethodsInner::Const(v) => v.clone()?,
            AllowMethodsInner::MirrorRequest => parts
                .headers
                .get(header::ACCESS_CONTROL_REQUEST_METHOD)?
                .clone(),
        };

        Some((header::ACCESS_CONTROL_ALLOW_METHODS, allow_methods))
    }
}

impl fmt::Debug for AllowMethods {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            AllowMethodsInner::Const(inner) => f.debug_tuple("Const").field(inner).finish(),
            AllowMethodsInner::MirrorRequest => f.debug_tuple("MirrorRequest").finish(),
        }
    }
}

impl From<Any> for AllowMethods {
    fn from(_: Any) -> Self {
        Self::any()
    }
}

impl From<Method> for AllowMethods {
    fn from(method: Method) -> Self {
        Self::exact(method)
    }
}

impl<const N: usize> From<[Method; N]> for AllowMethods {
    fn from(arr: [Method; N]) -> Self {
        Self::list(IntoIterator::into_iter(arr))
    }
}

impl From<Vec<Method>> for AllowMethods {
    fn from(vec: Vec<Method>) -> Self {
        Self::list(vec)
    }
}

#[derive(Clone)]
enum AllowMethodsInner {
    Const(Option<HeaderValue>),
    MirrorRequest,
}

impl Default for AllowMethodsInner {
    fn default() -> Self {
        Self::Const(None)
    }
}
