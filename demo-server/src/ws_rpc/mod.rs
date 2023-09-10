use futures_util::{SinkExt, StreamExt};
use poem::{
    web::websocket::{Message, WebSocket, WebSocketStream},
    IntoResponse,
};

#[poem::handler]
pub async fn ws_upgrade(ws: WebSocket) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_handle(socket))
}

async fn ws_handle(socket: WebSocketStream) {
    let (mut tx, mut rx) = socket.split();
    while let Some(msg) = rx.next().await {
        match msg {
            Ok(Message::Text(msg)) => {
                println!("Received message: {}", msg);
                let echo = Message::Text(msg);
                tx.send(echo).await.unwrap();
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
