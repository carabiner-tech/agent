use futures_util::StreamExt;
use poem::{
    web::{
        websocket::{Message, WebSocket, WebSocketStream},
        Data,
    },
    IntoResponse,
};

pub mod manager;
pub mod session;
use self::{manager::WsSessionManager, session::WsSession};

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
