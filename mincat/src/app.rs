use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use bytes::Bytes;
use http::{Request, Response};
use http_body_util::combinators::BoxBody;
use hyper::{body::Incoming, service::service_fn};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
};
use tokio::net::TcpListener;

use crate::{body::Body, http::StatusCode, router::Router};

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
        let addr = addr.parse::<SocketAddr>().expect("地址错误");
        let listener = TcpListener::bind(addr).await.expect("tcp bind failed");

        loop {
            let (stream, _) = listener.accept().await.expect("tcp accept failed");
            let io = TokioIo::new(stream);
            let router = self.router.clone();
            let service = service_fn(move |request| {
                let router = router.clone();
                handler(router, request)
            });

            tokio::task::spawn(async move {
                match Builder::new(TokioExecutor::new())
                    .serve_connection_with_upgrades(io, service)
                    .await
                {
                    Ok(_) => (),
                    Err(_) => (),
                }
            });
        }
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;

async fn handler(
    router: Arc<Router>,
    request: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, Error>>, Infallible> {
    let request = request.map(Body::incoming);
    let path = request.uri().path();
    let method = request.method();

    if let Some((_define_path, handler)) = router.get_handler(method, path) {
        return Ok(handler.exectue(request).await.map(Body::box_body));
    }

    let res = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
        .map(Body::box_body);

    Ok(res)
}
