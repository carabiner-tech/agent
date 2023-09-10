pub mod api;
mod app;
pub mod manifest;
pub mod settings;
pub mod ws_rpc;

use poem::{
    listener::TcpListener,
    middleware::{Cors, Tracing},
    EndpointExt, Server,
};

use crate::{app::build_app, settings::get_settings};

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let settings = get_settings();

    let app = build_app().with(Cors::new()).with(Tracing::default());
    Server::new(TcpListener::bind(&settings.host))
        .run(app)
        .await
        .unwrap();
}
