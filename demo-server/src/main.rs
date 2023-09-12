pub mod api;
mod app;
pub mod dependencies;
pub mod manifest;
pub mod settings;
pub mod ws;

use std::{collections::HashMap, sync::Arc};

use poem::{
    listener::TcpListener,
    middleware::{Cors, Tracing},
    EndpointExt, Server,
};
use tokio::sync::Mutex;
use ws::manager::WsSessionManager;

use crate::{app::build_app, settings::get_settings};

type ConversationSessionMap = Arc<Mutex<HashMap<String, uuid::Uuid>>>;

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let settings = get_settings();
    // when Agents connect over websocket, they're assigned an id. Endpoints sending RPC messages to
    // Agents use the ws_session_manager to get the websocket connection by session id
    let ws_session_manager = WsSessionManager::default();

    // When Users are working with LLM, they "set" an Agent ID for the LLM conversation. Endpoints
    // that make RPC request-replies use this conversation_session_map to get the right websocket
    // session for that Agent. If the Agent disconnects, RPC endpoints will return 400's.
    let conversation_session_map: ConversationSessionMap = Arc::new(Mutex::new(HashMap::new()));

    let app = build_app()
        .with(Cors::new())
        .with(Tracing::default())
        .data(ws_session_manager)
        .data(conversation_session_map);

    Server::new(TcpListener::bind(&settings.host))
        .run(app)
        .await
        .unwrap();
}
