use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, WebSocketStream};
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream};
mod settings;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use rpc::{RpcMessage, RpcRequest, RpcResponse};
use settings::get_settings;

type WebsocketTx = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

#[derive(Serialize, Deserialize, Debug)]
struct PartialRpcMessage {
    id: uuid::Uuid,
    payload: Value,
}

async fn handle_successful_payload(id: uuid::Uuid, payload: RpcRequest, tx: &mut WebsocketTx) {
    let resp = payload.process().await;
    let resp_msg = RpcMessage { id, payload: resp };
    let resp_msg_ser = serde_json::to_string(&resp_msg).unwrap();
    tx.send(Message::Text(resp_msg_ser)).await.unwrap();
}

async fn handle_failed_payload(id: uuid::Uuid, error: serde_json::Error, tx: &mut WebsocketTx) {
    let error_msg = format!("Deserialization error: {:?}", error);
    let resp_msg = RpcMessage {
        id,
        payload: RpcResponse::RpcError { e: error_msg },
    };
    let resp_msg_ser = serde_json::to_string(&resp_msg).unwrap();
    tx.send(Message::Text(resp_msg_ser)).await.unwrap();
}

#[tokio::main]
async fn main() {
    let settings = get_settings();
    let (ws_stream, _addr) = connect_async(&settings.rpc_server).await.unwrap();
    let (mut tx, mut rx) = ws_stream.split();

    while let Some(msg) = rx.next().await {
        match msg {
            Ok(Message::Text(msg)) => {
                let partial_result: Result<PartialRpcMessage, _> = serde_json::from_str(&msg);
                if let Ok(partial_msg) = partial_result {
                    let payload_result: Result<RpcRequest, _> =
                        serde_json::from_value(partial_msg.payload.clone());
                    match payload_result {
                        Ok(payload) => {
                            println!("Got RPC message: {:?}", payload);
                            handle_successful_payload(partial_msg.id, payload, &mut tx).await
                        }
                        Err(error) => handle_failed_payload(partial_msg.id, error, &mut tx).await,
                    }
                } else {
                    println!("Got non-RPC message: {}", msg);
                }
            }
            Err(e) => println!("Error: {}", e),
            _ => println!("Unknown message {:?}", msg),
        }
    }
}
