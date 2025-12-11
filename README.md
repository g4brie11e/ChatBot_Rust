# chatbot-backend

This is a Rust-based project for a chatbot application, built with the [Axum](https://github.com/tokio-rs/axum) web framework.

## Features

- Asynchronous Web Framework : Built on `tokio` and `axum`.
- Session Management : Includes basic session management with automatic expiration.
- JSON API : Provides a JSON-based API for chatbot interactions.

## Getting Started

### Running the Application

1.  **Build and run the project:**
    ```bash
    cargo run
    ```

2.  The server will start and listen locally on the port 3000.

## Project Structure

chatbot-backend/
├── src/
│   ├── main.rs
│   ├── routes/
│   │   ├── mod.rs
│   │   └── chat.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── session_manager.rs
│   │   └── chatbot.rs
│   ├── errors.rs
│   ├── message.rs
│   ├── state.rs
│   └── rules.rs
├── public/
│   └── index.html
└── Cargo.toml


## API

The primary API endpoint is for the chat functionality.

- `/chat` This endpoint is used to send a message to the chatbot and receive a response.
- `/health` This endpoint is used to check the health status of the server.

### Example Request

- Hello, Hi ->  "Hi, how can I help you"
- web site -> Do you have a specific ides of your project and your price ? 
- else -> I didnt quit understood :

## Project Milestones

### Iteration 1 – MVP

At the end of Milestone 1, we have a functional rule-based chatbot:
- Rust server running with Axum
- Working /chat API endpoint
- Session system with message history
- Rule-based reply logic
- API successfully tested and documented