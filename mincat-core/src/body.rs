use crate::error::{BoxError, Error};

use bytes::Bytes;
use futures_util::Stream;
use http_body::Body as _;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use std::{any::Any, pin::Pin, task::{Context, Poll}};

const BODY_LIMITED: usize = 1024 * 1204 * 2;

#[derive(Debug, Default, Clone, Copy)]
pub struct BodyLimitedSize(pub usize);

impl BodyLimitedSize {
    pub fn new() -> Self {
        Self(BODY_LIMITED)
    }
}

fn boxed<B>(body: B) -> BoxBody<Bytes, Error>
where
    B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
    B::Error: Into<BoxError>,
{
    try_downcast(body).unwrap_or_else(|body| body.map_err(Error::new).boxed())
}

pub(crate) fn try_downcast<T, K>(k: K) -> Result<T, K>
where
    T: 'static,
    K: Send + 'static,
{
    let mut k = Some(k);
    if let Some(k) = <dyn Any>::downcast_mut::<Option<T>>(&mut k) {
        Ok(k.take().unwrap())
    } else {
        Err(k.unwrap())
    }
}

pub struct Body(BoxBody<Bytes, Error>);

impl Body {
    pub fn new<B>(body: B) -> Self
    where
        B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
        B::Error: Into<BoxError>,
    {
        try_downcast(body).unwrap_or_else(|body| Self(boxed(body)))
    }

    pub fn empty() -> Self {
        Self::new(Empty::new())
    }
}

impl http_body::Body for Body {
    type Data = Bytes;
    type Error = Error;

    #[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        Pin::new(&mut self.0).poll_frame(cx)
    }

    #[inline]
    fn size_hint(&self) -> http_body::SizeHint {
        self.0.size_hint()
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.0.is_end_stream()
    }
}

impl Stream for Body {
    type Item = Result<Bytes, Error>;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match futures_util::ready!(Pin::new(&mut self.0).poll_frame(cx)?) {
                Some(frame) => match frame.into_data() {
                    Ok(data) => return Poll::Ready(Some(Ok(data))),
                    Err(_frame) => {}
                },
                None => return Poll::Ready(None),
            }
        }
    }
}

macro_rules! body_from_impl {
    ($ty:ty) => {
        impl From<$ty> for Body {
            fn from(value: $ty) -> Self {
                Self::new(Full::from(value))
            }
        }
    };
}

body_from_impl!(&'static str);
body_from_impl!(String);
body_from_impl!(Bytes);
