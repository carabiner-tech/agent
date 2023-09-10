mod settings;
use futures_util::{SinkExt, StreamExt};
use rpc::{Operation, RpcMessage};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::settings::get_settings;

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
                let req: RpcMessage = serde_json::from_str(&msg).unwrap();
                let op = req.op.process();
                let resp = RpcMessage { id: req.id, op };
                let resp_se = serde_json::to_string(&resp).unwrap();
                let resp_msg = Message::Text(resp_se);
                tx.send(resp_msg).await.unwrap();
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
