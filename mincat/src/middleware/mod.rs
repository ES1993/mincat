mod body_limit;

#[cfg(feature = "session")]
pub mod session;

#[cfg(feature = "cors")]
pub mod cors;

#[cfg(feature = "body-limit")]
pub use body_limit::BodyLimit;

pub use mincat_core::{
    middleware::{FuncMiddleware, Middleware, MiddlewareFunc},
    next::Next,
};

pub use mincat_macro::middleware;
