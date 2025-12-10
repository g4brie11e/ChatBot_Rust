use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppState {
    pub sessions: Arc<Mutex<HashMap<String, Vec<String>>>>,
}
