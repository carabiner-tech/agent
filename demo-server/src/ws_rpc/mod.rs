use std::{collections::HashMap, sync::Arc};

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use poem::{
    web::{
        websocket::{Message, WebSocket, WebSocketStream},
        Data,
    },
    IntoResponse,
};
use tokio::sync::Mutex;

#[poem::handler]
pub async fn ws_upgrade(
    ws: WebSocket,
    session_manager: Data<&WsSessionManager>,
) -> impl IntoResponse {
    let session_manager = session_manager.clone();
    ws.on_upgrade(move |socket| ws_handle(socket, session_manager))
}

async fn ws_handle(socket: WebSocketStream, session_manager: WsSessionManager) {
    let (mut tx, mut rx) = socket.split();
    let session = WsSession::new(tx);
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
    pub tx: Arc<Mutex<SplitSink<WebSocketStream, Message>>>,
    pub id: uuid::Uuid,
}

impl WsSession {
    pub fn new(ws_tx: SplitSink<WebSocketStream, Message>) -> Self {
        let tx = Arc::new(Mutex::new(ws_tx));
        let id = uuid::Uuid::new_v4();
        Self { tx, id }
    }

    pub async fn handle_message(&self, msg: String) {
        println!("Received message: {}", msg);
        let echo = Message::Text(msg);
        self.tx.lock().await.send(echo).await.unwrap();
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
