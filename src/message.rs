use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub session_id: String,
    pub reply: String,
}
