//! Dependencies used in API route handlers
use poem::{http::StatusCode, Error, FromRequest, Request, RequestBody, Result};

use crate::{
    ws::{manager::WsSessionManager, session::WsSession},
    ConversationSessionMap,
};

// Use this when conversation-id header is required but not a connected websocket session,
// such as for a route to set the active Agent for a conversation. Raises 400 if header missing.
pub struct ConversationHeader(pub String);

#[poem::async_trait]
impl<'a> FromRequest<'a> for ConversationHeader {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self, Error> {
        // Check if conversation-id header is present, raise 400 if not
        let conv_id = match req.headers().get("openai-conversation-id") {
            Some(header) => Some(header).unwrap().to_str().unwrap(),
            // In production, you probably want to err out if there's no conversation header
            // For dev/debug, we'll just fall back to a hardcoded conv id 123
            // None => {
            //     println!("Missing header, req headers: {:#?}", req.headers());
            //     let msg = "Missing conversation-id header";
            //     return Err(Error::from_string(msg, StatusCode::BAD_REQUEST));
            // }
            None => {
                println!("Falling back to hard-coded id");
                "123"
            }
        };

        Ok(Self(conv_id.to_string()))
    }
}

// Use this when you need an active websocket connection for the Agent in the conversation.
// Raises 400 if conversation id header is missing, no Agent has been set for the conversation,
// or if the websocket connection for that Agent has ended.
pub struct Conversation {
    pub id: String,
    pub session: WsSession,
}

#[poem::async_trait]
impl<'a> FromRequest<'a> for Conversation {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self, Error> {
        let conv_id = ConversationHeader::from_request_without_body(req).await?;

        // Check if an Agent has been set for this conversation, raise 400 if not
        let conversation_session_map = req.data::<ConversationSessionMap>().unwrap();
        let ws_session_manager = req.data::<WsSessionManager>().unwrap();

        let binding = conversation_session_map.lock().await;
        let session = match binding.get(&conv_id.0) {
            Some(session_id) => {
                let session = ws_session_manager.get_session(session_id).await;
                match session {
                    Some(session) => session,
                    None => {
                        let msg = "No session found for conversation";
                        return Err(Error::from_string(msg, StatusCode::BAD_REQUEST));
                    }
                }
            }
            None => match ws_session_manager.first_session().await {
                Some(session) => session,
                None => {
                    let msg = "No session found for conversation";
                    return Err(Error::from_string(msg, StatusCode::BAD_REQUEST));
                }
            },
        };
        Ok(Self {
            id: conv_id.0,
            session: session,
        })
    }
}
