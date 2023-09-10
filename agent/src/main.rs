mod settings;
use crate::settings::get_settings;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    let settings = get_settings();

    let (ws_stream, _addr) = connect_async(&settings.rpc_server).await.unwrap();

    let (mut tx, mut rx) = ws_stream.split();
    let msg = Message::Text("Test".to_string());
    tx.send(msg).await.unwrap();
    while let Some(msg) = rx.next().await {
        match msg {
            Ok(Message::Text(msg)) => {
                println!("Received message: {}", msg);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
            _ => {
                println!("Unknown message {:?}", msg);
            }
        }
    }
}
