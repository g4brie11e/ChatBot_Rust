# chatbot-backend

This is a Rust-based project for a chatbot application, built with the [Axum](https://github.com/tokio-rs/axum) web framework.

## Project Grading Criteria & Highlights

### 1. Usability
I focused on making the application responsive and user-friendly. It detects the user's language (English, French, Spanish, Polish) automatically. I also implemented PDF report generation using `tokio::task::spawn_blocking` to ensure the server stays responsive even during heavy tasks. Administrative endpoints are included to monitor leads and metrics in real-time (so we can get the info easier). But they are protected so to try it you have to do a `curl -H "x-admin-key: secret123" http://localhost:3000/admin/leads`

### 2. Usage of Rust Functionalities
I used `tokio` and `axum` for the async architecture. To manage shared state safely, I used `Arc<RwLock<T>>`, which prevents data races. I also used Rust's `enum` types and `match` patterns extensively for control flow, and `thiserror` for clean error handling. Background tasks are used to manage session cleanup automatically.

### 3. Programming Challenges
The main challenge was building the Finite State Machine (FSM) for the conversation logic. I used a hybrid approach: local rules handle the standard flow for speed, while a Mistral AI (a french AI that gives api key to their ai for free) client handles unknown inputs. I also had to be careful not to block the async executor during PDF generation, ensuring CPU-bound tasks are offloaded to a blocking thread pool.

### 4. Size of Project
The project is structured into modules to keep the code clean. It includes integration tests in the `tests/` directory. I integrated several libraries like `reqwest`, `printpdf`, and `serde` to build a complete backend.

### 5. Quality of Code
I tried to keep the code maintainable by separating the web layer from the business logic. I used strong typing (Enums) instead of strings wherever possible to prevent invalid states, and avoided magic strings by using constants.

### 6. Technical Decisions & Justifications
*   **`HashMap` for Session Storage**: I used a `HashMap` to store active sessions because it offers O(1) access time. Since every incoming message requires looking up the user's session by ID, this structure ensures low latency even with many concurrent users.
*   **`Arc<RwLock<...>>` for Concurrency**: To share state across the async web server, I wrapped data structures in `Arc` (for shared ownership) and `RwLock`. I chose `RwLock` over `Mutex` because the application reads state (checking conversation status) more frequently than it's modified,this allow higher concurrency.
*   **`tokio::task::spawn_blocking`**: The PDF generation library (`printpdf`) is synchronous and CPU-bound. Running it directly in an async handler would freeze the event loop. I used `spawn_blocking` to offload this work to a dedicated thread pool, ensuring the server remains responsive.
*   **Hybrid Architecture (FSM + LLM)**: I used a Finite State Machine (FSM) for the core business logic (collecting leads) to ensure 100% reliability and data structure. I only use the Mistral LLM as a fallback for "Unknown" intents, balancing control with flexibility..
*   **Context Window Management**: To optimize API costs and latency when communicating with Mistral AI, I implemented a sliding window mechanism that only sends the last 10 messages. This ensures the LLM has enough context to be helpful without exceeding token limits.
*   **JSON-based Persistence**: For storing leads, I chose an append-only JSON file strategy (`leads.json`) instead of a complex database. This keeps the application lightweight and easy to deploy without requiring external dependencies like PostgreSQL.
*   **Axum State Injection**: I utilized Axum's `State` extractor to inject shared resources into handlers. This avoids global static variables, making the application easier to test and ensuring thread safety across the async runtime.

---

## Features

### Smart Multilingual Support
The chatbot is designed to be accessible to a global audience. It includes an **automatic language detection system** that analyzes incoming messages for specific keywords.
*   **Supported Languages**: English, French, Spanish, and Polish.
*   **Context Awareness**: Once a language is detected (user says "Hola"), the session should automatically switch locks into that language, and all subsequent system prompts (asking for name, budget, etc.) are localized.

### Hybrid AI Architecture (Reliability + Flexibility)
The bot uses a dual-engine approach to handle user inputs:
1.  **Deterministic FSM (Finite State Machine)**: Critical business logic—such as collecting lead information (Name, Email, Budget)—is handled by strict Rust code. This ensures that data is captured accurately 100% of the time without hallucination.
2.  **Mistral AI Fallback**: If the user's intent is "Unknown" (not a standard command like "price" or "website"), the system forwards the conversation history to the **Mistral LLM**. This allows the bot to answer general questions or engage in small talk, making it feel more "intelligent" without compromising the reliability of the lead generation flow.

### Admin Dashboard & Analytics
To help site administrators manage the application, a secured API layer is included:
*   **Lead Management**: Admins can retrieve a full list of collected leads via a JSON endpoint (`/admin/leads`).
*   **Live Metrics**: The system tracks usage statistics in real-time, such as which languages are most popular and which intents (Pricing, Contact, etc.) are triggered most often.

### Automated PDF Reporting
Upon completing the lead generation flow, the application performs a CPU-intensive task without blocking the server:
*   **Generation**: It compiles the user's inputs and detected keywords into a professional PDF document.
*   **Delivery**: A direct download link is generated and sent to the user in the chat window.

### High-Performance Session Management
*   **In-Memory Storage**: Sessions are stored in a thread-safe `HashMap` for O(1) access speeds.
*   **Concurrency**: Uses `Arc<RwLock<T>>` to allow multiple concurrent readers (users chatting) while ensuring safe writes (updating state).

## Getting Started

### Prerequisites
- Rust (latest stable)
- A Mistral API Key (set in `.env`)

### Running the Application

1.  **Build and run the project:**
    ```bash
    cargo run
    ```

2.  The server will start and listen locally on the port 3000.

## Project Structure

```
chatbot-backend/
├── src/
│   ├── main.rs             # Entry point
│   ├── routes/             # API Routes
│   ├── services/           # Business Logic (Chatbot, Sessions, Metrics, PDF)
│   ├── error.rs            # Error handling
│   ├── message.rs          # Data models
│   └── state.rs            # Shared application state
├── tests/                  # Integration tests
├── public/                 # Frontend
└── Cargo.toml
```

## API

The primary API endpoint is for the chat functionality.

- `POST /chat`: Send a message to the chatbot.
- `GET /health`: Check server status.

### Admin API & Monitoring

The application includes a secured administrative layer designed for monitoring and data retrieval. These endpoints are protected by a custom authentication middleware that verifies the presence of an API key.

**Authentication:**
All requests to `/admin/*` must include the HTTP header `x-admin-key`.
*   **Key**: `secret123` (configured in `src/routes/mod.rs`)

**Endpoints:**

1.  **`GET /admin/leads`** : Returns a JSON array of all leads collected during chat sessions (saved in `leads.json`).Used by sales teams to export potential client details.
    *   **Example**:
        ```bash
        curl -H "x-admin-key: secret123" http://localhost:3000/admin/leads
        ```

2.  **`GET /admin/metrics`** :Provides real-time telemetry on chatbot usage (intents and languages).
    *   **Example**:
        ```bash
        curl -H "x-admin-key: secret123" http://localhost:3000/admin/metrics
        ```

### Example Request

- **Greeting**: "Hello", "Bonjour", "Hola" -> Bot adapts language.
- **Website**: "I want a website" -> Starts lead collection flow.
- **Unknown**: "What is the capital of France?" -> Falls back to AI.