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
    Object, OpenApi,
};
use rpc::operations::{
    list_files::ListFilesRequest,
    time::{SystemTimeRequest, SystemTimeResponse},
};
use rpc::{RpcMessage, RpcRequest, RpcResponse};
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
    async fn use_agent(
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
        let s = "Agent set for this conversation";
        Ok(PlainText(s.to_string()))
    }

    /// RPC operation to get the current system time for the active Agent in the conversation
    #[oai(path = "/current_time", method = "post", operation_id = "current_time")]
    async fn current_time(
        &self,
        body: Json<SystemTimeRequest>,
        conversation: Conversation,
    ) -> Result<PlainText<String>, Error> {
        let req = body.0;
        let resp = conversation
            .session
            .send_rpc(RpcRequest::SystemTime(req))
            .await;
        match resp.into_system_time() {
            Ok(SystemTimeResponse { time }) => {
                let s = format!("Current time: {}", time);
                Ok(PlainText(s))
            }
            Err(e) => {
                let rpc_error = e.into_rpc_error().unwrap();

                Err(Error::from_string(
                    rpc_error.to_string(),
                    poem::http::StatusCode::BAD_REQUEST,
                ))
            }
        }
    }

    /// RPC operation to list files in the current working directory or subdirectory for the active Agent
    #[oai(path = "/list_files", method = "post", operation_id = "list_files")]
    async fn list_files(
        &self,
        body: Json<ListFilesRequest>,
        conversation: Conversation,
    ) -> Result<PlainText<String>, Error> {
        let req = body.0;
        let resp = conversation
            .session
            .send_rpc(RpcRequest::ListFiles(req))
            .await;
        match resp.into_list_files() {
            Ok(rpc::operations::list_files::ListFilesResponse { files, directories }) => {
                let mut s = String::new();
                for file in files {
                    s.push_str(&format!("{}: {} bytes\n", file.name, file.size));
                }
                for dir in directories {
                    s.push_str(&format!("{}: directory\n", dir));
                }
                Ok(PlainText(s))
            }
            Err(e) => {
                let rpc_error = e.into_rpc_error().unwrap();

                Err(Error::from_string(
                    rpc_error.to_string(),
                    poem::http::StatusCode::BAD_REQUEST,
                ))
            }
        }
    }
}
