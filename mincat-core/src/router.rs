use std::{borrow::BorrowMut, collections::HashMap, fmt::Debug};

use http::Method;

use crate::{handler::Handler, middleware::Middleware, route::Route};

#[derive(Clone, Debug, Default)]
pub struct Endpoint {
    path: String,
    method_handler: HashMap<Method, Handler>,
}

impl Endpoint {
    fn new() -> Self {
        Self::default()
    }

    fn path(&mut self, path: &str) -> Self {
        self.path = path.to_owned();
        self.clone()
    }

    fn method_handler(&mut self, method: &Method, handler: &Handler) -> Self {
        self.method_handler.insert(method.clone(), handler.clone());
        self.clone()
    }
}

#[derive(Clone, Default)]
pub struct Router {
    index: usize,
    index_endpoint: HashMap<usize, Endpoint>,
    path_index: matchit::Router<usize>,
}

impl Debug for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Router")
            .field("index", &self.index)
            .field("index_endpoint", &self.index_endpoint)
            .finish()
    }
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn group(&mut self, path: &str, router: Router) -> Self {
        if !path.starts_with('/') {
            panic!("group routes must start with '/'");
        }

        for (_, endpoint) in router.index_endpoint {
            let path = format!("{}{}", path, endpoint.path);
            for (method, hadnler) in endpoint.method_handler {
                self.route((method, path.clone(), hadnler));
            }
        }

        self.clone()
    }

    pub fn route<T>(&mut self, route: T) -> Self
    where
        T: Into<Route>,
    {
        let Route {
            method,
            path,
            handler,
        } = route.into();

        if let Ok(matched) = self.path_index.at(&path) {
            if let Some(endpoint) = self.index_endpoint.get_mut(matched.value) {
                endpoint.method_handler.insert(method, handler);
            }
        } else {
            let index = self.index;
            self.path_index
                .insert(&path, index)
                .expect("path_index insert failed");
            let endpoint = Endpoint::new()
                .path(&path)
                .method_handler(&method, &handler);
            self.index_endpoint.insert(index, endpoint);
            self.index += 1;
        };

        self.clone()
    }

    pub fn merge(&mut self, router: Router) -> Self {
        for (_, endpoint) in router.index_endpoint {
            for (method, hadnler) in endpoint.method_handler {
                self.route((method, endpoint.path.clone(), hadnler));
            }
        }

        self.clone()
    }

    pub fn get_handler(&self, method: &Method, path: &str) -> Option<(String, Handler)> {
        if let Ok(matchit) = self.path_index.at(path) {
            if let Some(endpoint) = self.index_endpoint.get(matchit.value) {
                if let Some(handler) = endpoint.method_handler.get(method) {
                    return Some((endpoint.path.clone(), handler.clone()));
                }
            }
        }

        None
    }

    pub fn middleware<T>(&mut self, middleware: T) -> Self
    where
        T: Into<Box<dyn Middleware>> + Clone,
    {
        let middleware = middleware.into();
        for endpoint in self.index_endpoint.borrow_mut().values_mut() {
            for handler in endpoint.method_handler.borrow_mut().values_mut() {
                handler.middleware(middleware.clone());
            }
        }

        self.clone()
    }
}
