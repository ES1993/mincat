use std::{fmt, sync::Arc};

use http::{
    header::{HeaderName, HeaderValue},
    request::Parts as RequestParts,
};

#[derive(Clone, Default)]
#[must_use]
pub struct AllowPrivateNetwork(AllowPrivateNetworkInner);

impl AllowPrivateNetwork {
    pub fn yes() -> Self {
        Self(AllowPrivateNetworkInner::Yes)
    }

    pub fn predicate<F>(f: F) -> Self
    where
        F: Fn(&HeaderValue, &RequestParts) -> bool + Send + Sync + 'static,
    {
        Self(AllowPrivateNetworkInner::Predicate(Arc::new(f)))
    }

    #[allow(
        clippy::declare_interior_mutable_const,
        clippy::borrow_interior_mutable_const
    )]
    pub(super) fn to_header(
        &self,
        origin: Option<&HeaderValue>,
        parts: &RequestParts,
    ) -> Option<(HeaderName, HeaderValue)> {
        #[allow(clippy::declare_interior_mutable_const)]
        const REQUEST_PRIVATE_NETWORK: HeaderName =
            HeaderName::from_static("access-control-request-private-network");

        #[allow(clippy::declare_interior_mutable_const)]
        const ALLOW_PRIVATE_NETWORK: HeaderName =
            HeaderName::from_static("access-control-allow-private-network");

        const TRUE: HeaderValue = HeaderValue::from_static("true");

        if let AllowPrivateNetworkInner::No = &self.0 {
            return None;
        }

        if parts.headers.get(REQUEST_PRIVATE_NETWORK) != Some(&TRUE) {
            return None;
        }

        let allow_private_network = match &self.0 {
            AllowPrivateNetworkInner::Yes => true,
            AllowPrivateNetworkInner::No => false,
            AllowPrivateNetworkInner::Predicate(c) => c(origin?, parts),
        };

        allow_private_network.then_some((ALLOW_PRIVATE_NETWORK, TRUE))
    }
}

impl From<bool> for AllowPrivateNetwork {
    fn from(v: bool) -> Self {
        match v {
            true => Self(AllowPrivateNetworkInner::Yes),
            false => Self(AllowPrivateNetworkInner::No),
        }
    }
}

impl fmt::Debug for AllowPrivateNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            AllowPrivateNetworkInner::Yes => f.debug_tuple("Yes").finish(),
            AllowPrivateNetworkInner::No => f.debug_tuple("No").finish(),
            AllowPrivateNetworkInner::Predicate(_) => f.debug_tuple("Predicate").finish(),
        }
    }
}

type PredicateParam =
    Arc<dyn for<'a> Fn(&'a HeaderValue, &'a RequestParts) -> bool + Send + Sync + 'static>;

#[derive(Clone)]
enum AllowPrivateNetworkInner {
    Yes,
    No,
    Predicate(PredicateParam),
}

impl Default for AllowPrivateNetworkInner {
    fn default() -> Self {
        Self::No
    }
}
