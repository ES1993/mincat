use mincat::{
    extract::form::{Form, FormData, FormFile},
    http::{post, Router},
};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello1).route(hello2);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[derive(Form, Debug)]
struct Data {
    file: FormFile,
}

#[post("/hello1")]
async fn hello1(FormData(form): FormData<Data>) -> Result<(), String> {
    dbg!(
        form.file.name(),
        form.file.file_name(),
        form.file.content_type(),
        form.file.bytes().len()
    );
    let mut file = tokio::fs::File::create(form.file.file_name())
        .await
        .unwrap();
    file.write_all(form.file.bytes()).await.unwrap();
    Ok(())
}

#[post("/hello2")]
async fn hello2(mut form: FormData) -> Result<(), String> {
    while let Some(mut field) = form.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();

        dbg!(name, &file_name, content_type);

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_name)
            .await
            .unwrap();

        while let Some(chunk) = field.chunk().await.unwrap() {
            file.write_all(&chunk).await.unwrap();
        }
    }
    Ok(())
}
