# syntax=docker/dockerfile:1

# ─── Build Stage ─────────────────────────────────────────────────────────────
FROM rust:1.88-slim-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build dengan cache mount untuk cargo registry, git, dan target
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp /app/target/release/ferris-bot /app/ferris-bot-bin

# ─── Runtime Stage ───────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/ferris-bot-bin ./ferris-bot
COPY locales/ /app/locales/

ENV RUST_LOG=info
EXPOSE 8080 8443

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./ferris-bot"]