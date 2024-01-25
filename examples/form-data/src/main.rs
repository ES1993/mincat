use mincat::{
    extract::form::{Form, FormData, FormFile},
    http::{post, Router},
};

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello1).route(hello2);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[post("/hello1")]
async fn hello1(mut form: FormData) -> Result<(), String> {
    while let Some(mut field) = form.next_field().await.map_err(|e| e.to_string())? {
        dbg!("=================split================");
        dbg!(field.name());
        dbg!(field.file_name());
        dbg!(field.content_type());

        let mut field_bytes_len = 0;
        while let Some(field_chunk) = field.chunk().await.map_err(|e| e.to_string())? {
            field_bytes_len += field_chunk.len();
        }

        dbg!(field_bytes_len);
    }

    Ok(())
}

#[derive(Form, Debug)]
struct Data {
    string: String,
    integer: i128,
    boolean: bool,
    number: usize,
    files: Vec<FormFile>,
}

#[post("/hello2")]
async fn hello2(FormData(form): FormData<Data>) -> Result<(), String> {
    dbg!(form.string, form.integer, form.boolean, form.number);
    for file in form.files {
        dbg!(
            file.name(),
            file.file_name(),
            file.content_type(),
            file.bytes().len()
        );
    }
    Ok(())
}
