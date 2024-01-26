use std::future::Future;

use crate::{
    next::Next,
    request::Request,
    response::{IntoResponse, Response},
};

#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    async fn call(self: Box<Self>, request: Request, next: Next) -> Response;

    fn clone_box(&self) -> Box<dyn Middleware>;
}

impl Clone for Box<dyn Middleware> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub struct FuncMiddleware<Func>(Func)
where
    Func: MiddlewareFunc;

impl<Func> FuncMiddleware<Func>
where
    Func: MiddlewareFunc,
{
    pub fn from_fn(func: Func) -> Self {
        FuncMiddleware(func)
    }
}

#[async_trait::async_trait]
impl<Func> Middleware for FuncMiddleware<Func>
where
    Func: MiddlewareFunc,
{
    async fn call(self: Box<Self>, request: Request, next: Next) -> Response {
        self.0.call(request, next).await
    }

    fn clone_box(&self) -> Box<dyn Middleware> {
        Box::new(FuncMiddleware(self.0.clone()))
    }
}

#[async_trait::async_trait]
pub trait MiddlewareFunc: Clone + Send + Sync + 'static {
    async fn call(self, request: Request, next: Next) -> Response;
}

#[async_trait::async_trait]
impl<Func, Res, Fut> MiddlewareFunc for Func
where
    Func: FnOnce(Request, Next) -> Fut,
    Func: Clone + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    Res: IntoResponse,
{
    async fn call(self, request: Request, next: Next) -> Response {
        self(request, next).await.into_response()
    }
}
