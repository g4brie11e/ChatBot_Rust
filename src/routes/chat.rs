// src/routes/chat.rs
use axum::{extract::State, Json};
use axum::routing::get_service;
use tower_http::services::ServeDir;
use crate::{
    message::{ChatRequest, ChatResponse},
    state::SharedState,
    services::{
        session_manager::MessageRole,
        chatbot::generate_reply,
    },
    error::AppError,
    Router,
    routes::post,
    get,
};

pub async fn chat_handler(
    State(state): State<SharedState>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {

    let session_id = match &payload.session_id {
        Some(s) if !s.trim().is_empty() => {
            state.sessions.ensure_session(s).await;
            s.clone()
        }
        _ => state.sessions.create_session().await,
    };

    let trimmed = payload.message.trim();

    // a check if empty
    // if trimmed.is_empty() {
    //     return Ok(Json(ChatResponse {
    //         session_id,
    //         reply: "I didn't get anything, can you write again ?".to_string(),
    //     }));
    // }

        //used the personalized error handling
        if trimmed.is_empty() {
        return Err(AppError::BadRequest("Message cannot be empty".to_string()));
    }
    // Append user message
    state.sessions.append_message(&session_id, MessageRole::User, trimmed).await;

    let history_raw = state.sessions.get_history(&session_id).await.unwrap_or_default();
    let history_text: Vec<String> = history_raw.iter().map(|m| m.content.clone()).collect();
    
    let reply = generate_reply(&history_text, trimmed);
   state.sessions.append_message(&session_id, MessageRole::Bot, &reply).await;

    Ok(Json(ChatResponse { session_id, reply }))
}

pub fn create_router() -> Router<SharedState> {
    Router::new()
        .route("/chat", post(chat_handler))
        .route("/health", get(|| async { "OK" }))
        // Serve the `public/` folder at the root
        .nest_service("/", get_service(ServeDir::new("public")).handle_error(|err| async move {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erreur server: {}", err),
            )
        }))
}
