use http::Method;

use crate::handler::{FuncParamHandler, Handler, HandlerFuncParam};

pub struct Route {
    pub method: Method,
    pub path: String,
    pub handler: Handler,
}

impl Route {
    pub fn middleware(&mut self) -> Self {
        todo!()
    }

    pub fn init<Path, Func, Param>(method: Method, path: Path, func: Func) -> Self
    where
        Path: Into<String>,
        Func: HandlerFuncParam<Param>,
        Param: Clone + Send + Sync + 'static,
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
