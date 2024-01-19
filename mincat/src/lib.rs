use crate::router::Router;
use app::App;

pub(crate) mod app;

pub use mincat_core::*;

pub mod http {
    pub use http::{Method, StatusCode};
    pub use mincat_macro::{delete, get, head, options, patch, post, put};
}

pub fn router(router: Router) -> App {
    App::new().router(router)
}
