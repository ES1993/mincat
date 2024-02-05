#[cfg(feature = "session")]
pub mod session;

#[cfg(feature = "cors")]
pub mod cors;

#[cfg(feature = "body-limit")]
mod body_limit;
#[cfg(feature = "body-limit")]
pub use body_limit::BodyLimit;

#[cfg(feature = "http-log")]
mod http_log;
#[cfg(feature = "http-log")]
pub use http_log::HttpLog;

pub use mincat_core::{
    middleware::{FuncMiddleware, Middleware, MiddlewareFunc},
    next::Next,
};

pub use mincat_macro::middleware;
