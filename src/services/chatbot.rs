use super::session_manager::{ConversationState, SessionData, Message, MessageRole};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, PartialEq)]
pub enum Intent {
    Greeting,
    WebsiteRequest,
    Pricing,
    Contact,
    Help,
    Services,
    Unknown,
}

pub fn detect_intent(msg: &str) -> Intent {
    let msg_lower = msg.to_lowercase();

    if msg_lower.contains("hello") || msg_lower.contains("hi") || msg_lower.contains("hey") {
        Intent::Greeting
    } else if msg_lower.contains("web site") || msg_lower.contains("website") || msg_lower.contains("e-commerce") {
        Intent::WebsiteRequest
    } else if msg_lower.contains("price") || msg_lower.contains("cost") || msg_lower.contains("quote") {
        Intent::Pricing
    } else if msg_lower.contains("email") || msg_lower.contains("phone") || msg_lower.contains("contact") {
        Intent::Contact
    } else if msg_lower.contains("help") || msg_lower.contains("support") || msg_lower.contains("assist") {
        Intent::Help
    } else if msg_lower.contains("service") || msg_lower.contains("offer") || msg_lower.contains("what do you do") {
        Intent::Services
    } else {
        Intent::Unknown
    }
}

// "AI" Knowledge Base: Keywords the bot listens for
const KNOWLEDGE_BASE: &[&str] = &[
    "website", "ecommerce", "shop", "blog", "landing", // Types
    "rust", "python", "javascript", "react", "vue", "node", // Tech
    "design", "ui", "ux", "logo", // Design
    "seo", "marketing", "ads", // Marketing
    "mobile", "app", "ios", "android", // Mobile
    "api", "database", "sql", "cloud" // Backend
];

