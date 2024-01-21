use bytes::{BufMut, Bytes, BytesMut};
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Incoming;
use std::error::Error;

pub type BoxBodyError = Box<dyn Error + Send + Sync>;
pub struct Body(BoxBody<Bytes, BoxBodyError>);

impl Body {
    pub fn incoming(incoming: Incoming) -> Self {
        Body(incoming.boxed().map_err(Into::into).boxed())
    }

    pub fn box_body(body: Body) -> BoxBody<Bytes, BoxBodyError> {
        body.0
    }

    pub fn empty() -> Self {
        Body(Empty::new().map_err(Into::into).boxed())
    }

    pub async fn bytes(&mut self) -> Result<Bytes, BoxBodyError> {
        let mut res = BytesMut::new();
        while let Some(frame) = self.0.frame().await.transpose()? {
            if let Ok(bytes) = frame.into_data() {
                res.put(bytes);
            }
        }
        Ok(res.freeze())
    }
}

macro_rules! body_from_impl {
    ($ty:ty) => {
        impl From<$ty> for Body {
            fn from(value: $ty) -> Self {
                Body(full(value))
            }
        }
    };
}

body_from_impl!(&'static str);
body_from_impl!(String);
body_from_impl!(Bytes);

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, BoxBodyError> {
    Full::new(chunk.into()).map_err(Into::into).boxed()
}
