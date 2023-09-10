use poem::web::{Data, Query};
use poem_openapi::{param::Path, payload::PlainText, Object, OpenApi};
use rpc::{operations::current_time::CurrentTime, Op};
use serde::Deserialize;

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

    /// Request the current system time from a connected Agent by session id
    #[oai(
        path = "/current_time/:session_id",
        method = "get",
        operation_id = "current_time"
    )]
    async fn current_time(
        &self,
        session_manager: Data<&WsSessionManager>,
        session_id: Path<uuid::Uuid>,
    ) -> PlainText<String> {
        if let Some(session) = session_manager.get_session(&session_id.0).await {
            let rpc_msg = CurrentTime::make_request();
            let Op::CurrentTime(op) = session.send_rpc(rpc_msg).await.unwrap();
            PlainText(op.response.unwrap().time)
        } else {
            PlainText("session not found".to_string())
        }
    }
}
