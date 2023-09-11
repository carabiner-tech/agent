use crate::{
    ws_rpc::{WsSession, WsSessionManager},
    ConversationSessionMap,
};
use poem::http::StatusCode;
use poem::Error;
use poem::Result;
use poem::{FromRequest, Request, RequestBody};

pub struct ConversationHeader(pub String);

#[poem::async_trait]
impl<'a> FromRequest<'a> for ConversationHeader {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self, Error> {
        let conv_header = req.headers().get("conversation-id");
        // 404 right away if there's no conversation-id header
        if conv_header.is_none() {
            let msg = "Missing conversation-id header";
            return Err(Error::from_string(msg, StatusCode::BAD_REQUEST));
        }
        let conv_id = conv_header.unwrap().to_str().unwrap();
        Ok(Self(conv_id.to_string()))
    }
}

pub struct Conversation {
    pub id: String,
    pub session: WsSession,
}

#[poem::async_trait]
impl<'a> FromRequest<'a> for Conversation {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self, Error> {
        let conv_id = ConversationHeader::from_request_without_body(req).await?;

        let conversation_session_map = req.data::<ConversationSessionMap>().unwrap();
        let binding = conversation_session_map.lock().await;
        let maybe_session_id = binding.get(&conv_id.0);
        if maybe_session_id.is_none() {
            let msg = "No session set for this Conversation yet";
            return Err(Error::from_string(msg, StatusCode::NOT_FOUND));
        }
        let session_id = maybe_session_id.unwrap();

        // Check if we still have a connected websocket session for the Agent
        let ws_session_manager = req.data::<WsSessionManager>().unwrap();
        let maybe_session = ws_session_manager.get_session(session_id).await;
        if maybe_session.is_none() {
            let msg = "Agent websocket session ended. Set a new session id or reconnect Agent";
            return Err(Error::from_string(msg, StatusCode::NOT_FOUND));
        }
        Ok(Self {
            id: conv_id.0,
            session: maybe_session.unwrap().clone(),
        })
    }
}
