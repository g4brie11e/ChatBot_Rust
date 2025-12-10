pub fn generate_reply(history: &Vec<String>, user_msg: &str) -> String {
    let msg = user_msg.to_lowercase();

    // Vérifier d'abord les mots-clés
    if msg.contains("bonjour") {
        return "Salut ! Comment puis-je t’aider ?".to_string();
    }

    // Puis vérifier si c’est la première interaction
    if history.is_empty() {
        return format!("Bienvenue ! Tu me dis : {}", user_msg);
    }

    // Sinon écho simple
    format!("Tu as dit : {}", user_msg)
}
