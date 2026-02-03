use axum::{
    extract::State,
    Json,
};
use crate::{
    message::{ChatRequest, ChatResponse},
    state::SharedState,
    services::{
        session_manager::{MessageRole, ConversationState},
        chatbot::generate_reply,
    },
    error::AppError,
};
use crate::services::metrics_manager::MetricsData;
use crate::services::report_generator::generate_pdf_report;
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
    let history = state.sessions.get_history(&session_id).await.unwrap_or_default();
    
    let (mut reply, next_state, next_data) = generate_reply(current_state.clone(), trimmed, current_data.clone(), history, &state.metrics).await;

    // Check if the flow just finished (Transition from AskingProjectDetails -> Idle)
    if current_state == ConversationState::AskingProjectDetails && next_state == ConversationState::Idle {
        // Save the lead to a file
        if let Ok(json) = serde_json::to_string(&next_data) {
            let _ = save_lead_to_file(&json).await;
        }

        // Generate Downloadable Report
        if let Ok(url) = generate_pdf_report(&session_id, &next_data).await {
            reply.push_str(&format!(
                "\n\n<a href='{}' target='_blank' style='display: inline-block; padding: 10px 20px; background-color: #0084ff; color: white; text-decoration: none; border-radius: 20px; font-weight: bold; margin-top: 5px;'>Download PDF Report</a>", 
                url
            ));
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

// New Handler: Get Metrics
pub async fn get_metrics_handler(State(state): State<SharedState>) -> Json<MetricsData> {
    Json(state.metrics.get_metrics().await)
}
