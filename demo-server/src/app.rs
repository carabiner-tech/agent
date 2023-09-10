use poem::Route;
use poem_openapi::OpenApiService;

use crate::{api::Api, manifest::get_manifest, settings::get_settings, ws_rpc::ws_upgrade};

pub fn build_app() -> Route {
    let settings = get_settings();

    let public_url = settings.public_url.join("/api").unwrap();
    let api_service = OpenApiService::new(Api, "Plugin Server", "1.0").server(public_url);
    let ui = api_service.swagger_ui();
    let spec = api_service.spec_endpoint();

    Route::new()
        .at("/openapi.json", spec)
        .at("/.well-known/ai-plugin.json", serve_manifest)
        .nest("/docs", ui)
        .nest("/api", api_service)
        .at("/ws", ws_upgrade)
}

#[poem::handler]
fn serve_manifest() -> String {
    let manifest = get_manifest();
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
    manifest_json
}
