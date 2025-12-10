use axum::{extract::State, Json};
use uuid::Uuid;

use crate::{
    state::AppState,
    message::{ChatRequest, ChatResponse},
    services::chatbot::generate_reply,  
    rules::apply_basic_rules,
};

pub async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    let session_id = payload.session_id.unwrap_or_else(|| Uuid::new_v4().to_string());

    let cleaned = apply_basic_rules(&payload.message);

    let reply = {
        let mut sessions = state.sessions.lock().await;
        let history = sessions.entry(session_id.clone()).or_default();

        let res = generate_reply(&history, &cleaned);
        history.push(cleaned);

        res
    };

    Json(ChatResponse { session_id, reply })
}
