use mincat::{
    extract::form::{Form, FormData, FormFile},
    http::{post, Router},
    middleware::BodyLimit,
};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route(hello)
        .middleware(BodyLimit(1024 * 1024 * 20));

    let router = Router::new()
        .group("/api", router)
        .route(hello)
        .middleware(BodyLimit(1024 * 1024 * 2));

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[derive(Form, Debug)]
struct Data {
    file: FormFile,
}

#[post("/hello")]
async fn hello(FormData(form): FormData<Data>) -> Result<(), String> {
    dbg!(
        form.file.name(),
        form.file.file_name(),
        form.file.content_type(),
        form.file.bytes().len()
    );
    Ok(())
}
