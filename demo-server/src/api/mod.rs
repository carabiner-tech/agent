use poem::{web::Data, Error};
use poem_openapi::{param::Path, payload::PlainText, Object, OpenApi};
use rpc::operations::current_time::CurrentTime;
use serde::Deserialize;

use crate::{
    dependencies::{Conversation, ConversationHeader},
    ws::manager::WsSessionManager,
    ConversationSessionMap,
};

pub struct Api;

#[derive(Object, Deserialize, Debug)]
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

    /// List all the ids for Agents that have live websocket connections
    #[oai(path = "/agents", method = "get", operation_id = "list_agent_ids")]
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

    /// Set the active Agent to use for any RPC operations in this Conversation
    #[oai(
        path = "/use_agent/:agent_id",
        method = "post",
        operation_id = "use_agent"
    )]
    async fn set_session(
        &self,
        agent_id: Path<String>,
        conversation_header: ConversationHeader,
        conversation_session_map: Data<&ConversationSessionMap>,
        ws_session_manager: Data<&WsSessionManager>,
    ) -> Result<PlainText<String>, Error> {
        let conversation_id = conversation_header.0;
        let agent_id = uuid::Uuid::parse_str(&agent_id).map_err(|err| {
            let s = format!("Agent ID must be a valid UUID: {}", err);
            Error::from_string(s, poem::http::StatusCode::BAD_REQUEST)
        })?;

        if ws_session_manager.get_session(&agent_id).await.is_none() {
            let s = "No session found for that session id";
            return Err(Error::from_string(s, poem::http::StatusCode::BAD_REQUEST));
        }
        let mut binding = conversation_session_map.lock().await;
        binding.insert(conversation_id, agent_id);
        let s = "Session set";
        Ok(PlainText(s.to_string()))
    }

    /// RPC operation to get the current system time for the active Agent in the conversation
    #[oai(path = "/current_time", method = "get", operation_id = "current_time")]
    async fn current_time(&self, conversation: Conversation) -> PlainText<String> {
        println!("session id: {:?}", conversation.session.id);

        let rpc_msg = CurrentTime::make_request();
        let op = conversation.session.send_rpc(rpc_msg).await.unwrap();
        let ct = op.into_current_time().unwrap();
        PlainText(ct.response.unwrap().time)
    }
}
