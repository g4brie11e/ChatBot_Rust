use axum::{
    extract::State,
    routing::{get, post, get_service},
    Json, Router,
};
use tower_http::services::ServeDir;
use crate::{
    message::{ChatRequest, ChatResponse},
    state::SharedState,
    services::{
        session_manager::{MessageRole, ConversationState},
        chatbot::generate_reply,
    },
    error::AppError,
};
use crate::services::report_generator::generate_html_report;
use tokio::fs::{OpenOptions, read_to_string};
use tokio::io::AsyncWriteExt;

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

    if trimmed.is_empty() {
        return Err(AppError::BadRequest("Message cannot be empty".to_string()));
    }
    // Append user message
    state.sessions.append_message(&session_id, MessageRole::User, trimmed).await;

    // Get current state
    let current_state = state.sessions.get_state(&session_id).await;
    let current_data = state.sessions.get_data(&session_id).await;
    
    let (mut reply, next_state, next_data) = generate_reply(current_state.clone(), trimmed, current_data.clone());

    // Check if the flow just finished (Transition from AskingProjectDetails -> Idle)
    if current_state == ConversationState::AskingProjectDetails && next_state == ConversationState::Idle {
        // Save the lead to a file
        if let Ok(json) = serde_json::to_string(&next_data) {
            let _ = save_lead_to_file(&json).await;
        }

        // Generate Downloadable Report
        if let Ok(url) = generate_html_report(&session_id, &next_data).await {
            reply.push_str(&format!("\n\n📄 <a href='{}' target='_blank'>Download Project Report</a>", url));
        }
    }
    
    state.sessions.set_state(&session_id, next_state).await;
    state.sessions.set_data(&session_id, next_data).await;
    state.sessions.append_message(&session_id, MessageRole::Bot, &reply).await;

    Ok(Json(ChatResponse { session_id, reply }))
}

async fn save_lead_to_file(json_data: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("leads.json")
        .await?;
    
    file.write_all(json_data.as_bytes()).await?;
    file.write_all(b"\n").await?;
    Ok(())
}

// New Handler: Read leads.json and return as JSON array
pub async fn get_leads_handler() -> Json<Vec<serde_json::Value>> {
    let content = read_to_string("leads.json").await.unwrap_or_default();
    let leads = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    Json(leads)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::sync::Arc;
    use std::time::Duration;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for oneshot

    #[tokio::test]
    async fn test_chat_endpoint() {
        let state = Arc::new(AppState::new(Duration::from_secs(60)));
        // We use the router defined in this file for testing
        let app = create_router().with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/chat")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"message": "hello", "session_id": null}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_stateful_flow_integration() {
        let state = Arc::new(AppState::new(Duration::from_secs(60)));
        let app = create_router().with_state(state);

        // 1. User asks for website
        let req = Request::builder()
            .method("POST")
            .uri("/chat")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message": "I want a website", "session_id": null}"#))
            .unwrap();
        
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Parse response to get session_id
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let chat_resp: ChatResponse = serde_json::from_slice(&body_bytes).unwrap();
        
        let session_id = chat_resp.session_id;
        assert!(chat_resp.reply.contains("name")); // Bot should ask for name

        // 2. User sends Name (using the same session_id)
        let req = Request::builder()
            .method("POST")
            .uri("/chat")
            .header("content-type", "application/json")
            .body(Body::from(format!(r#"{{"message": "Alice", "session_id": "{}"}}"#, session_id)))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let chat_resp: ChatResponse = serde_json::from_slice(&body_bytes).unwrap();

        // Bot should remember we are in AskingName state and transition to AskingEmail
        assert!(chat_resp.reply.contains("Alice")); 
        assert!(chat_resp.reply.contains("email")); 
    }
}
