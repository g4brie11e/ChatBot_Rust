// src/main.rs (relevant parts)
use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

mod routes;
mod state;
mod message;
mod error;
mod services;

use crate::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState::new(Duration::from_secs(60 * 60)));

    {
        let sessions_clone = state.sessions.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60 * 5));
            loop {
                interval.tick().await;
                let removed = sessions_clone.purge_expired().await;
                if removed > 0 {
                    tracing::info!(removed, "purged expired sessions");
                }
            }
        });
    }

    let app = routes::create_router()
        .with_state(state.clone());

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
