pub fn generate_reply(history: &Vec<String>, user_msg: &str) -> String {
    let msg = user_msg.to_lowercase();

    if msg.contains("bonjour") {
        return "Salut ! Comment puis-je t’aider ?".to_string();
    }

    if history.is_empty() {
        return format!("Bienvenue ! Tu me dis : {}", user_msg);
    }

    format!("Tu as dit : {}", user_msg)
}
