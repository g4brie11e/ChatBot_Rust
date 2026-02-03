// src/routes/mod.rs
pub mod chat;

use axum::{Router, routing::{post, get}};
use crate::state::SharedState;
use chat::{chat_handler, get_leads_handler, get_metrics_handler};
use tower_http::services::ServeDir;

pub fn create_router() -> Router<SharedState> {
    Router::new()
        .route("/chat", post(chat_handler))
        .route("/admin/leads", get(get_leads_handler))
        .route("/admin/metrics", get(get_metrics_handler))
        .route("/health", get(|| async { "OK" }))
        .fallback_service(ServeDir::new("public"))
}
