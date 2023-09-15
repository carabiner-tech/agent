mod settings;
use futures_util::{SinkExt, StreamExt};
use rpc::{RpcMessage, RpcRequest};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::settings::get_settings;

#[tokio::main]
async fn main() {
    let settings = get_settings();

    let (ws_stream, _addr) = connect_async(&settings.rpc_server).await.unwrap();

    let (mut tx, mut rx) = ws_stream.split();
    while let Some(msg) = rx.next().await {
        match msg {
            Ok(Message::Text(msg)) => {
                println!("Received message: {}", msg);
                let result: Result<RpcMessage<RpcRequest>, _> = serde_json::from_str(&msg);

                if let Ok(msg_de) = result {
                    let resp = msg_de.payload.process().await;
                    let resp_msg = RpcMessage {
                        id: msg_de.id,
                        payload: resp,
                    };
                    let resp_msg_ser = serde_json::to_string(&resp_msg).unwrap();
                    tx.send(Message::Text(resp_msg_ser)).await.unwrap();
                } else {
                    // Continue to the next iteration of the loop
                    continue;
                }
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
