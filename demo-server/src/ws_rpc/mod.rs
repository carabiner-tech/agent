use std::{collections::HashMap, sync::Arc};

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use poem::{
    web::{
        websocket::{Message, WebSocket, WebSocketStream},
        Data,
    },
    IntoResponse,
};
use rpc::RpcMessage;
use tokio::sync::{oneshot, Mutex};

#[poem::handler]
pub async fn ws_upgrade(
    ws: WebSocket,
    session_manager: Data<&WsSessionManager>,
) -> impl IntoResponse {
    let session_manager = session_manager.clone();
    ws.on_upgrade(move |socket| ws_handle(socket, session_manager))
}

async fn ws_handle(socket: WebSocketStream, session_manager: WsSessionManager) {
    let (tx, mut rx) = socket.split();
    let session = WsSession::new(tx);
    println!("New session: {:?}", session.id);
    session_manager.add_session(session.clone()).await;
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
    session_manager.remove_session(session).await;
}

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

#[derive(Clone, Default)]
pub struct WsSessionManager {
    sessions: Arc<Mutex<HashMap<uuid::Uuid, WsSession>>>,
}

impl WsSessionManager {
    pub async fn get_session(&self, id: &uuid::Uuid) -> Option<WsSession> {
        let sessions = self.sessions.lock().await;
        sessions.get(id).cloned()
    }

    pub async fn add_session(&self, session: WsSession) {
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session.id.clone(), session);
    }

    pub async fn remove_session(&self, session: WsSession) {
        let mut sessions = self.sessions.lock().await;
        sessions.remove(&session.id);
    }

    pub async fn list_sessions(&self) -> Vec<WsSession> {
        let sessions = self.sessions.lock().await;
        sessions.values().cloned().collect()
    }
}
