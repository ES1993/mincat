use mincat::{
    extract::FormData,
    http::{post, Router},
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Data {
    id: u64,
    name: String,
}

#[post("/hello")]
async fn hello(mut form: FormData) -> Result<(), String> {
    while let Some(mut field) = form.next_field().await.map_err(|e| e.to_string())? {
        let a= field.headers().into_iter().map(|(name,value)|{
            dbg!(name,value);
            "s"
        }).collect::<Vec<_>>();


        // let name = field.name();
        // let file_name = field.file_name();
        // let content_type = field.content_type();

        // println!(
        //     "Name: {:?}, FileName: {:?}, Content-Type: {:?}",
        //     name, file_name, content_type
        // );

        let mut field_bytes_len = 0;
        while let Some(field_chunk) = field.chunk().await.map_err(|e| e.to_string())? {
            field_bytes_len += field_chunk.len();
            let s = String::from_utf8(field_chunk.to_vec());
            dbg!(s);
        }

        println!("Field Bytes Length: {:?}", field_bytes_len);
    }

    Ok(())
}
