// src/services/session_manager.rs
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: Instant,
}

#[derive(Clone, Debug)]
pub enum MessageRole {
    User,
    Bot,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub id: String,
    pub messages: Vec<Message>,
    pub last_active: Instant,
}

impl Session {
    pub fn new(id: impl Into<String>) -> Self {
        let now = Instant::now();
        Self { id: id.into(), messages: Vec::new(), last_active: now }
    }
}

#[derive(Clone)]
pub struct SessionManager {
    inner: Arc<RwLock<HashMap<String, Session>>>,
    ttl: Duration,
}

impl Debug for SessionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionManager")
            .field("ttl", &self.ttl)
            .finish()
    }
}

impl SessionManager {
    // Create a new manager 
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    // Create a fresh session and return its id.
    pub async fn create_session(&self) -> String {
        let id = Uuid::new_v4().to_string();
        let session = Session::new(id.clone());

        let mut guard = self.inner.write().await;
        guard.insert(id.clone(), session);
        id
    }
    

    // Ensure there's a session with this id.
    pub async fn ensure_session(&self, id: &str) -> String {
        {
            let guard = self.inner.read().await;
            if guard.contains_key(id) {
                return id.to_string();
            }
        }
        let mut guard = self.inner.write().await;
        let session = Session::new(id.to_string());
        guard.insert(id.to_string(), session);
        id.to_string()
    }

    // Append a message to a session's history and touch last_active.
    pub async fn append_message(&self, session_id: &str, role: MessageRole, content: impl Into<String>) -> usize {
        let mut guard = self.inner.write().await;
        let entry = guard.entry(session_id.to_string()).or_insert_with(|| Session::new(session_id.to_string()));
        let msg = Message {
            role,
            content: content.into(),
            timestamp: Instant::now(),
        };
        entry.messages.push(msg);
        entry.last_active = Instant::now();
        entry.messages.len()
    }

    /// Get a copy of the session history
    pub async fn get_history(&self, session_id: &str) -> Option<Vec<Message>> {
        let guard = self.inner.read().await;
        guard.get(session_id).map(|s| s.messages.clone())
    }

    /// Remove a session by id
    pub async fn remove_session(&self, session_id: &str) -> bool {
        let mut guard = self.inner.write().await;
        guard.remove(session_id).is_some()
    }

    /// Remove sessions idle longer than ttl. Returns number removed.
    pub async fn purge_expired(&self) -> usize {
        let mut guard = self.inner.write().await;
        let now = Instant::now();
        let before = guard.len();
        guard.retain(|_, s| now.duration_since(s.last_active) < self.ttl);
        before - guard.len()
    }

    /// Number of sessions 
    pub async fn len(&self) -> usize {
        let guard = self.inner.read().await;
        guard.len()
    }

    /// List session ids
    pub async fn list_session_ids(&self) -> Vec<String> {
        let guard = self.inner.read().await;
        guard.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn basic_session_flow() {
        let mgr = SessionManager::new(Duration::from_secs(60));
        let sid = mgr.create_session().await;
        assert!(!sid.is_empty());
        let len = mgr.append_message(&sid, MessageRole::User, "hello").await;
        assert_eq!(len, 1);
        let history = mgr.get_history(&sid).await.unwrap();
        assert_eq!(history.len(), 1);
        assert!(mgr.remove_session(&sid).await);
    }
}
