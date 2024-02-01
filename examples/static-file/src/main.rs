use mincat::{http::Router, route::StaticDirBuilder};

#[tokio::main]
async fn main() {
    let static_dir_route = StaticDirBuilder::default()
        .route_path("/hello")
        .static_dir_path("assets")
        .not_found_file_path("assets/404.json")
        .build()
        .unwrap();
    let router = Router::new().route(static_dir_route);

    mincat::router(router).run("127.0.0.1:3000").await;
}
