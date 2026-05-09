# ─── Build Stage ──────────────────────────────────────────────────────────────
FROM rust:1.88-slim-bookworm AS builder

# Install dependencies sistem (apt-get install -y pkg-config libssl-dev)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Clone sudah digantikan oleh COPY
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/

# cargo build --release
RUN cargo build --release

# ─── Runtime Stage ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Ambil binary hasil build
COPY --from=builder /app/target/release/ferris-bot ./ferris-bot
COPY locales/ /app/locales/

ENV RUST_LOG=info

# ./target/release/ferris-bot
CMD ["./ferris-bot"]