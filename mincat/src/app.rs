use http::{Extensions, Request, StatusCode};
use hyper::{body::Incoming, service::service_fn};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
};
use mincat_core::{
    body::Body,
    response::{IntoResponse, Response},
    router::Router,
};
use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;

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

async fn handler(router: Arc<Router>, request: Request<Incoming>) -> Result<Response, Infallible> {
    let mut request = request.map(Body::new);
    let path = request.uri().path();
    let method = request.method();

    if let Some((define_path, handler)) = router.get_handler(method, path) {
        request
            .extensions_mut()
            .insert(MincatRoutePath(define_path));

        return Ok(handler.exectue(request).await);
    }

    Ok(StatusCode::NOT_FOUND.into_response())
}
