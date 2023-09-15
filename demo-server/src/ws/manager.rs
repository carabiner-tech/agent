use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use super::session::WsSession;

#[derive(Clone, Default)]
pub struct WsSessionManager {
    sessions: Arc<Mutex<HashMap<uuid::Uuid, WsSession>>>,
}

impl WsSessionManager {
    pub async fn get_session(&self, id: &uuid::Uuid) -> Option<WsSession> {
        let sessions = self.sessions.lock().await;
        sessions.get(id).cloned()
    }

    pub async fn first_session(&self) -> Option<WsSession> {
        let sessions = self.sessions.lock().await;
        sessions.values().next().cloned()
    }

    pub async fn add_session(&self, session: WsSession) {
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session.id, session);
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
