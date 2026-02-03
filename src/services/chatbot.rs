use super::session_manager::{ConversationState, SessionData, Message, MessageRole};
use super::metrics_manager::MetricsManager;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::error;

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

const GREETING_KEYWORDS: &[&str] = &["hello", "hi", "hey", "bonjour", "cześć", "hola"];
const WEBSITE_KEYWORDS: &[&str] = &["web site", "website", "e-commerce", "site web", "strona", "sitio web"];
const PRICING_KEYWORDS: &[&str] = &["price", "cost", "quote", "prix", "cena", "precio"];
const CONTACT_KEYWORDS: &[&str] = &["email", "phone", "contact", "kontakt", "contacto"];
const HELP_KEYWORDS: &[&str] = &["help", "support", "assist", "aide", "pomoc", "ayuda"];
const SERVICES_KEYWORDS: &[&str] = &["service", "offer", "what do you do", "usługi", "servicios"];

pub fn detect_intent(msg: &str) -> Intent {
    let msg_lower = msg.to_lowercase();

    if GREETING_KEYWORDS.iter().any(|k| msg_lower.contains(k)) {
        Intent::Greeting
    } else if WEBSITE_KEYWORDS.iter().any(|k| msg_lower.contains(k)) {
        Intent::WebsiteRequest
    } else if PRICING_KEYWORDS.iter().any(|k| msg_lower.contains(k)) {
        Intent::Pricing
    } else if CONTACT_KEYWORDS.iter().any(|k| msg_lower.contains(k)) {
        Intent::Contact
    } else if HELP_KEYWORDS.iter().any(|k| msg_lower.contains(k)) {
        Intent::Help
    } else if SERVICES_KEYWORDS.iter().any(|k| msg_lower.contains(k)) {
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

pub async fn generate_reply(current_state: ConversationState, user_msg: &str, mut data: SessionData, history: Vec<Message>, metrics: &MetricsManager) -> (String, ConversationState, SessionData) {
    // 0. Language Inference: Attempt to detect language change on every message
    if let Some(lang) = infer_language(user_msg) {
        data.language = lang;
    }
    
    // Track language usage
    metrics.increment_language(&data.language).await;

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
            "Conversation reset. Please select your language: English, Polish, French, Spanish.".to_string(),
            ConversationState::AskingLanguage,
            SessionData::default()
        );
    }

    // Global status command (Multitasking: checking state while in flow)
    if user_msg.trim().eq_ignore_ascii_case("status") {
         let status_msg = match current_state {
            ConversationState::AskingLanguage => "I am waiting for you to select a language.",
            ConversationState::Idle => "I am currently idle.",
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
        match intent {
            Intent::Pricing | Intent::Contact | Intent::Help => {
                // Track interruption intents
                metrics.increment_intent(&format!("{:?}", intent)).await;
            }
            _ => {}
        }
        
        let interruption_reply = match intent {
            Intent::Pricing => Some(get_localized_msg("pricing_info", &data.language)),
            Intent::Contact => {
                // Avoid triggering contact info if the user is actually inputting their email
                if current_state != ConversationState::AskingEmail {
                    Some(get_localized_msg("contact_info", &data.language))
                } else {
                    None
                }
            },
            Intent::Help => Some(get_localized_msg("help_interruption", &data.language)),
            _ => None,
        };

        if let Some(reply) = interruption_reply {
            let reminder = get_reminder(&current_state);
            return (format!("{} {}", reply, reminder), current_state, data);
        }
    }

    match current_state {
        ConversationState::AskingLanguage => {
            let msg_lower = user_msg.trim().to_lowercase();
            if msg_lower.contains("english") || msg_lower == "en" {
                data.language = "en".to_string();
                (get_localized_msg("greeting", "en"), ConversationState::Idle, data)
            } else if msg_lower.contains("polish") || msg_lower.contains("polski") || msg_lower == "pl" {
                data.language = "pl".to_string();
                (get_localized_msg("greeting", "pl"), ConversationState::Idle, data)
            } else if msg_lower.contains("french") || msg_lower.contains("francais") || msg_lower == "fr" {
                data.language = "fr".to_string();
                (get_localized_msg("greeting", "fr"), ConversationState::Idle, data)
            } else if msg_lower.contains("spanish") || msg_lower.contains("espanol") || msg_lower == "es" {
                data.language = "es".to_string();
                (get_localized_msg("greeting", "es"), ConversationState::Idle, data)
            } else if let Some(lang) = infer_language(user_msg) {
                data.language = lang.clone();
                (get_localized_msg("greeting", &lang), ConversationState::Idle, data)
            } else {
                ("Please select your language: English, Polish, French, Spanish.\nChoisissez votre langue: English, Polish, French, Spanish.\nWybierz język: English, Polish, French, Spanish.\nElige tu idioma: English, Polish, French, Spanish.".to_string(), ConversationState::AskingLanguage, data)
            }
        }
        ConversationState::Idle => {
            let intent = detect_intent(user_msg);
            metrics.increment_intent(&format!("{:?}", intent)).await;
            match intent {
                Intent::Greeting => (
                    get_localized_msg("greeting", &data.language),
                    ConversationState::Idle,
                    data
                ),
                Intent::WebsiteRequest => {
                    // Reset data for a new request
                    data = SessionData {
                        language: data.language.clone(),
                        ..SessionData::default()
                    };
                    (
                        get_localized_msg("website_start", &data.language),
                        ConversationState::AskingName,
                        data
                    )
                },
                Intent::Pricing => (
                    get_localized_msg("pricing_start", &data.language),
                    ConversationState::AskingProjectConfirmation,
                    data
                ),
                Intent::Contact => (
                    get_localized_msg("contact_info", &data.language),
                    ConversationState::Idle,
                    data
                ),
                Intent::Help => (
                    get_localized_msg("help_info", &data.language),
                    ConversationState::Idle,
                    data
                ),
                Intent::Services => (
                    get_localized_msg("services_info", &data.language),
                    ConversationState::Idle,
                    data
                ),
                Intent::Unknown => {
                    if let Some(ai_reply) = call_mistral(&history).await {
                        (ai_reply, ConversationState::Idle, data)
                    } else {
                        (format!("{} '{}'.", get_localized_msg("unknown", &data.language), user_msg), ConversationState::Idle, data)
                    }
                },
            }
        }
        ConversationState::AskingName => {
            let name = user_msg.trim();
            if is_valid_name(name) {
                data.name = Some(name.to_string());
                (format!("{}, {}! {}", get_localized_msg("nice_meet", &data.language), name, get_localized_msg("ask_email", &data.language)), ConversationState::AskingEmail, data)
            } else {
                (get_localized_msg("invalid_name", &data.language), ConversationState::AskingName, data)
            }
        }
        ConversationState::AskingEmail => {
            if user_msg.contains("@") {
                data.email = Some(user_msg.to_string());
                (get_localized_msg("ask_budget", &data.language), ConversationState::AskingBudget, data)
            } else {
                (get_localized_msg("invalid_email", &data.language), ConversationState::AskingEmail, data)
            }
        }
        ConversationState::AskingBudget => {
            if user_msg.chars().any(|c| c.is_numeric()) {
                data.budget = Some(user_msg.to_string());
                (get_localized_msg("ask_details", &data.language), ConversationState::AskingProjectDetails, data)
            } else {
                (get_localized_msg("invalid_budget", &data.language), ConversationState::AskingBudget, data)
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
            if msg_lower.contains("yes") || msg_lower.contains("sure") || msg_lower.contains("ok") || msg_lower.contains("yep") || msg_lower.contains("tak") || msg_lower.contains("oui") || msg_lower.contains("si") {
                 // Reset data for new project
                 data = SessionData {
                    language: data.language.clone(),
                    ..SessionData::default()
                 };
                 (
                    get_localized_msg("website_start", &data.language),
                    ConversationState::AskingName,
                    data
                 )
            } else {
                 (
                    get_localized_msg("ok_no_problem", &data.language),
                    ConversationState::Idle,
                    data
                 )
            }
        }
    }
}

fn get_reminder(state: &ConversationState) -> &str {
    match state {
        ConversationState::AskingLanguage => "Please select your language.",
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

fn infer_language(msg: &str) -> Option<String> {
    let msg_lower = msg.to_lowercase();
    // Spanish
    if msg_lower.contains("hola") || msg_lower.contains("ayuda") || msg_lower.contains("contacto") || msg_lower.contains("precio") || msg_lower.contains("servicios") {
        return Some("es".to_string());
    }
    // French
    if msg_lower.contains("bonjour") || msg_lower.contains("aide") || msg_lower.contains("site web") {
        return Some("fr".to_string());
    }
    // Polish
    if msg_lower.contains("cześć") || msg_lower.contains("czesc") || msg_lower.contains("pomoc") || msg_lower.contains("kontakt") || msg_lower.contains("cena") || msg_lower.contains("usługi") || msg_lower.contains("strona") {
        return Some("pl".to_string());
    }
    // English
    if msg_lower.contains("hello") || msg_lower.contains("hi") || msg_lower.contains("help") || msg_lower.contains("price") || msg_lower.contains("website") {
        return Some("en".to_string());
    }
    None
}

fn get_localized_msg(key: &str, lang: &str) -> String {
    match (lang, key) {
        // Polish
        ("pl", "greeting") => "Cześć! Witamy w naszej agencji. Jak mogę Ci pomóc?".to_string(),
        ("pl", "website_start") => "Chętnie pomożemy. Na początek, jak masz na imię?".to_string(),
        ("pl", "pricing_start") => "Nasze strony zaczynają się od 1000$. Czy chcesz rozpocząć wycenę?".to_string(),
        ("pl", "pricing_info") => "Nasze strony zaczynają się od 1000$.".to_string(),
        ("pl", "contact_info") => "Możesz się z nami skontaktować pod adresem contact@webagency.com lub +1-555-0199.".to_string(),
        ("pl", "help_info") => "Mogę pomóc z wyceną, kontaktem lub rozpoczęciem nowego projektu.".to_string(),
        ("pl", "services_info") => "Oferujemy tworzenie stron www, aplikacji i SEO.".to_string(),
        ("pl", "unknown") => "Nie zrozumiałem:".to_string(),
        ("pl", "nice_meet") => "Miło Cię poznać".to_string(),
        ("pl", "ask_email") => "Jaki jest Twój adres email?".to_string(),
        ("pl", "invalid_name") => "To nie wygląda na poprawne imię. Użyj tylko liter.".to_string(),
        ("pl", "ask_budget") => "Dzięki! Jaki jest szacowany budżet?".to_string(),
        ("pl", "invalid_email") => "Niepoprawny email. Spróbuj ponownie.".to_string(),
        ("pl", "ask_details") => "Zrozumiałem. Na koniec opisz krótko swój projekt.".to_string(),
        ("pl", "invalid_budget") => "Wpisz poprawną kwotę (np. 1000).".to_string(),
        ("pl", "ok_no_problem") => "Ok, nie ma problemu. Daj znać jeśli będziesz czegoś potrzebować.".to_string(),
        ("pl", "help_interruption") => "Aktualnie pytam o szczegóły. Wpisz 'reset' aby zacząć od nowa.".to_string(),

        // French
        ("fr", "greeting") => "Bonjour! Bienvenue dans notre agence. Comment puis-je vous aider ?".to_string(),
        ("fr", "website_start") => "Nous serions ravis de vous aider. Pour commencer, quel est votre nom ?".to_string(),
        ("fr", "pricing_start") => "Nos sites commencent à 1000$. Voulez-vous lancer une demande ?".to_string(),
        ("fr", "pricing_info") => "Nos sites commencent à 1000$.".to_string(),
        ("fr", "contact_info") => "Vous pouvez nous joindre à contact@webagency.com ou +1-555-0199.".to_string(),
        ("fr", "help_info") => "Je peux vous aider avec les prix, le contact ou un nouveau projet.".to_string(),
        ("fr", "services_info") => "Nous proposons du développement Web, des applications et du SEO.".to_string(),
        ("fr", "unknown") => "Je n'ai pas bien compris :".to_string(),
        ("fr", "nice_meet") => "Enchanté".to_string(),
        ("fr", "ask_email") => "Quelle est votre adresse email ?".to_string(),
        ("fr", "invalid_name") => "Ce nom ne semble pas valide. Utilisez uniquement des lettres.".to_string(),
        ("fr", "ask_budget") => "Merci ! Quel est votre budget estimé ?".to_string(),
        ("fr", "invalid_email") => "Email invalide. Veuillez réessayer.".to_string(),
        ("fr", "ask_details") => "C'est noté. Enfin, décrivez brièvement votre projet.".to_string(),
        ("fr", "invalid_budget") => "Veuillez entrer un montant valide (ex: 1000).".to_string(),
        ("fr", "ok_no_problem") => "D'accord, pas de problème.".to_string(),
        ("fr", "help_interruption") => "Je demande actuellement vos détails. Tapez 'reset' pour recommencer.".to_string(),

        // Spanish
        ("es", "greeting") => "¡Hola! Bienvenido a nuestra agencia. ¿Cómo puedo ayudarte?".to_string(),
        ("es", "website_start") => "Nos encantaría ayudar. Para empezar, ¿cuál es tu nombre?".to_string(),
        ("es", "pricing_start") => "Nuestros sitios comienzan en $1000. ¿Quieres iniciar una consulta?".to_string(),
        ("es", "pricing_info") => "Nuestros sitios comienzan en $1000.".to_string(),
        ("es", "contact_info") => "Puedes contactarnos en contact@webagency.com o +1-555-0199.".to_string(),
        ("es", "help_info") => "Puedo ayudarte con precios, contacto o un nuevo proyecto.".to_string(),
        ("es", "services_info") => "Ofrecemos desarrollo web, aplicaciones y SEO.".to_string(),
        ("es", "unknown") => "No entendí bien:".to_string(),
        ("es", "nice_meet") => "Encantado de conocerte".to_string(),
        ("es", "ask_email") => "¿Cuál es tu correo electrónico?".to_string(),
        ("es", "invalid_name") => "Ese nombre no parece válido. Usa solo letras.".to_string(),
        ("es", "ask_budget") => "¡Gracias! ¿Cuál es tu presupuesto estimado?".to_string(),
        ("es", "invalid_email") => "Correo inválido. Inténtalo de nuevo.".to_string(),
        ("es", "ask_details") => "Entendido. Finalmente, describe brevemente tu proyecto.".to_string(),
        ("es", "invalid_budget") => "Ingresa un monto válido (ej. 1000).".to_string(),
        ("es", "ok_no_problem") => "Bien, no hay problema.".to_string(),
        ("es", "help_interruption") => "Estoy pidiendo tus detalles. Escribe 'reset' para reiniciar.".to_string(),

        // Default (English)
        (_, "greeting") => "Hello! Welcome to our agency. How can I help you today?".to_string(),
        (_, "website_start") => "We'd love to help with your website. To start, could you tell me your name?".to_string(),
        (_, "pricing_start") => "Our websites start at $1000. Would you like to start a project inquiry?".to_string(),
        (_, "pricing_info") => "Our websites start at $1000.".to_string(),
        (_, "contact_info") => "You can reach us at contact@webagency.com or call +1-555-0199.".to_string(),
        (_, "help_info") => "I can help you with pricing, contact info, or starting a new project.".to_string(),
        (_, "services_info") => "We offer Web Development, App Design, and SEO optimization.".to_string(),
        (_, "unknown") => "I didn't quite catch that:".to_string(),
        (_, "nice_meet") => "Nice to meet you".to_string(),
        (_, "ask_email") => "What is your email address?".to_string(),
        (_, "invalid_name") => "That doesn't look like a valid name. Please use letters only (no numbers).".to_string(),
        (_, "ask_budget") => "Thanks! What is your estimated budget for this project?".to_string(),
        (_, "invalid_email") => "That doesn't look like a valid email. Please try again.".to_string(),
        (_, "ask_details") => "Got it. Finally, please describe your project requirements briefly.".to_string(),
        (_, "invalid_budget") => "Please enter a valid budget amount (e.g. 1000).".to_string(),
        (_, "ok_no_problem") => "Okay, no problem. Let me know if you need anything else.".to_string(),
        (_, "help_interruption") => "I'm currently asking for your details. You can type 'reset' to start over.".to_string(),
        (_, _) => "Text missing".to_string(),
    }
}

async fn call_mistral(history: &[Message]) -> Option<String> {
    let api_key = match env::var("MISTRAL_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            error!("MISTRAL_API_KEY environment variable is not set.");
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
                error!("Error sending request to Mistral: {}", e);
                return None;
            }
        };

    if !res.status().is_success() {
        error!("Mistral API returned error: {}", res.status());
        return None;
    }

    let chat_res = match res.json::<MistralChatResponse>().await {
        Ok(r) => r,
        Err(e) => {
            error!("Error parsing Mistral response: {}", e);
            return None;
        }
    };

    chat_res.choices.first().map(|c| c.message.content.clone())
}
