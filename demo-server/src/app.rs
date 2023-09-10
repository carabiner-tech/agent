use poem::Route;
use poem::{middleware::AddDataEndpoint, EndpointExt};
use poem_openapi::OpenApiService;

use crate::{
    api::Api, manifest::get_manifest, settings::get_settings, ws_rpc::ws_upgrade,
    ws_rpc::WsSessionManager,
};

pub fn build_app() -> AddDataEndpoint<Route, WsSessionManager> {
    let settings = get_settings();

    let public_url = settings.public_url.join("/api").unwrap();
    let api_service = OpenApiService::new(Api, "Plugin Server", "1.0").server(public_url);
    let ui = api_service.swagger_ui();
    let spec = api_service.spec_endpoint();
    let ws_session_manager = WsSessionManager::default();

    Route::new()
        .at("/openapi.json", spec)
        .at("/.well-known/ai-plugin.json", serve_manifest)
        .nest("/docs", ui)
        .nest("/api", api_service)
        .at("/ws", ws_upgrade)
        .data(ws_session_manager)
}

#[poem::handler]
fn serve_manifest() -> String {
    let manifest = get_manifest();
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
    manifest_json
}
