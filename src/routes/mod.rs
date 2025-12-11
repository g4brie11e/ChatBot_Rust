// src/routes/mod.rs
pub mod chat;

use axum::{Router, routing::{post, get}};
use crate::state::SharedState;
use chat::chat_handler;
use axum::routing::get_service;
use tower_http::services::ServeDir;

pub fn create_router() -> Router<SharedState> {
    Router::new()
        .route("/chat", post(chat_handler))
        .route("/health", get(|| async { "OK" }))
        .fallback_service(ServeDir::new("public"))
}
