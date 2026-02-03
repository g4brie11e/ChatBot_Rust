use chatbot_backend::services::session_manager::{
    ConversationState, MessageRole, SessionData, SessionManager,
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn basic_session_flow() {
    let mgr = SessionManager::new(Duration::from_secs(60));
    let sid = mgr.create_session().await;
    assert!(!sid.is_empty());
    let len = mgr.append_message(&sid, MessageRole::User, "hello").await;
    assert_eq!(len, 1);
    let history = mgr.get_history(&sid).await.unwrap();
    assert_eq!(history.len(), 1);
    assert!(mgr.remove_session(&sid).await);
}

#[tokio::test]
async fn test_session_expiration() {
    let mgr = SessionManager::new(Duration::from_millis(10));
    let sid = mgr.create_session().await;

    // Wait for expiration
    sleep(Duration::from_millis(20)).await;

    let removed_count = mgr.purge_expired().await;
    assert_eq!(removed_count, 1, "Should have removed 1 expired session");
    assert!(
        !mgr.remove_session(&sid).await,
        "Session should already be gone"
    );
}

#[tokio::test]
async fn test_state_and_data_persistence() {
    let mgr = SessionManager::new(Duration::from_secs(60));
    let sid = mgr.create_session().await;

    // Test State
    assert_eq!(mgr.get_state(&sid).await, ConversationState::AskingLanguage);
    mgr.set_state(&sid, ConversationState::AskingName).await;
    assert_eq!(mgr.get_state(&sid).await, ConversationState::AskingName);

    // Test Data
    let data = SessionData {
        name: Some("Test".to_string()),
        ..Default::default()
    };
    mgr.set_data(&sid, data.clone()).await;
    let retrieved = mgr.get_data(&sid).await;
    assert_eq!(retrieved.name, Some("Test".to_string()));
}
