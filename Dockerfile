# 1. Build Stage
FROM rust:1.84-slim-bookworm as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# 2. Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app

# Install SSL certificates for HTTPS (needed for Mistral API)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binary and assets
COPY --from=builder /app/target/release/chatbot-backend .
COPY --from=builder /app/public ./public
# Note: In production, pass API keys via environment variables, don't copy .env
COPY --from=builder /app/.env .

EXPOSE 3000
CMD ["./chatbot-backend"]