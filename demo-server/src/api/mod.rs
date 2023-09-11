pub mod conversation;
use poem::{web::Data, Body};
use poem_openapi::{param::Path, payload::PlainText, OpenApi};
use poem_openapi::{payload::Json, Object};
use rpc::{operations::current_time::CurrentTime, Op};
use serde::Deserialize;

use crate::{ws_rpc::WsSessionManager, ConversationSessionMap};

use self::conversation::{Conversation, ConversationHeader};

pub struct Api;

#[derive(Object, Deserialize)]
struct SetSessionBody {
    session_id: uuid::Uuid,
}

#[OpenApi]
impl Api {
    #[oai(path = "/", method = "get", operation_id = "hello")]
    async fn index(&self) -> PlainText<String> {
        let s = "Hello, world!";
        PlainText(s.to_string())
    }

    #[oai(path = "/sessions", method = "get", operation_id = "list_sessions")]
    async fn list_sessions(&self, session_manager: Data<&WsSessionManager>) -> PlainText<String> {
        let sessions = session_manager.list_sessions().await;
        let mut session_ids = Vec::new();
        for session in sessions {
            let id = session.id.to_string();
            session_ids.push(id.to_string());
        }
        let s = session_ids.join("\n ");
        PlainText(s)
    }

    /// Set an Agent websocket session id for the Conversation
    #[oai(path = "/set_session", method = "post", operation_id = "set_session")]
    async fn set_session(
        &self,
        body: Json<SetSessionBody>,
        conversation_header: ConversationHeader,
        conversation_session_map: Data<&ConversationSessionMap>,
        ws_session_manager: Data<&WsSessionManager>,
    ) -> PlainText<String> {
        let conversation_id = conversation_header.0;
        let session_id = body.session_id;
        if ws_session_manager.get_session(&session_id).await.is_none() {
            let s = "No session found for that session id";
            return PlainText(s.to_string());
        }
        let mut binding = conversation_session_map.lock().await;
        binding.insert(conversation_id, session_id);
        let s = "Session set";
        PlainText(s.to_string())
    }

    /// Request the current system time from a connected Agent by session id
    #[oai(path = "/current_time", method = "get", operation_id = "current_time")]
    async fn current_time(&self, conversation: Conversation) -> PlainText<String> {
        println!("session id: {:?}", conversation.session.id);

        let rpc_msg = CurrentTime::make_request();
        let op = conversation.session.send_rpc(rpc_msg).await.unwrap();
        let ct = op.into_current_time().unwrap();
        PlainText(ct.response.unwrap().time)
    }
}
