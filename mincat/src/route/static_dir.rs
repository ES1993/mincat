use derive_builder::Builder;
use futures_util::TryStreamExt;
use http::{header, HeaderValue, Method, StatusCode};
use http_body_util::StreamBody;
use hyper::body::Frame;
use mincat_core::{
    body::Body,
    error::Error,
    middleware::Middleware,
    request::{FromRequestParts, Parts},
    response::{IntoResponse, Response},
    route::Route,
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::app::MincatRoutePath;

#[derive(Clone, Builder)]
pub struct StaticDir {
    #[builder(setter(into))]
    route_path: String,
    #[builder(setter(skip))]
    file_path: String,
    #[builder(setter(into))]
    static_dir_path: String,
    #[builder(setter(into, strip_option))]
    not_found_file_path: Option<String>,
    #[builder(setter(skip))]
    middleware: Vec<Box<dyn Middleware>>,
}

impl StaticDir {
    fn get_file_path(&self) -> String {
        format!("{}/{}", self.static_dir_path, self.file_path)
    }

    fn get_route_path(&self) -> String {
        format!("{}/*file", self.route_path)
    }

    async fn open_file(&self) -> Result<(File, String), Error> {
        let file_path = self.get_file_path();
        let file = match File::open(&file_path).await {
            Ok(file) => (file, file_path),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    if let Some(not_found_file_path) = self.not_found_file_path.clone() {
                        match File::open(&not_found_file_path).await {
                            Ok(file) => (file, not_found_file_path),
                            Err(e) => return Err(Error::new(e)),
                        }
                    } else {
                        return Err(Error::new(e));
                    }
                }
                _ => return Err(Error::new(e)),
            },
        };

        Ok(file)
    }

    pub fn middleware<T>(&mut self, middleware: T) -> Self
    where
        T: Into<Box<dyn Middleware>>,
    {
        self.middleware.push(middleware.into());

        self.clone()
    }
}

impl From<StaticDir> for Route {
    fn from(static_dir: StaticDir) -> Route {
        let mut static_dir1 = static_dir.clone();
        let mut route = Route::init(
            Method::GET,
            static_dir.get_route_path(),
            move |FilePath(file_path): FilePath| async move {
                static_dir1.file_path = file_path;
                match static_dir_handle(static_dir1).await {
                    Ok(res) => res.into_response(),
                    Err(e) => {
                        let error = e.into_inner();
                        if let Some(error) = error.downcast_ref::<std::io::Error>() {
                            match error.kind() {
                                std::io::ErrorKind::NotFound => {
                                    StatusCode::NOT_FOUND.into_response()
                                }
                                _ => error.to_string().into_response(),
                            }
                        } else {
                            error.to_string().into_response()
                        }
                    }
                }
            },
        );

        for middleware in static_dir.middleware {
            route.middleware(middleware.clone());
        }

        route
    }
}

struct FilePath(String);

#[async_trait::async_trait]
impl FromRequestParts for FilePath {
    type Error = Error;

    async fn from_request_parts(parts: &mut Parts) -> Result<Self, Self::Error> {
        let MincatRoutePath(define_path) = parts
            .extensions
            .get::<MincatRoutePath>()
            .ok_or(Error::new("missing path"))?;

        let real_path = parts.uri.path();
        let mut router = matchit::Router::new();
        router.insert(define_path, true).map_err(Error::new)?;
        let matched = router.at(real_path).map_err(Error::new)?;
        let value = matched.params.get("file").unwrap_or("");

        Ok(FilePath(value.to_string()))
    }
}

async fn static_dir_handle(static_dir: StaticDir) -> Result<Response, Error> {
    let (file, path) = static_dir.open_file().await?;
    let reader_stream = ReaderStream::new(file);
    let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));

    let file_mime = mime_guess::from_path(path)
        .first()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM)
        .to_string();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&file_mime).map_err(Error::new)?,
        )],
        Body::new(stream_body),
    )
        .into_response())
}
