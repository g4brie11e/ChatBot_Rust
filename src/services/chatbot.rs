use super::session_manager::{ConversationState, SessionData};

#[derive(Debug, PartialEq)]
pub enum Intent {
    Greeting,
    WebsiteRequest,
    Pricing,
    Contact,
    Help,
    Unknown,
}

pub fn detect_intent(msg: &str) -> Intent {
    let msg_lower = msg.to_lowercase();

    if msg_lower.contains("hello") || msg_lower.contains("hi") || msg_lower.contains("hey") {
        Intent::Greeting
    } else if msg_lower.contains("web site") || msg_lower.contains("e-commerce") {
        Intent::WebsiteRequest
    } else if msg_lower.contains("price") || msg_lower.contains("cost") || msg_lower.contains("quote") {
        Intent::Pricing
    } else if msg_lower.contains("email") || msg_lower.contains("phone") || msg_lower.contains("contact") {
        Intent::Contact
    } else if msg_lower.contains("help") || msg_lower.contains("support") || msg_lower.contains("assist") {
        Intent::Help
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

pub fn generate_reply(current_state: ConversationState, user_msg: &str, mut data: SessionData) -> (String, ConversationState, SessionData) {
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
                    Some("You can reach us at contact@webagency.com.")
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
                    ConversationState::Idle, // Could transition to AskingName if we added a Yes/No check
                    data
                ),
                Intent::Contact => (
                    "You can reach us at contact@webagency.com.".to_string(),
                    ConversationState::Idle,
                    data
                ),
                Intent::Help => (
                    "I can help you with pricing, contact info, or starting a new project.".to_string(),
                    ConversationState::Idle,
                    data
                ),
                Intent::Unknown => (
                    format!("I didn't quite catch that: '{}'. Try asking for a 'website' or 'price'.", user_msg),
                    ConversationState::Idle,
                    data
                ),
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
    }
}

fn get_reminder(state: &ConversationState) -> &str {
    match state {
        ConversationState::AskingName => "Could you please tell me your name?",
        ConversationState::AskingEmail => "Please provide your email address.",
        ConversationState::AskingBudget => "What is your estimated budget?",
        ConversationState::AskingProjectDetails => "Please briefly describe your project.",
        _ => "",
    }
}

fn is_valid_name(name: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_intent() {
        assert_eq!(detect_intent("Hello there"), Intent::Greeting);
        assert_eq!(detect_intent("I want a web site"), Intent::WebsiteRequest);
        assert_eq!(detect_intent("What is the price?"), Intent::Pricing);
        assert_eq!(detect_intent("Give me your email"), Intent::Contact);
        assert_eq!(detect_intent("I need help"), Intent::Help);
        assert_eq!(detect_intent("random text"), Intent::Unknown);
    }

    #[test]
    fn test_conversation_flow() {
        // 1. Start
        let mut data = SessionData::default();
        let (reply, state, data) = generate_reply(ConversationState::Idle, "I want a web site", data);
        assert_eq!(state, ConversationState::AskingName);
        assert!(reply.contains("name"));

        // 2. Provide Name
        let (reply, state, data) = generate_reply(state, "John", data);
        assert_eq!(state, ConversationState::AskingEmail);
        assert_eq!(data.name.as_deref(), Some("John"));
        assert!(reply.contains("John"));
        assert!(reply.contains("email"));
        
        // 3. Provide Email
        let (reply, state, data) = generate_reply(state, "john@test.com", data);
        assert_eq!(state, ConversationState::AskingBudget);
        assert_eq!(data.email.as_deref(), Some("john@test.com"));
        assert!(reply.contains("budget"));

        // 4. Provide Budget
        let (reply, state, data) = generate_reply(state, "5000", data);
        assert_eq!(state, ConversationState::AskingProjectDetails);
        assert!(reply.contains("requirements"));

        // 5. Finish
        let (reply, state, _data) = generate_reply(state, "I need a blog", data);
        assert_eq!(state, ConversationState::Idle);
        assert!(reply.contains("Thank you"));
        assert!(reply.contains("5000"));
    }

    #[test]
    fn test_interruption_logic() {
        let mut data = SessionData::default();
        // 1. Start flow
        let (reply, state, data) = generate_reply(ConversationState::Idle, "website", data);
        assert_eq!(state, ConversationState::AskingName);

        // 2. Interrupt with pricing question
        let (reply, state, data) = generate_reply(state, "what is the price?", data);
        assert_eq!(state, ConversationState::AskingName); // State should not change
        assert!(reply.contains("$1000")); // Should answer question
        assert!(reply.contains("name")); // Should remind user

        // 3. Resume flow
        let (reply, state, _) = generate_reply(state, "John", data);
        assert_eq!(state, ConversationState::AskingEmail);
    }

    #[test]
    fn test_name_validation() {
        assert!(is_valid_name("John Doe"));
        assert!(is_valid_name("Jean-Pierre"));
        assert!(!is_valid_name("User123")); // Contains numbers
        assert!(!is_valid_name(""));       // Empty
        assert!(!is_valid_name("A"));      // Too short
    }

    #[test]
    fn test_keyword_extraction_and_report() {
        let mut data = SessionData::default();
        // User mentions "rust" and "api"
        let (_, _, data) = generate_reply(ConversationState::Idle, "I need a Rust backend API", data);
        
        assert!(data.detected_keywords.contains(&"rust".to_string()));
        assert!(data.detected_keywords.contains(&"api".to_string()));
        assert!(!data.detected_keywords.contains(&"python".to_string()));
    }
}
