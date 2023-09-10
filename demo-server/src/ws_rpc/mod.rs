use std::{collections::HashMap, sync::Arc};

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use poem::{
    web::websocket::{Message, WebSocket, WebSocketStream},
    IntoResponse,
};
use tokio::sync::Mutex;

#[poem::handler]
pub async fn ws_upgrade(ws: WebSocket) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_handle(socket))
}

async fn ws_handle(socket: WebSocketStream) {
    let (mut tx, mut rx) = socket.split();
    let session = WsSession::new(tx);
    while let Some(msg) = rx.next().await {
        match msg {
            Ok(Message::Text(msg)) => session.handle_message(msg).await,
            Err(e) => {
                println!("Error: {}", e);
            }
            _ => {
                println!("Unknown message {:?}", msg);
            }
        }
    }
}

#[derive(Clone)]
pub struct WsSession {
    tx: Arc<Mutex<SplitSink<WebSocketStream, Message>>>,
}

impl WsSession {
    pub fn new(ws_tx: SplitSink<WebSocketStream, Message>) -> Self {
        let tx = Arc::new(Mutex::new(ws_tx));
        Self { tx }
    }

    pub async fn handle_message(&self, msg: String) {
        println!("Received message: {}", msg);
        let echo = Message::Text(msg);
        self.tx.lock().await.send(echo).await.unwrap();
    }
}
