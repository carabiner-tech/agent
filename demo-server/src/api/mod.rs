use poem::web::Data;
use poem_openapi::{payload::PlainText, OpenApi};

use crate::ws_rpc::WsSessionManager;

pub struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/", method = "get", operation_id = "hello")]
    async fn index(&self) -> PlainText<String> {
        let s = "Hello, world!";
        PlainText(s.to_string())
    }

    #[oai(path = "/sessions", method = "get", operation_id = "list_sessions")]
    async fn list_sessions(&self, session_manager: Data<&WsSessionManager>) -> PlainText<String> {
        // return list of abbreviated session ids (last part of uuid)
        let sessions = session_manager.list_sessions().await;
        let mut session_ids = Vec::new();
        for session in sessions {
            let id = session.id.to_string();
            let id = id.split('-').last().unwrap();
            session_ids.push(id.to_string());
        }
        let s = session_ids.join(", ");
        PlainText(s)
    }
}
