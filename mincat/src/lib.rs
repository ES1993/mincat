use app::App;

pub(crate) mod app;
pub mod extract;
pub mod response;

pub mod http {
    pub use http::{Method, StatusCode};
    pub use mincat_core::{
        body::Body,
        request::{FromRequest, Parts, Request},
        response::{IntoResponse, Response},
        route::Route,
        router::Router,
    };
    pub use mincat_macro::{delete, get, head, options, patch, post, put};
}

pub mod middleware {
    pub use mincat_core::{
        middleware::{FuncMiddleware, Middleware, MiddlewareFunc},
        next::Next,
    };
    pub use mincat_macro::middleware;
}

pub fn router(router: crate::http::Router) -> App {
    App::new().router(router)
}
