use std::{fmt::Debug, future::Future, marker::PhantomData};

use mincat_macro::repeat_macro_max_generics_param;

use crate::{
    body::Body,
    middleware::Middleware,
    request::{FromRequest, Request},
    response::{IntoResponse, Response},
};

#[derive(Clone)]
pub struct Handler {
    pub func: Box<dyn HandlerFunc>,
    pub middleware: Vec<Box<dyn Middleware>>,
}

impl Debug for Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handler")
            .field("func", &"func")
            .field("middleware", &self.middleware.len())
            .finish()
    }
}

impl Handler {
    pub fn middleware<T>(&mut self, middleware: T) -> Self
    where
        T: Middleware,
    {
        self.middleware.push(Box::new(middleware));

        self.clone()
    }

    pub async fn exectue(self, mut request: Request) -> Response {
        for middleware in &self.middleware {
            let mut response = Response::new(Body::empty());
            middleware.on_request(&mut request, &mut response).await;
        }

        let response = self.func.call(&mut request).await;

        for middleware in &self.middleware {
            let mut response = Response::new(Body::empty());
            middleware.on_request(&mut request, &mut response).await;
        }

        response
    }
}

#[async_trait::async_trait]
pub trait HandlerFunc: Send + Sync + 'static {
    async fn call(self: Box<Self>, request: &mut Request) -> Response;

    fn clone_box(&self) -> Box<dyn HandlerFunc>;
}

impl Clone for Box<dyn HandlerFunc> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[async_trait::async_trait]
pub trait HandlerFuncParam<Param>: Clone + Send + Sync + 'static {
    async fn call(self, request: &mut Request) -> Response;
}

#[derive(Clone)]
pub struct FuncParamHandler<Func, Param>
where
    Func: HandlerFuncParam<Param>,
    Param: Clone + Send + Sync + 'static,
{
    func: Func,
    _mark: PhantomData<Param>,
}

impl<Func, Param> From<Func> for FuncParamHandler<Func, Param>
where
    Func: HandlerFuncParam<Param>,
    Param: Clone + Send + Sync + 'static,
{
    fn from(value: Func) -> Self {
        Self {
            func: value,
            _mark: PhantomData,
        }
    }
}

impl<Func, Param> From<FuncParamHandler<Func, Param>> for Handler
where
    Func: HandlerFuncParam<Param>,
    Param: Clone + Send + Sync + 'static,
{
    fn from(value: FuncParamHandler<Func, Param>) -> Self {
        Handler {
            func: Box::new(value),
            middleware: vec![],
        }
    }
}

#[async_trait::async_trait]
impl<Func, Param> HandlerFunc for FuncParamHandler<Func, Param>
where
    Func: HandlerFuncParam<Param>,
    Param: Clone + Send + Sync + 'static,
{
    async fn call(self: Box<Self>, request: &mut Request) -> Response {
        self.func.call(request).await
    }

    fn clone_box(&self) -> Box<dyn HandlerFunc> {
        Box::new(self.clone())
    }
}

macro_rules! handle_func_param {
    ($($param: ident),*) => {
        #[allow(non_snake_case)]
        #[async_trait::async_trait]
        impl<Func, Fut, Res, $($param),*> HandlerFuncParam<($($param,)*)> for Func
        where
            Func: FnOnce($($param),*) -> Fut,
            Func: Clone + Send + Sync + 'static,
            Fut: Future<Output = Res> + Send,
            Res: IntoResponse,
            $($param: FromRequest),*
        {
            #[allow(unused_variables)]
            async fn call(self, request: &mut Request) -> Response {
                let exec = || async move {
                    let res = self($({
                        let param = match $param::from_request(request).await {
                            Ok(v) => v,
                            Err(e) => return Err(e.into_response()),
                        };

                        param
                    }),*).await;

                    Ok::<Response,Response>(res.into_response())
                };

                match exec().await {
                    Ok(v) => v,
                    Err(e) => e,
                }
            }

        }
    }
}

repeat_macro_max_generics_param!(handle_func_param, 17, P);
