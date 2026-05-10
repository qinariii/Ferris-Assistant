# ─── Planner Stage (cargo-chef) ──────────────────────────────────────────────
FROM lukemathwalker/cargo-chef:latest-rust-1.88-slim-bookworm AS chef
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# ─── Prepare recipe (hanya tergantung Cargo.toml/Cargo.lock) ────────────────
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo chef prepare --recipe-path recipe.json

# ─── Build Stage ─────────────────────────────────────────────────────────────
FROM chef AS builder

# Step 1: Cook dependencies dari recipe — di-cache selama Cargo.toml/lock sama
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Step 2: Build source code — hanya recompile kode kamu, bukan dependencies
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release

# ─── Runtime Stage ───────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libpq5 curl && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Ambil binary hasil build
COPY --from=builder /app/target/release/ferris-bot ./ferris-bot
COPY locales/ /app/locales/

ENV RUST_LOG=info
EXPOSE 8080 8443

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./ferris-bot"]