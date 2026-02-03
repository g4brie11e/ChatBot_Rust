// src/main.rs (relevant parts)
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use chatbot_backend::routes;
use chatbot_backend::state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
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

    let app = routes::create_router().with_state(state.clone());

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr_str = format!("0.0.0.0:{}", port);
    let addr: SocketAddr = addr_str.parse().expect("Invalid address");
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Error: Could not bind to address {}: {}", addr, e);
            std::process::exit(1);
        });

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
