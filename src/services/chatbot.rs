#[derive(Debug)]
pub enum Intent {
    Greeting,
    WebsiteRequest,
    Unknown,
}

pub fn detect_intent(msg: &str) -> Intent {
    let msg_lower = msg.to_lowercase();

    if msg_lower.contains("hello") || msg_lower.contains("hi") {
        Intent::Greeting
    } else if msg_lower.contains("web site") || msg_lower.contains("e-commerce") {
        Intent::WebsiteRequest
    } else {
        Intent::Unknown
    }
}


pub fn generate_reply(history: &Vec<String>, user_msg: &str) -> String {
    use Intent::*;

    let intent = detect_intent(user_msg);

    match intent {
        Greeting => {
            if history.is_empty() {
                "Hy again can I help you ?".to_string()
            } else {
                "Hi, how can I help you".to_string()
            }
        }

        WebsiteRequest => {
            if history.iter().any(|m| m.to_lowercase().contains("web site")) {
                "Have you some suggestion about youor project".to_string()
            } else {
                "Do you have a specific ides of your project and your price ? ".to_string()
            }
        }

        Unknown => {
            if history.is_empty() {
                format!("Welcome : {}", user_msg)
            } else {
                format!("I didnt quit understood : {}", user_msg)
            }
        }
    }
}