#[derive(Serialize)]
struct MistralChatRequest {
    model: String,
    messages: Vec<MistralMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MistralMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct MistralChatResponse {
    choices: Vec<MistralChoice>,
}

#[derive(Deserialize, Debug)]
struct MistralChoice {
    message: MistralMessage,
}

pub async fn generate_reply(current_state: ConversationState, user_msg: &str, mut data: SessionData, history: Vec<Message>) -> (String, ConversationState, SessionData) {
    // 1. Continuous Learning: Analyze message for keywords
    let new_topics = extract_keywords(user_msg);
    for topic in new_topics {
        if !data.detected_keywords.contains(&topic) {
            data.detected_keywords.push(topic);
        }
    }

    // Global reset command
    if user_msg.trim().eq_ignore_ascii_case("reset") || user_msg.trim().eq_ignore_ascii_case("cancel") {
        return (
            "Conversation reset. How can I help you?".to_string(),
            ConversationState::Idle,
            SessionData::default()
        );
    }

    // Global status command (Multitasking: checking state while in flow)
    if user_msg.trim().eq_ignore_ascii_case("status") {
         let status_msg = match current_state {
            ConversationState::Idle => "I am currently idle. Ask me about websites!",
            ConversationState::AskingName => "I am waiting for your name.",
            ConversationState::AskingEmail => "I am waiting for your email address.",
            ConversationState::AskingBudget => "I am waiting for your budget estimate.",
            ConversationState::AskingProjectDetails => "I am waiting for your project details.",
            ConversationState::AskingProjectConfirmation => "I am waiting for confirmation to start a project inquiry.",
        };
        return (status_msg.to_string(), current_state, data);
    }

    // Context-Aware Interruptions (Multitasking: answering FAQs during a flow)
    // If the user asks a FAQ question while we are in a flow, answer it but stay in the flow.
    if current_state != ConversationState::Idle {
        let intent = detect_intent(user_msg);
        let interruption_reply = match intent {
            Intent::Pricing => Some("Our websites start at $1000."),
            Intent::Contact => {
                // Avoid triggering contact info if the user is actually inputting their email
                if current_state != ConversationState::AskingEmail {
                    Some("You can reach us at contact@webagency.com or call +1-555-0199.")
                } else {
                    None
                }
            },
            Intent::Help => Some("I'm currently asking for your details. You can type 'reset' to start over."),
            _ => None,
        };

        if let Some(reply) = interruption_reply {
            let reminder = get_reminder(&current_state);
            return (format!("{} {}", reply, reminder), current_state, data);
        }
    }

    match current_state {
        ConversationState::Idle => {
            let intent = detect_intent(user_msg);
            match intent {
                Intent::Greeting => (
                    "Hello! Welcome to our agency. How can I help you today?".to_string(),
                    ConversationState::Idle,
                    data
                ),
                Intent::WebsiteRequest => {
                    // Reset data for a new request
                    data = SessionData::default();
                    (
                        "We'd love to help with your website. To start, could you tell me your name?".to_string(),
                        ConversationState::AskingName,
                        data
                    )
                },
                Intent::Pricing => (
                    "Our websites start at $1000. Would you like to start a project inquiry?".to_string(),
                    ConversationState::AskingProjectConfirmation,
                    data
                ),
                Intent::Contact => (
                    "You can reach us at contact@webagency.com or call +1-555-0199.".to_string(),
                    ConversationState::Idle,
                    data
                ),
                Intent::Help => (
                    "I can help you with pricing, contact info, or starting a new project.".to_string(),
                    ConversationState::Idle,
                    data
                ),
                Intent::Services => (
                    "We offer Web Development, App Design, and SEO optimization.".to_string(),
                    ConversationState::Idle,
                    data
                ),
                Intent::Unknown => {
                    if let Some(ai_reply) = call_mistral(&history).await {
                        (ai_reply, ConversationState::Idle, data)
                    } else {
                        (format!("I didn't quite catch that: '{}'. Try asking for a 'website' or 'price'.", user_msg), ConversationState::Idle, data)
                    }
                },
            }
        }
        ConversationState::AskingName => {
            let name = user_msg.trim();
            if is_valid_name(name) {
                data.name = Some(name.to_string());
                (format!("Nice to meet you, {}! What is your email address?", name), ConversationState::AskingEmail, data)
            } else {
                ("That doesn't look like a valid name. Please use letters only (no numbers).".to_string(), ConversationState::AskingName, data)
            }
        }
        ConversationState::AskingEmail => {
            if user_msg.contains("@") {
                data.email = Some(user_msg.to_string());
                ("Thanks! What is your estimated budget for this project?".to_string(), ConversationState::AskingBudget, data)
            } else {
                ("That doesn't look like a valid email. Please try again.".to_string(), ConversationState::AskingEmail, data)
            }
        }
        ConversationState::AskingBudget => {
            if user_msg.chars().any(|c| c.is_numeric()) {
                data.budget = Some(user_msg.to_string());
                ("Got it. Finally, please describe your project requirements briefly.".to_string(), ConversationState::AskingProjectDetails, data)
            } else {
                ("Please enter a valid budget amount (e.g. 1000).".to_string(), ConversationState::AskingBudget, data)
            }
        }
        ConversationState::AskingProjectDetails => {
            // Generate Smart Report
            let topics_str = if data.detected_keywords.is_empty() {
                "General Inquiry".to_string()
            } else {
                data.detected_keywords.join(", ").to_uppercase()
            };

            let summary = format!(
                "✅ **REPORT GENERATED**\n\nI have compiled your project brief:\n- **Client**: {}\n- **Contact**: {}\n- **Budget**: {}\n- **Detected Topics**: {}\n\nWe will review this context and contact you shortly!",
                data.name.as_deref().unwrap_or("Unknown"),
                data.email.as_deref().unwrap_or("N/A"),
                data.budget.as_deref().unwrap_or("N/A"),
                topics_str
            );
            (summary, ConversationState::Idle, data)
        }
        ConversationState::AskingProjectConfirmation => {
            let msg_lower = user_msg.trim().to_lowercase();
            if msg_lower.contains("yes") || msg_lower.contains("sure") || msg_lower.contains("ok") || msg_lower.contains("yep") {
                 // Reset data for new project
                 data = SessionData::default();
                 (
                    "Great! To start, could you tell me your name?".to_string(),
                    ConversationState::AskingName,
                    data
                 )
            } else {
                 (
                    "Okay, no problem. Let me know if you need anything else.".to_string(),
                    ConversationState::Idle,
                    data
                 )
            }
        }
    }
}

fn get_reminder(state: &ConversationState) -> &str {
    match state {
        ConversationState::AskingName => "Could you please tell me your name?",
        ConversationState::AskingEmail => "Please provide your email address.",
        ConversationState::AskingBudget => "What is your estimated budget?",
        ConversationState::AskingProjectDetails => "Please briefly describe your project.",
        ConversationState::AskingProjectConfirmation => "Would you like to start a project inquiry? (yes/no)",
        _ => "",
    }
}

pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() > 1
        && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace() || c == '-' || c == '\'')
}

// Basic NLP: Extract known keywords from text
fn extract_keywords(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut found = Vec::new();
    
    // Split by non-alphanumeric characters to get clean words
    let words: Vec<&str> = lower.split(|c: char| !c.is_alphanumeric()).collect();

    for &keyword in KNOWLEDGE_BASE {
        if words.contains(&keyword) {
            found.push(keyword.to_string());
        }
    }
    found
}

async fn call_mistral(history: &[Message]) -> Option<String> {
    let api_key = match env::var("MISTRAL_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            eprintln!("Error: MISTRAL_API_KEY environment variable is not set.");
            return None;
        }
    };
    
    let client = reqwest::Client::new();
    let mut messages = vec![
        MistralMessage { role: "system".to_string(), content: "You are a helpful assistant for a web agency. Keep answers concise.".to_string() },
    ];

    // Limit history to last 10 messages to save tokens
    let start_index = history.len().saturating_sub(10);
    for msg in &history[start_index..] {
        let role = match msg.role {
            MessageRole::User => "user",
            MessageRole::Bot => "assistant",
        };
        messages.push(MistralMessage { role: role.to_string(), content: msg.content.clone() });
    }

    let request_body = MistralChatRequest {
        model: "mistral-tiny".to_string(),
        messages,
    };

    let res = match client.post("https://api.mistral.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error sending request to Mistral: {}", e);
                return None;
            }
        };

    if !res.status().is_success() {
        eprintln!("Mistral API returned error: {}", res.status());
        return None;
    }

    let chat_res = match res.json::<MistralChatResponse>().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error parsing Mistral response: {}", e);
            return None;
        }
    };

    chat_res.choices.first().map(|c| c.message.content.clone())
}
