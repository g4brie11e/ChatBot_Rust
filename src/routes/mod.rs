// src/routes/mod.rs
pub mod chat;

use crate::state::SharedState;
use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};
use chat::{chat_handler, get_leads_handler, get_metrics_handler};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

pub fn create_router() -> Router<SharedState> {
    let admin_routes = Router::new()
        .route("/leads", get(get_leads_handler))
        .route("/metrics", get(get_metrics_handler))
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .route("/chat", post(chat_handler))
        .nest("/admin", admin_routes)
        .route("/health", get(|| async { "OK" }))
        .fallback_service(ServeDir::new("public"))
        .layer(TraceLayer::new_for_http())
}

async fn auth_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    // API Key check.
    match req.headers().get("x-admin-key") {
        Some(val) if val == "secret123" => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
