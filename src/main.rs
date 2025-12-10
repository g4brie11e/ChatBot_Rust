use axum::{Router, routing::get};
use tower_http::cors::CorsLayer;

mod routes;
mod state;
mod message;
mod rules;
mod services;

use state::AppState;

#[tokio::main]
async fn main() {
    let state = AppState {
        sessions: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
    };

    let cors = CorsLayer::very_permissive();

    let app = routes::create_router()
        .route("/", get(|| async { "YOU ARE CONNECTED " }))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 MVP chatbot running at http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
