//! Represents a Websocket session for an Agent.
//! Reminder that the server is the one sending RPC requests to the Agent and receiving replies.
use std::{collections::HashMap, sync::Arc};

use futures_util::{stream::SplitSink, SinkExt};
use poem::web::websocket::{Message, WebSocketStream};
use rpc::RpcMessage;
use tokio::sync::{oneshot, Mutex};
#[derive(Clone)]
pub struct WsSession {
    pub id: uuid::Uuid,
    pub tx: Arc<Mutex<SplitSink<WebSocketStream, Message>>>,
    callbacks: Arc<Mutex<HashMap<uuid::Uuid, oneshot::Sender<rpc::Op>>>>,
}

impl WsSession {
    pub fn new(ws_tx: SplitSink<WebSocketStream, Message>) -> Self {
        let id = uuid::Uuid::new_v4();
        let tx = Arc::new(Mutex::new(ws_tx));
        let callbacks = Arc::new(Mutex::new(HashMap::new()));
        Self { id, tx, callbacks }
    }

    pub async fn handle_message(&self, msg: String) {
        println!("Received message: {}", msg);
        let parsed_msg = serde_json::from_str::<RpcMessage>(&msg);
        match parsed_msg {
            Ok(msg) => {
                // check if msg.id is in callbacks
                let mut callbacks = self.callbacks.lock().await;
                if let Some(tx) = callbacks.remove(&msg.id) {
                    println!("Found callback for message: {}", msg.id);
                    let _ = tx.send(msg.op);
                } else {
                    println!("No callback for message: {}", msg.id);
                }
            }
            Err(e) => {
                println!("Error parsing message: {}", e);
            }
        }
    }

    pub async fn send_rpc(&self, msg: RpcMessage) -> Option<rpc::Op> {
        let msg_se = serde_json::to_string(&msg).unwrap();
        let (cb_tx, cb_rx) = oneshot::channel::<rpc::Op>();
        let mut callbacks = self.callbacks.lock().await;
        callbacks.insert(msg.id, cb_tx);
        drop(callbacks);
        let mut tx = self.tx.lock().await;
        let _ = tx.send(Message::Text(msg_se)).await;
        drop(tx);
        cb_rx.await.ok()
    }
}
