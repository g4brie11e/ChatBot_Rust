// src/state.rs
use std::sync::Arc;
use std::time::Duration;

use crate::services::session_manager::SessionManager;
use crate::services::metrics_manager::MetricsManager;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub sessions: SessionManager,
    pub metrics: MetricsManager,
}

impl AppState {
    pub fn new(session_ttl: Duration) -> Self {
        Self {
            sessions: SessionManager::new(session_ttl),
            metrics: MetricsManager::new(),
        }
    }
}
