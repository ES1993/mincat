use app::App;

pub(crate) mod app;
pub mod extract;
pub mod middleware;
pub mod response;
pub mod route;

pub mod http {
    pub mod header {
        pub use http::header::*;
        pub use http::HeaderValue;
    }

    pub mod mime {
        pub use mime::*;
    }

    pub use http::{Method, StatusCode};
    pub use mincat_core::{
        body::Body,
        request::{FromRequest, FromRequestParts, Parts, Request},
        response::{IntoResponse, Response},
        route::Route,
        router::Router,
    };
    pub use mincat_macro::{delete, get, head, options, patch, post, put};
}

pub fn router(router: crate::http::Router) -> App {
    App::new().router(router)
}
