//! Endpoints that will show up in the OpenAPI Schema.
//!
//! Endpoints can use the Conversation "dependency injection" style function to ensure
//! an Agent id has been "set" for the conversation (based on HTTP header) and that
//! the Agent has an active websocket connection. It also contains a ref to the websocket
//! session to send / receive RPC messages over.
//!
//! RPC Request structs from the rpc lib implement the poem-openapi Object trait,
//! so they can be used as the body for a POST request and automatically documented
//! in the OpenAPI schema.

use poem::{web::Data, Error};
use poem_openapi::{
    param::Path,
    payload::{Json, PlainText},
    OpenApi,
};
use rpc::operations::{list_files::ListFilesRequest, time::SystemTimeRequest};

use crate::{
    dependencies::{Conversation, ConversationHeader},
    ws::manager::WsSessionManager,
    ConversationSessionMap,
};

// Websocket session state and conversation -> Agent ws connection are both stored
// in App state rather than as part of the Api struct (which routes could access with
// &self), because the ws endpoint needs to get access to them too.
pub struct Api;

fn rpc_error(e: rpc::RpcResponse) -> Error {
    let rpc_error = e.into_rpc_error().unwrap();
    Error::from_string(rpc_error.to_string(), poem::http::StatusCode::BAD_REQUEST)
}

#[OpenApi]
impl Api {
    /// Set the active Agent to use for any RPC operations in this Conversation
    #[oai(
        path = "/use_agent/:agent_id",
        method = "post",
        operation_id = "use_agent"
    )]
    async fn use_agent(
        &self,
        agent_id: Path<String>,
        conversation_header: ConversationHeader,
        conversation_session_map: Data<&ConversationSessionMap>,
        ws_session_manager: Data<&WsSessionManager>,
    ) -> poem::Result<PlainText<String>> {
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
        let s = "Agent set for this conversation";
        Ok(PlainText(s.to_string()))
    }

    /// RPC operation to get the current system time for the active Agent in the conversation
    #[oai(path = "/current_time", method = "post", operation_id = "current_time")]
    async fn current_time(&self, conversation: Conversation) -> poem::Result<PlainText<String>> {
        let req = SystemTimeRequest {};
        let resp = conversation.session.send_rpc(req.into()).await;
        match resp.into_system_time() {
            Ok(resp) => {
                let s = format!("Current time: {}", resp.time);
                Ok(PlainText(s))
            }
            Err(e) => Err(rpc_error(e)),
        }
    }

    /// RPC operation to list files in the current working directory or subdirectory for the active Agent
    #[oai(path = "/list_files", method = "post", operation_id = "list_files")]
    async fn list_files(
        &self,
        body: Json<ListFilesRequest>,
        conversation: Conversation,
    ) -> poem::Result<PlainText<String>> {
        let req = body.0;
        let resp = conversation.session.send_rpc(req.into()).await;
        match resp.into_list_files() {
            Ok(resp) => {
                let mut s = String::new();
                for file in resp.files {
                    s.push_str(&format!("{}: {} bytes\n", file.name, file.size));
                }
                for dir in resp.directories {
                    s.push_str(&format!("{}: directory\n", dir));
                }
                Ok(PlainText(s))
            }
            Err(e) => Err(rpc_error(e)),
        }
    }
}
