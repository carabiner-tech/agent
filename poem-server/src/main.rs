pub mod api;

pub mod dependencies;
pub mod manifest;
pub mod settings;
pub mod ws;

use std::{collections::HashMap, sync::Arc};

use poem::endpoint::StaticFileEndpoint;
use poem::{
    listener::TcpListener,
    middleware::{Cors, Tracing},
    EndpointExt, Route, Server,
};
use poem_openapi::OpenApiService;
use tokio::sync::Mutex;
use ws::manager::WsSessionManager;

use crate::{api::Api, manifest::get_manifest, settings::get_settings, ws::ws_upgrade};
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

    // Build up the API http routes / OpenAPI schema
    let public_url = settings.public_url.join("/api").unwrap();
    let api_service = OpenApiService::new(Api, "Plugin Server", "1.0").server(public_url);
    let ui = api_service.swagger_ui();
    let _spec = api_service.spec_endpoint();

    let app = Route::new()
        //.at("/openapi.json", spec)
        .at("/openapi.json", StaticFileEndpoint::new("openapi.json"))
        .at("/.well-known/ai-plugin.json", serve_manifest)
        .nest("/docs", ui)
        .nest("/api", api_service)
        .at("/ws", ws_upgrade)
        .with(Cors::new())
        .with(Tracing)
        .data(ws_session_manager)
        .data(conversation_session_map);

    Server::new(TcpListener::bind(&settings.host))
        .run(app)
        .await
        .unwrap();
}

// The manifest file tells ChatGPT where the OpenAPI schema is, what auth type the plugin uses,
// and a system prompt the plugin can use to hint to the LLM how to use the plugin
// (in conjunction with OpenAPI schema). This route not need to be in OpenAPI schema itself.
#[poem::handler]
fn serve_manifest() -> String {
    let manifest = get_manifest();

    serde_json::to_string_pretty(&manifest).unwrap()
}
