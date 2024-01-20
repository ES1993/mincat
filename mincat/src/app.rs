use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};

use bytes::Bytes;
use http::{Request, Response};
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

#[derive(Clone)]
pub struct App {
    router: Arc<Router>,
}

impl App {
    pub fn new() -> Self {
        App {
            router: Arc::new(Router::new()),
        }
    }

    pub fn router(&mut self, router: Router) -> Self {
        self.router = Arc::new(router);
        self.clone()
    }

    pub fn state(&mut self) -> Self {
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
            let service = service_fn(move |request| {
                let router = router.clone();
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
