use poem_openapi::{payload::PlainText, OpenApi};

pub struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/", method = "get", operation_id = "hello")]
    async fn index(&self) -> PlainText<String> {
        let s = "Hello, world!";
        PlainText(s.to_string())
    }
}
