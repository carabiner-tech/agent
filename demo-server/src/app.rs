use poem::Route;
use poem_openapi::OpenApiService;

use crate::{api::Api, manifest::get_manifest, settings::get_settings, ws::ws_upgrade};

pub fn build_app() -> Route {
    let settings = get_settings();

    let public_url = settings.public_url.join("/api").unwrap();
    let api_service = OpenApiService::new(Api, "Plugin Server", "1.0").server(public_url);
    let ui = api_service.swagger_ui();
    let spec = api_service.spec_endpoint();

    // For the sake of sane type inference, middleware and "data" are added in main.rs
    Route::new()
        .at("/openapi.json", spec)
        .at("/.well-known/ai-plugin.json", serve_manifest)
        .nest("/docs", ui)
        .nest("/api", api_service)
        .at("/ws", ws_upgrade)
}

// Doesn't need to be in OpenAPI schema. The manifest file tells ChatGPT where the OpenAPI
// schema is at, what auth type the plugin uses, and a system prompt the plugin can use
// to hint to the LLM how to use the plugin (in conjunction with OpenAPI schema)
#[poem::handler]
fn serve_manifest() -> String {
    let manifest = get_manifest();
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
    manifest_json
}
