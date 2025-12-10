use axum::{Router, routing::post};

pub mod chat;

use crate::state::AppState;
use chat::chat_handler;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/chat", post(chat_handler))
        .route("/health", axum::routing::get(|| async { "OK" }))
}
