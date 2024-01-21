use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};

use bytes::Bytes;
use http::{Extensions, Request, Response};
use http_body_util::combinators::BoxBody;
use hyper::{body::Incoming, service::service_fn};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
};
use tokio::net::TcpListener;

use crate::{
    body::{Body, BoxBodyError},
    http::StatusCode,
    router::Router,
};

#[derive(Debug, Clone)]
pub struct MincatRoutePath(pub String);

#[derive(Clone, Default)]
pub struct App {
    router: Arc<Router>,
    state: Extensions,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn router(&mut self, router: Router) -> Self {
        let mut self_router = self.router.clone();
        let self_router = Arc::make_mut(&mut self_router);
        self.router = Arc::new(self_router.merge(router));
        self.clone()
    }

    pub fn state<T>(&mut self, state: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        self.state.insert(state);
        // let mut self_state = self.state.clone();
        // let self_state = Arc::make_mut(&mut self_state);
        // self_state.insert(TypeId::of::<T>(), Box::new(state));
        // self.state = Arc::new(self_state.clone());
        self.clone()
    }

    pub async fn run(&mut self, addr: &str) {
        let addr = addr.parse::<SocketAddr>().expect("addr parse failed");
        let listener = TcpListener::bind(addr).await.expect("tcp bind failed");

        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!("accept error: {e}");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };
            let io = TokioIo::new(stream);
            let router = self.router.clone();
            let state = self.state.clone();
            let service = service_fn(move |mut request| {
                let router = router.clone();
                let state = state.clone();
                request.extensions_mut().extend(state);
                handler(router, request)
            });

            tokio::task::spawn(async move {
                if let Err(e) = Builder::new(TokioExecutor::new())
                    .serve_connection_with_upgrades(io, service)
                    .await
                {
                    tracing::error!("serve_connection_with_upgrades error: {e}");
                }
            });
        }
    }
}

async fn handler(
    router: Arc<Router>,
    request: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, BoxBodyError>>, Infallible> {
    let mut request = request.map(Body::incoming);
    let path = request.uri().path();
    let method = request.method();

    if let Some((define_path, handler)) = router.get_handler(method, path) {
        request
            .extensions_mut()
            .insert(MincatRoutePath(define_path));

        return Ok(handler.exectue(request).await.map(Body::box_body));
    }

    let res = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
        .map(Body::box_body);

    Ok(res)
}
