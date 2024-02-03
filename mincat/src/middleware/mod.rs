mod body_limit;

#[cfg(any(
    feature = "session",
    feature = "session-memory",
    feature = "session-redis",
    feature = "session-postgres",
    feature = "session-mysql"
))]
pub mod session;

pub mod cors;

pub use body_limit::BodyLimit;

pub use mincat_core::{
    middleware::{FuncMiddleware, Middleware, MiddlewareFunc},
    next::Next,
};

pub use mincat_macro::middleware;
