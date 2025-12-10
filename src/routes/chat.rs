// src/routes/chat.rs
use axum::{extract::State, Json};

use crate::message::{ChatRequest, ChatResponse};
use crate::state::SharedState;
use crate::services::session_manager::MessageRole;
use crate::services::chatbot::{generate_reply}; 

pub async fn chat_handler(
    State(state): State<SharedState>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    let session_id = match payload.session_id {
        Some(ref s) if !s.trim().is_empty() => {
            state.sessions.ensure_session(s).await;
            s.clone()
        }
        _ => state.sessions.create_session().await,
    };

    let trimmed = payload.message.trim();

    if trimmed.is_empty() {
        return Json(ChatResponse {
            session_id,
            reply: "Je n’ai pas reçu de message. Peux-tu écrire quelque chose ?".to_string(),
        });
    }

    // Append user message
    state.sessions.append_message(&session_id, MessageRole::User, trimmed).await;

    let history_raw = state.sessions.get_history(&session_id).await.unwrap_or_default();
    let history_text: Vec<String> = history_raw
        .iter()
        .map(|m| m.content.clone())
        .collect();
    
    let reply = generate_reply(&history_text, trimmed);
    state.sessions.append_message(&session_id, MessageRole::Bot, &reply).await;

    Json(ChatResponse { session_id, reply })
}
