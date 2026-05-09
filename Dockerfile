# ─── Build Stage ──────────────────────────────────────────────────────────────
FROM rust:1.79-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/

RUN cargo build --release

# ─── Runtime Stage ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/ferris-bot /app/ferris-bot
COPY locales/ /app/locales/

ENV RUST_LOG=info

CMD ["./ferris-bot"]
