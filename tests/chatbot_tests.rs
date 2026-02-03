use chatbot_backend::services::chatbot::{Intent, detect_intent, generate_reply, is_valid_name};
use chatbot_backend::services::metrics_manager::MetricsManager;
use chatbot_backend::services::session_manager::{
    ConversationState, Message, MessageRole, SessionData,
};
use std::time::Instant;

#[test]
fn test_detect_intent() {
    assert_eq!(detect_intent("Hello there"), Intent::Greeting);
    assert_eq!(detect_intent("I want a web site"), Intent::WebsiteRequest);
    assert_eq!(detect_intent("What is the price?"), Intent::Pricing);
    assert_eq!(detect_intent("Give me your email"), Intent::Contact);
    assert_eq!(detect_intent("I need help"), Intent::Help);
    assert_eq!(
        detect_intent("What services do you offer?"),
        Intent::Services
    );
    assert_eq!(detect_intent("random text"), Intent::Unknown);
}

#[tokio::test]
async fn test_conversation_flow() {

    let data = SessionData::default();
    let metrics = MetricsManager::new();
    let (reply, state, data) = generate_reply(
        ConversationState::Idle,
        "I want a web site",
        data,
        vec![],
        &metrics,
    )
    .await;
    assert_eq!(state, ConversationState::AskingName);
    assert!(reply.contains("name"));

    // Name
    let (reply, state, data) = generate_reply(state, "John", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::AskingEmail);
    assert_eq!(data.name.as_deref(), Some("John"));
    assert!(reply.contains("John"));
    assert!(reply.contains("email"));

    // Email
    let (reply, state, data) = generate_reply(state, "john@test.com", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::AskingBudget);
    assert_eq!(data.email.as_deref(), Some("john@test.com"));
    assert!(reply.contains("budget"));

    // Budget
    let (reply, state, data) = generate_reply(state, "5000", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::AskingProjectDetails);
    assert!(reply.contains("requirements"));

    // Finish
    let (reply, state, _data) =
        generate_reply(state, "I need a blog", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::Idle);
    assert!(reply.contains("REPORT GENERATED"));
    assert!(reply.contains("5000"));
}

#[tokio::test]
async fn test_interruption_logic() {
    let data = SessionData::default();
    let metrics = MetricsManager::new();
    // Start
    let (_reply, state, data) =
        generate_reply(ConversationState::Idle, "website", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::AskingName);

    // Interrupt with pricing question
    let (reply, state, data) =
        generate_reply(state, "what is the price?", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::AskingName); 
    assert!(reply.contains("$1000")); // Should answer 
    assert!(reply.contains("name")); // Should remind user to give the name 

    // Resume flow
    let (_reply, state, _) = generate_reply(state, "John", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::AskingEmail);
}

#[test]
fn test_name_validation() {
    assert!(is_valid_name("John Doe"));
    assert!(is_valid_name("Jean-Pierre"));
    // Contains numbers
    assert!(!is_valid_name("User123")); 
    // Empty
    assert!(!is_valid_name("")); 
    // Too short
    assert!(!is_valid_name("A"));
}

#[tokio::test]
async fn test_keyword_extraction_and_report() {
    let data = SessionData::default();
    let metrics = MetricsManager::new();
    let (_, _, data) = generate_reply(
        ConversationState::Idle,
        "I need a Rust backend API",
        data,
        vec![],
        &metrics,
    )
    .await;

    assert!(data.detected_keywords.contains(&"rust".to_string()));
    assert!(data.detected_keywords.contains(&"api".to_string()));
    assert!(!data.detected_keywords.contains(&"python".to_string()));
}

#[tokio::test]
async fn test_project_confirmation_flow() {
    let data = SessionData::default();
    let metrics = MetricsManager::new();
    // Ask for price
    let (reply, state, data) =
        generate_reply(ConversationState::Idle, "price", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::AskingProjectConfirmation);
    assert!(reply.contains("start a project inquiry"));

    let (reply, state, _) = generate_reply(state, "yes", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::AskingName);
    assert!(reply.contains("name"));
}

#[tokio::test]
async fn test_services_intent() {
    let data = SessionData::default();
    let metrics = MetricsManager::new();
    let (reply, state, _) = generate_reply(
        ConversationState::Idle,
        "what service do you offer?",
        data,
        vec![],
        &metrics,
    )
    .await;
    assert_eq!(state, ConversationState::Idle);
    assert!(reply.contains("Web Development"));
}

#[tokio::test]
async fn test_classic_and_ai_response() {
    dotenvy::dotenv().ok();
    let data = SessionData::default();
    let metrics = MetricsManager::new();

    // Test Classic Response 
    let (reply, state, _) = generate_reply(
        ConversationState::Idle,
        "help",
        data.clone(),
        vec![],
        &metrics,
    )
    .await;
    assert_eq!(state, ConversationState::Idle);
    assert!(reply.contains("pricing, contact info"));

    // Test AI Response (Fallback for unknown intent)
    // "What is the capital of France?" is NOT a known intent, so it goes to Mistral AI
    let question = "Capital of France?"; 
    let history = vec![Message {
        role: MessageRole::User,
        content: question.to_string(),
        timestamp: Instant::now(),
    }];
    let (reply, _state, _) =
        generate_reply(ConversationState::Idle, question, data, history, &metrics).await;

    // check behavior based on API Key presence
    let reply_lower = reply.to_lowercase();
    // Accept generic polite response too
    let is_ai_reply = reply_lower.contains("paris") || reply_lower.contains("assist");
    let is_fallback = reply.contains("I didn't quite catch that");

    assert!(
        is_ai_reply || is_fallback,
        "AI response unexpected. Got: '{}'",
        reply
    );
}

#[tokio::test]
async fn test_language_switching_spanish() {
    let data = SessionData::default();
    let metrics = MetricsManager::new();
    let (reply, state, data) =
        generate_reply(ConversationState::Idle, "hola", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::Idle);
    assert_eq!(data.language, "es");
    assert!(reply.contains("Hola"));
}

#[tokio::test]
async fn test_language_switching_polish_keyword() {
    let data = SessionData::default();
    let metrics = MetricsManager::new();

    // Say "cześć" should switch to PL
    let (reply, state, data) =
        generate_reply(ConversationState::Idle, "cześć", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::Idle);
    assert_eq!(data.language, "pl");
    assert!(reply.contains("Cześć"));

    // Say "strona" continue in PL
    let (reply, state, data) = generate_reply(state, "strona", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::AskingName);
    assert_eq!(data.language, "pl");
    assert!(reply.contains("Chętnie pomożemy"));
}

#[tokio::test]
async fn test_language_switching_french() {
    let data = SessionData::default();
    let metrics = MetricsManager::new();
    let (reply, state, data) =
        generate_reply(ConversationState::Idle, "bonjour", data, vec![], &metrics).await;
    assert_eq!(state, ConversationState::Idle);
    assert_eq!(data.language, "fr");
    assert!(reply.contains("Bonjour"));
}

#[tokio::test]
async fn test_metrics_increment() {
    let data = SessionData::default();
    let metrics = MetricsManager::new();

    // "price" triggers Intent::Pricing and language "en"
    let (_reply, _state, _data) = generate_reply(
        ConversationState::Idle,
        "price",
        data.clone(),
        vec![],
        &metrics,
    )
    .await;

    let metrics_data = metrics.get_metrics().await;

    assert_eq!(metrics_data.intent_usage.get("Pricing"), Some(&1));
    assert_eq!(metrics_data.language_usage.get("en"), Some(&1));
}
