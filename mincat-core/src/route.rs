use http::Method;

use crate::{
    handler::{FuncParamHandler, Handler, HandlerFuncParam},
    middleware::Middleware,
};

#[derive(Clone)]
pub struct Route {
    pub method: Method,
    pub path: String,
    pub handler: Handler,
}

impl Route {
    pub fn middleware<T>(&mut self, middleware: T) -> Self
    where
        T: Into<Box<dyn Middleware>>,
    {
        self.handler.middleware(middleware);
        self.clone()
    }

    pub fn init<Path, Func, Param>(method: Method, path: Path, func: Func) -> Self
    where
        Path: Into<String>,
        Func: HandlerFuncParam<Param> + Sync + Clone + 'static,
        Param: Send + Sync + 'static,
    {
        let path = path.into();
        let handler = FuncParamHandler::from(func).into();
        Self {
            method,
            path,
            handler,
        }
    }
}

impl From<(Method, String, Handler)> for Route {
    fn from(value: (Method, String, Handler)) -> Self {
        Self {
            method: value.0,
            path: value.1,
            handler: value.2,
        }
    }
}
