<div align="center">

# Ferris Assistant

**A high-performance Telegram group management bot built in Rust.**

Powered by [Teloxide](https://github.com/teloxide/teloxide) 0.17, async Tokio runtime, PostgreSQL for persistent storage, Redis for caching, and Prometheus for production observability.

[![Rust](https://img.shields.io/badge/Rust-2021_Edition-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Teloxide](https://img.shields.io/badge/Teloxide-0.17-4A90D9)](https://github.com/teloxide/teloxide)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-336791?logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![Redis](https://img.shields.io/badge/Redis-7-DC382D?logo=redis&logoColor=white)](https://redis.io/)
[![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?logo=docker&logoColor=white)](./Dockerfile)
[![License](https://img.shields.io/badge/License-MIT-22c55e)](LICENSE)

</div>

---

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Module Reference](#module-reference)
- [Tech Stack](#tech-stack)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [Requirements](#requirements)
- [Getting Started](#getting-started)
  - [Build from Source](#build-from-source)
  - [Docker Compose](#docker-compose)
- [Configuration](#configuration)
- [Running](#running)
  - [Polling Mode](#polling-mode)
  - [Webhook Mode](#webhook-mode)
- [Observability](#observability)
  - [Health Check](#health-check)
  - [Readiness Probe](#readiness-probe)
  - [Prometheus Metrics](#prometheus-metrics)
- [Internationalization](#internationalization)
- [Testing](#testing)
- [Development](#development)
- [Internal Architecture](#internal-architecture)
- [Contributing](#contributing)
- [Acknowledgements](#acknowledgements)
- [License](#license)

---

## Overview

Ferris Assistant is a feature-rich, production-ready Telegram bot designed for group administration. It provides 33 independent feature modules spanning moderation, anti-abuse, federation, customization, and developer tooling — all delivered with the safety and performance guarantees of Rust.

### Why Ferris Assistant?

- **Performance** — Zero-cost async I/O via Tokio; compiled to a single native binary.
- **Reliability** — Rust's ownership model eliminates data races and memory bugs at compile time.
- **Scalability** — PostgreSQL-backed persistence with optional Redis caching for hot paths.
- **Observability** — Prometheus metrics, structured logging, and HTTP health/readiness probes out of the box.
- **Extensibility** — Modular architecture makes adding new commands straightforward.
- **Deployment flexibility** — Long polling for development, webhooks for production; ships with a multi-stage Docker build and Compose stack.

---

## Key Features

| Category | Capabilities |
|---|---|
| **Moderation** | Ban, kick, mute, warn (configurable limits and modes), purge, pin management |
| **Anti-Abuse** | Antiflood, antispam rate limiter, word and sticker blacklists, CAPTCHA verification for new members |
| **Customization** | Welcome/goodbye messages, group rules, notes with `#hashtag` retrieval, keyword filters, message reactions |
| **Administration** | Promote/demote, custom admin titles, admin list, chat permissions, per-type message locks |
| **Logging** | Dedicated log channel for admin actions and significant events |
| **Federation** | Cross-group federations with shared ban lists (`/fban`, `/fedinfo`, etc.) |
| **Global Bans** | Bot-wide bans managed by sudo/dev users (`/gban`, `/ungban`) |
| **User Tools** | AFK status, user bio, `/id`, `/info`, settings panel, per-chat language selection |
| **Connections** | Manage groups privately via PM (`/connect`, `/disconnect`) |
| **Reports** | User-initiated reporting to alert group admins |
| **Backups** | Export and import full chat configuration (`/export`, `/import`) |
| **Cleaner** | Auto-remove service messages and "blue text" join notices |
| **Sed** | Regex find-and-replace on replied messages (`s/pattern/replacement/flags`) |
| **Dev Tools** | Sudo/dev management, broadcast, bot statistics, chat list, leave-chat utilities |
| **Observability** | HTTP health check, readiness probe, Prometheus metrics |

---

## Module Reference

All modules reside in [`src/modules/`](./src/modules). Each module encapsulates its own commands, handlers, and state logic.

| Module | Commands |
|---|---|
| **admin** | `/promote`, `/demote`, `/adminlist`, `/title` |
| **afk** | `/afk`, `brb` |
| **antiflood** | `/setflood`, `/flood`, `/setfloodmode` |
| **antispam** | *(automatic rate limiter — no user commands)* |
| **backups** | `/export`, `/import` |
| **bans** | `/ban`, `/sban`, `/dban`, `/tban`, `/unban`, `/kick`, `/dkick`, `/kickme` |
| **blacklist** | `/blacklist`, `/addblacklist`, `/rmblacklist`, `/blacklistmode` |
| **blstickers** | `/blsticker`, `/addblsticker`, `/rmblsticker`, `/blstickermode` |
| **captcha** | `/captcha`, `/captchamode`, `/captchatime`, `/captchaaction`, `/captchaattempts` |
| **chatpermissions** | `/setpermissions`, `/permissions` |
| **cleaner** | `/cleanservice`, `/cleanbluetext` |
| **connections** | `/connect`, `/disconnect`, `/connection`, `/allowconnect` |
| **devs** | `/addsudo`, `/remsudo`, `/adddev`, `/remdev`, `/teamusers`, `/chatinfo`, `/leavechat`, `/botstats`, `/broadcast` |
| **disable** | `/disable`, `/enable`, `/disabled`, `/disableable` |
| **feds** | `/newfed`, `/delfed`, `/joinfed`, `/leavefed`, `/fedinfo`, `/fedpromote`, `/feddemote`, `/fban`, `/unfban`, `/fbanlist`, `/fedchat`, `/fedrules` |
| **filters** | `/filter`, `/filters`, `/stop`, `/stopall` |
| **gbans** | `/gban`, `/ungban`, `/gbanlist` |
| **help** | `/help` *(interactive inline keyboard browser)* |
| **locks** | `/lock`, `/unlock`, `/locks`, `/locktypes` |
| **log_channel** | `/logchannel`, `/setlogchannel`, `/unsetlogchannel` |
| **mutes** | `/mute`, `/unmute`, `/tmute` |
| **notes** | `/save`, `/get`, `/notes`, `/saved`, `/clear`, `/clearall`, `#notename` |
| **pins** | `/pin`, `/unpin`, `/unpinall`, `/pinned` |
| **purges** | `/purge`, `/del` |
| **reactions** | `/addreaction`, `/removereaction`, `/reactions`, `/resetreactions` |
| **reports** | `/report`, `/reports`, `@admin` |
| **rules** | `/rules`, `/setrules`, `/clearrules` |
| **sed** | `s/pattern/replacement/flags` |
| **start** | `/start` |
| **userinfo** | `/setme`, `/me`, `/setbio`, `/bio` |
| **users** | `/id`, `/info`, `/settings`, `/setlang` |
| **warns** | `/warn`, `/dwarn`, `/swarn`, `/warns`, `/rmwarn`, `/resetwarns`, `/setwarnlimit`, `/setwarnmode` |
| **welcome** | `/welcome`, `/setwelcome`, `/resetwelcome`, `/goodbye`, `/setgoodbye`, `/resetgoodbye`, `/cleanservice` |

> Send `/help` to the bot to browse all modules via an interactive inline keyboard with usage examples.

---

## Tech Stack

| Layer | Technology |
|---|---|
| **Language** | Rust (2021 edition) |
| **Bot Framework** | [teloxide](https://crates.io/crates/teloxide) 0.17 |
| **Async Runtime** | [tokio](https://crates.io/crates/tokio) |
| **Database** | PostgreSQL 16 via [sqlx](https://crates.io/crates/sqlx) |
| **Cache** | Redis 7 via [redis](https://crates.io/crates/redis) (optional) |
| **HTTP Server** | [axum](https://crates.io/crates/axum) 0.7 |
| **Metrics** | [prometheus](https://crates.io/crates/prometheus) |
| **Migrations** | `sqlx::migrate!` |
| **Serialization** | serde, serde_json |
| **i18n** | [fluent-bundle](https://crates.io/crates/fluent-bundle), unic-langid |
| **Utilities** | regex, chrono, parking_lot, once_cell, uuid, rand, anyhow, thiserror |

---

## Architecture

```
┌─────────────┐      ┌───────────────┐      ┌─────────────────┐
│  Telegram   │─────▶│    Ferris     │─────▶│  PostgreSQL 16  │
│    API      │◀─────│  Assistant    │─────▶│  (persistent)   │
└─────────────┘      └───────┬───────┘      └─────────────────┘
                             │
                  ┌──────────┼──────────┐
                  │          │          │
            ┌─────▼───┐  ┌──▼───┐  ┌───▼────────┐
            │ Redis 7  │  │ Axum │  │ Prometheus │
            │ (cache)  │  │ HTTP │  │  /metrics  │
            └──────────┘  └──┬───┘  └────────────┘
                             │
                    ┌────────┼────────┐
                    │        │        │
              /health    /ready   /metrics
```

---

## Project Structure

```
Ferris-Assistant/
├── Cargo.toml                # Crate manifest and dependencies
├── Cargo.lock                # Dependency lock file
├── Dockerfile                # Multi-stage Docker build (cargo-chef)
├── docker-compose.yml        # Full stack: PostgreSQL + Redis + Bot
├── .env.example              # Environment variable template
├── LICENSE                   # MIT License
├── migrations/
│   └── 001_initial_schema.sql
├── locales/                  # Fluent i18n translations
│   ├── en/                   #   English  (29 files)
│   └── id/                   #   Indonesian (29 files)
├── tests/
│   ├── db_tests.rs           # Database integration tests
│   └── unit_tests.rs         # Unit tests (metrics, cache, etc.)
└── src/
    ├── main.rs               # Entry point — init, mode selection, dispatch
    ├── config.rs             # Environment-driven configuration
    ├── db/
    │   ├── mod.rs            # Connection pool initialization and migrations
    │   ├── models.rs         # Data models (sqlx::FromRow)
    │   └── queries.rs        # SQL query functions
    ├── handlers/
    │   └── mod.rs            # Update routing (commands, messages, callbacks)
    ├── keyboards/
    │   ├── mod.rs
    │   └── inline.rs         # Inline keyboard builders
    ├── modules/              # 33 feature modules
    │   ├── admin.rs          ├── antiflood.rs
    │   ├── afk.rs            ├── antispam.rs
    │   ├── backups.rs        ├── bans.rs
    │   ├── blacklist.rs      ├── blstickers.rs
    │   ├── captcha.rs        ├── chatpermissions.rs
    │   ├── cleaner.rs        ├── connections.rs
    │   ├── devs.rs           ├── disable.rs
    │   ├── feds.rs           ├── filters.rs
    │   ├── gbans.rs          ├── help.rs
    │   ├── locks.rs          ├── log_channel.rs
    │   ├── mutes.rs          ├── notes.rs
    │   ├── pins.rs           ├── purges.rs
    │   ├── reactions.rs      ├── reports.rs
    │   ├── rules.rs          ├── sed.rs
    │   ├── start.rs          ├── userinfo.rs
    │   ├── users.rs          ├── warns.rs
    │   └── welcome.rs
    └── utils/
        ├── cache.rs          # In-memory TTL cache
        ├── extraction.rs     # User and argument extraction helpers
        ├── formatting.rs     # HTML and greeting template utilities
        ├── http_server.rs    # Axum server (health, readiness, metrics)
        ├── i18n.rs           # Fluent translation loader
        ├── kick.rs           # Kick and ban utility functions
        ├── metrics.rs        # Prometheus metric definitions
        ├── permissions.rs    # Permission check utilities
        └── redis_cache.rs    # Redis caching layer (optional)
```

---

## Requirements

| Dependency | Version | Notes |
|---|---|---|
| **Rust** | ≥ 1.79 (stable) | Required |
| **PostgreSQL** | ≥ 14 (recommended 16) | Required |
| **Redis** | ≥ 6 | Optional — used for caching |
| **Telegram Bot Token** | — | Obtain from [@BotFather](https://t.me/BotFather) |
| **Docker + Compose** | ≥ 24 / v2 | Optional — for containerized deployment |

---

## Getting Started

### Build from Source

```bash
# Clone the repository
git clone https://github.com/qinariii/Ferris-Assistant.git
cd Ferris-Assistant

# Start backing services (or point to existing instances)
docker compose up -d postgres redis

# Configure environment variables
cp .env.example .env
$EDITOR .env    # Set TELOXIDE_TOKEN, OWNER_ID, DATABASE_URL, etc.

# Build and run
cargo build --release
./target/release/ferris-bot
```

### Docker Compose

```bash
# Configure environment variables
cp .env.example .env
$EDITOR .env

# Build and start all services
docker compose up -d --build

# Verify the bot is running
curl http://localhost:8080/health

# View logs
docker compose logs -f ferris-bot

# Tear down
docker compose down
```

PostgreSQL data is persisted in the `pgdata` Docker volume. Redis operates as an ephemeral cache.

---

## Configuration

All configuration is driven by environment variables. A `.env` file in the project root is loaded automatically at startup.

| Variable | Required | Default | Description |
|---|:---:|---|---|
| `TELOXIDE_TOKEN` | Yes | — | Bot token from [@BotFather](https://t.me/BotFather) |
| `OWNER_ID` | Yes | — | Telegram user ID of the bot owner |
| `BOT_NAME` | No | `FerrisBot` | Display name used in messages |
| `SUDO_USERS` | No | *(empty)* | Comma-separated user IDs with sudo privileges |
| `DATABASE_URL` | No | `postgres://ferris:ferris@localhost/ferris` | PostgreSQL connection string |
| `REDIS_URL` | No | *(disabled)* | Redis connection string (e.g., `redis://localhost:6379`) |
| `BOT_MODE` | No | `polling` | `polling` or `webhook` |
| `WEBHOOK_URL` | No | — | Public HTTPS URL (required when `BOT_MODE=webhook`) |
| `HTTP_PORT` | No | `8080` | Port for the health/metrics HTTP server |
| `RUST_LOG` | No | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |

<details>
<summary><strong>Example <code>.env</code></strong></summary>

```env
# Bot
TELOXIDE_TOKEN=123456:ABC-DEF...
BOT_NAME=FerrisBot
OWNER_ID=123456789
SUDO_USERS=111111111,222222222

# Database
DATABASE_URL=postgres://ferris:ferris@localhost/ferris

# Cache (optional)
REDIS_URL=redis://localhost:6379

# Mode
BOT_MODE=polling
# WEBHOOK_URL=https://yourdomain.com/webhook

# HTTP
HTTP_PORT=8080

# Logging
RUST_LOG=info
```

</details>

> **Tip:** To find your Telegram user ID, send `/id` to the bot after it starts, or message [@userinfobot](https://t.me/userinfobot).

---

## Running

### Polling Mode

The default mode. The bot long-polls the Telegram API — no public URL or TLS certificate is needed. Ideal for development and private deployments.

```bash
RUST_LOG=debug cargo run
```

### Webhook Mode

For production deployments behind a reverse proxy (Nginx, Caddy, etc.) with a valid TLS certificate:

```bash
BOT_MODE=webhook WEBHOOK_URL=https://bot.example.com/webhook cargo run --release
```

The webhook listener binds to `0.0.0.0:8443`. Configure your reverse proxy to forward HTTPS traffic to this port.

---

## Observability

An HTTP server (default port **8080**) exposes the following endpoints:

### Health Check

```
GET /health  →  200 OK
```

```json
{
  "status": "ok",
  "uptime_seconds": 3600,
  "version": "0.1.0"
}
```

Use this endpoint for **container liveness probes**.

### Readiness Probe

```
GET /ready  →  200 OK | 503 Service Unavailable
```

```json
{
  "status": "ready",
  "checks": {
    "database": true,
    "redis": true
  }
}
```

Returns **503** when any backing service is unreachable.

### Prometheus Metrics

```
GET /metrics
```

| Metric | Type | Description |
|---|---|---|
| `ferris_messages_total` | Counter | Total messages processed |
| `ferris_commands_total` | Counter | Commands processed (labeled by command name) |
| `ferris_callbacks_total` | Counter | Callback queries processed |
| `ferris_chats_active` | Gauge | Number of known chats |
| `ferris_users_active` | Gauge | Number of known users |
| `ferris_flood_triggers_total` | Counter | Antiflood trigger count |
| `ferris_gban_enforcements_total` | Counter | Global ban enforcements |

---

## Internationalization

Translations are managed with [Project Fluent](https://projectfluent.org/) and stored in [`locales/`](./locales) as `.ftl` files — one file per module per language.

**Supported languages:**

| Language | Directory |
|---|---|
| English | `locales/en/` |
| Indonesian | `locales/id/` |

Users can switch language per-chat using `/setlang`.

**Adding a new language:** Create a new directory `locales/<lang-code>/` mirroring the file structure of `locales/en/`, then translate all `.ftl` files.

---

## Testing

```bash
# Unit tests (no external dependencies required)
cargo test --test unit_tests

# Integration tests (requires a running PostgreSQL instance)
docker compose up -d postgres
DATABASE_URL=postgres://ferris:ferris@localhost/ferris_test cargo test --test db_tests
```

Integration tests run migrations automatically and clean up after themselves.

---

## Development

```bash
# Type-check without building
cargo check

# Run with debug logging
RUST_LOG=debug cargo run

# Format and lint
cargo fmt
cargo clippy --all-targets -- -D warnings

# Build optimized release binary
cargo build --release

# Run all tests
cargo test
```

**Database migrations:** Add new SQL files under `migrations/` (e.g., `002_add_column.sql`). Migrations are applied automatically on startup via `sqlx::migrate!`.

---

## Internal Architecture

The bot follows a layered processing pipeline:

1. **Initialization** — `main.rs` sets up the PostgreSQL connection pool, Redis cache, and Axum HTTP server.
2. **Mode Selection** — Starts either long polling or a webhook listener based on `BOT_MODE`.
3. **Dispatcher** — Receives Telegram updates and routes them through the handler tree.
4. **Command Handler** — Matches incoming commands to the appropriate module handler.
5. **Message Handler** — Evaluates messages against filters, blacklists, antiflood rules, and sed patterns.
6. **Callback Handler** — Processes inline keyboard interactions (help navigation, settings, undo actions).
7. **Chat Member Handler** — Triggers welcome/goodbye/CAPTCHA flows on member join and leave events.
8. **Permission Layer** — Gates every privileged action with owner / sudo / dev / chat-admin scoping.
9. **Persistence** — State is stored in PostgreSQL; frequently accessed data is cached in Redis with configurable TTL.
10. **Metrics Collection** — Prometheus counters and gauges are updated throughout the pipeline and served at `/metrics`.

---

## Contributing

Contributions are welcome. To get started:

1. **Fork** the repository and create a feature branch.
2. **Lint** your code: `cargo fmt && cargo clippy --all-targets -- -D warnings`.
3. **Test** your changes: `cargo test`.
4. **Submit** a pull request with a clear description of the changes.

For significant changes, please open an issue first to discuss the approach.

---

## Acknowledgements

This project draws inspiration from:

- [Alita_Robot](https://github.com/divkix/Alita_Robot) — Go
- [EmiliaHikari](https://github.com/AyraHikari/EmiliaHikari) — Python
- [tg_bot](https://github.com/SaitamaRobot/tgbot) — Python

---

## License

This project is licensed under the [MIT License](LICENSE).

Copyright &copy; 2024 Arumi
