// src/message.rs
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ChatRequest {
    pub session_id: Option<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub session_id: String,
    pub reply: String,
}
