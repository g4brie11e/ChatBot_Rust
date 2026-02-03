use chatbot_backend::message::ChatResponse;
use chatbot_backend::routes::create_router;
use chatbot_backend::state::AppState;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use std::time::Duration;
use tower::util::ServiceExt;

#[tokio::test]
async fn test_chat_endpoint() {
    let state = Arc::new(AppState::new(Duration::from_secs(60)));
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

    // Select Language
    let req = Request::builder()
        .method("POST")
        .uri("/chat")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"message": "English", "session_id": null}"#))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let chat_resp: ChatResponse = serde_json::from_slice(&body_bytes).unwrap();

    let session_id = chat_resp.session_id;

    // User asks for website
    let req = Request::builder()
        .method("POST")
        .uri("/chat")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"message": "I want a website", "session_id": "{}"}}"#,
            session_id
        )))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let chat_resp: ChatResponse = serde_json::from_slice(&body_bytes).unwrap();

    assert!(chat_resp.reply.contains("name")); // Bot should ask for name

    // User sends Name
    let req = Request::builder()
        .method("POST")
        .uri("/chat")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"message": "Alice", "session_id": "{}"}}"#,
            session_id
        )))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let chat_resp: ChatResponse = serde_json::from_slice(&body_bytes).unwrap();

    // Bot should remember we are in AskingName state and transition to AskingEmail
    assert!(chat_resp.reply.contains("Alice"));
    assert!(chat_resp.reply.contains("email"));
}

#[tokio::test]
async fn test_reset_command_integration() {
    let state = Arc::new(AppState::new(Duration::from_secs(60)));
    let app = create_router().with_state(state);

    // Start flow
    let req = Request::builder()
        .method("POST")
        .uri("/chat")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"message": "website", "session_id": null}"#))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let chat_resp: ChatResponse = serde_json::from_slice(&body_bytes).unwrap();
    let session_id = chat_resp.session_id;

    // Send Reset
    let req = Request::builder()
        .method("POST")
        .uri("/chat")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"message": "reset", "session_id": "{}"}}"#,
            session_id
        )))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let chat_resp: ChatResponse = serde_json::from_slice(&body_bytes).unwrap();

    assert!(chat_resp.reply.contains("reset"));
}
