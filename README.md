<div align="center">

# 🦀 Ferris Bot

**A powerful, modular Telegram group management bot written in Rust.**

Built on top of [Teloxide](https://github.com/teloxide/teloxide) v0.17, with async I/O powered by Tokio
and persistent storage via SQLite (`sqlx`).

[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)](https://www.rust-lang.org/)
[![Teloxide](https://img.shields.io/badge/Teloxide-0.17-blue)](https://github.com/teloxide/teloxide)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Docker](https://img.shields.io/badge/Docker-ready-2496ED?logo=docker)](./Dockerfile)

</div>

---

## Table of Contents

- [Highlights](#highlights)
- [Features](#features)
- [Modules](#modules)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [Requirements](#requirements)
- [Installation](#installation)
  - [From Source](#from-source)
  - [With Docker](#with-docker)
- [Configuration](#configuration)
- [Running the Bot](#running-the-bot)
- [Internationalization (i18n)](#internationalization-i18n)
- [Development](#development)
- [How It Works](#how-it-works)
- [Contributing](#contributing)
- [Credits](#credits)
- [License](#license)

---

## Highlights

- ⚡ **Fast & memory-safe** — written in idiomatic Rust with zero-cost async via Tokio.
- 🧩 **34+ feature modules** covering moderation, anti-abuse, customization, and federation.
- 🌐 **Internationalized** with [Fluent](https://projectfluent.org/) (English & Indonesian out of the box).
- 🗄️ **Persistent SQLite** storage with migrations managed by `sqlx`.
- 🎛️ **Inline keyboard UI** for help, settings, and admin actions.
- 🐳 **Docker & Compose** ready for production deployment.
- 🔒 **Permission-aware**: owner / sudo / dev / chat-admin scoping.

---

## Features

| Category            | Capabilities                                                                                          |
| ------------------- | ----------------------------------------------------------------------------------------------------- |
| **Moderation**      | Bans, kicks, mutes, warnings (with configurable limits & modes), purges, pins                         |
| **Anti-Abuse**      | Antiflood, antispam (rate limiting), word & sticker blacklists, captcha for new members               |
| **Customization**   | Welcome / goodbye messages, group rules, notes (`#hashtag` retrieval), keyword filters & reactions    |
| **Administration**  | Promote / demote, custom titles, admin list, chat permissions, locks (per message type)               |
| **Logging**         | Dedicated log channel for admin actions and important events                                          |
| **Federation**      | Cross-group federations with shared ban lists (`/fban`, `/fedinfo`, etc.)                             |
| **Global Bans**     | Bot-wide bans by sudo / dev users (`/gban`, `/ungban`)                                                |
| **User Tools**      | AFK status, bio / about, `/id`, `/info`, settings panel, language selection                          |
| **Connections**     | Manage groups privately from PM (`/connect`, `/disconnect`)                                           |
| **Reports**         | User-driven reporting system to alert admins                                                          |
| **Backups**         | Export and import chat configuration (`/export`, `/import`)                                           |
| **Cleaner**         | Automatically remove service messages and "blue text" join notices                                    |
| **Sed**             | Regex find & replace on replied messages (`s/pattern/replacement/flags`)                              |
| **Dev Tools**       | Sudo / dev management, broadcast, bot stats, chat list, leave-chat utilities                          |

---

## Modules

The bot is split into independent modules under [`src/modules`](./src/modules). Each module owns its
commands, state, and handlers.

| Module               | Commands                                                                                                                  |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| **admin**            | `/promote`, `/demote`, `/adminlist`, `/title`                                                                             |
| **bans**             | `/ban`, `/unban`, `/kick`, `/dban`, `/dkick`                                                                              |
| **mutes**            | `/mute`, `/unmute`, `/tmute`                                                                                              |
| **warns**            | `/warn`, `/warns`, `/resetwarns`, `/setwarnlimit`, `/setwarnmode`                                                         |
| **notes**            | `/save`, `/get`, `/notes`, `/clear`, `#notename`                                                                          |
| **filters**          | `/filter`, `/filters`, `/stop`, `/stopall`                                                                                |
| **welcome**          | `/welcome`, `/setwelcome`, `/goodbye`, `/setgoodbye`                                                                      |
| **rules**            | `/rules`, `/setrules`, `/clearrules`                                                                                      |
| **blacklist**        | `/blacklist`, `/addblacklist`, `/rmblacklist`, `/blacklistmode`                                                           |
| **blstickers**       | `/blsticker`, `/addblsticker`, `/rmblsticker`, `/blstickermode`                                                           |
| **purges**           | `/purge`, `/del`                                                                                                          |
| **pins**             | `/pin`, `/unpin`, `/unpinall`                                                                                             |
| **antiflood**        | `/setflood`, `/flood`, `/setfloodmode`                                                                                    |
| **antispam**         | *(automatic rate limiter)*                                                                                                |
| **disable**          | `/disable`, `/enable`, `/disabled`                                                                                        |
| **locks**            | `/lock`, `/unlock`, `/locks`, `/locktypes`                                                                                |
| **log_channel**      | `/logchannel`, `/setlogchannel`, `/unsetlogchannel`                                                                       |
| **reports**          | `/report`, `/reports`                                                                                                     |
| **gbans**            | `/gban`, `/ungban`, `/gbanlist`                                                                                           |
| **backups**          | `/export`, `/import`                                                                                                      |
| **connections**      | `/connect`, `/disconnect`, `/connection`                                                                                  |
| **afk**              | `/afk`                                                                                                                    |
| **chatpermissions**  | `/setpermissions`, `/permissions`                                                                                         |
| **captcha**          | `/captcha`, `/captchamode`, `/captchatime`, `/captchaaction`                                                              |
| **devs**             | `/addsudo`, `/remsudo`, `/adddev`, `/remdev`, `/teamusers`, `/chatinfo`, `/leavechat`, `/botstats`, `/broadcast`          |
| **feds**             | `/newfed`, `/delfed`, `/joinfed`, `/leavefed`, `/fedinfo`, `/fedpromote`, `/feddemote`, `/fban`, `/unfban`, `/fbanlist`, `/fedchat`, `/fedrules` |
| **userinfo**         | `/setme`, `/me`, `/setbio`, `/bio`                                                                                        |
| **cleaner**          | `/cleanservice`, `/cleanbluetext`                                                                                         |
| **reactions**        | `/addreaction`, `/removereaction`, `/reactions`, `/resetreactions`                                                        |
| **sed**              | `s/pattern/replacement/`                                                                                                  |
| **misc**             | `/id`, `/info`, `/settings`, `/setlang`, `/stats`, `/chatlist`                                                            |

> Use `/help` in chat to browse all modules through an interactive inline keyboard.

---

## Tech Stack

| Layer            | Choice                                                                  |
| ---------------- | ----------------------------------------------------------------------- |
| Language         | **Rust** (2021 edition)                                                 |
| Bot framework    | [`teloxide`](https://crates.io/crates/teloxide) `0.17`                  |
| Async runtime    | [`tokio`](https://crates.io/crates/tokio)                               |
| Database         | **SQLite** via [`sqlx`](https://crates.io/crates/sqlx) (compile-time checked) |
| Migrations       | `sqlx::migrate!` from [`migrations/`](./migrations)                     |
| Serialization    | `serde`, `serde_json`                                                   |
| i18n             | [`fluent-bundle`](https://crates.io/crates/fluent-bundle) + `unic-langid` |
| Misc             | `regex`, `chrono`, `parking_lot`, `once_cell`, `uuid`, `rand`, `anyhow`, `thiserror` |

---

## Project Structure

```text
Ferris/
├── Cargo.toml              # Crate manifest & dependencies
├── Dockerfile              # Multi-stage Docker build
├── docker-compose.yml      # Compose service definition
├── .env.example            # Example environment configuration
├── migrations/             # SQL migrations (sqlx)
│   └── 001_initial_schema.sql
├── locales/                # Fluent translations
│   ├── en/                 # English (29 .ftl files)
│   └── id/                 # Indonesian (29 .ftl files)
└── src/
    ├── main.rs             # Entry point & dispatcher
    ├── config.rs           # Env-driven configuration
    ├── db/
    │   ├── mod.rs          # Pool init & migrations
    │   ├── models.rs       # Data models
    │   └── queries.rs      # SQL queries
    ├── handlers/
    │   └── mod.rs          # Command / message / callback routing
    ├── keyboards/
    │   ├── mod.rs
    │   └── inline.rs       # Inline keyboard builders
    ├── modules/            # 34 feature modules (see table above)
    └── utils/
        ├── cache.rs        # TTL cache for hot DB reads
        ├── extraction.rs   # User & argument extraction
        ├── formatting.rs   # HTML / greeting templates
        ├── i18n.rs         # Fluent loader & translator
        └── permissions.rs  # Permission checks
```

---

## Requirements

- **Rust** ≥ 1.79 (stable toolchain)
- **SQLite** (bundled via `sqlx`, no system install required)
- A **Telegram bot token** from [@BotFather](https://t.me/BotFather)
- *(Optional)* **Docker** ≥ 24 and **Docker Compose** v2 for containerized deployment

---

## Installation

### From Source

```bash
# 1. Clone the repository
git clone https://github.com/<your-username>/Ferris.git
cd Ferris

# 2. Configure environment
cp .env.example .env
$EDITOR .env

# 3. Build a release binary
cargo build --release

# 4. Run
./target/release/ferris-bot
```

### With Docker

```bash
# 1. Configure environment
cp .env.example .env
$EDITOR .env

# 2. Build & start in the background
docker compose up -d --build

# 3. Tail logs
docker compose logs -f ferris-bot

# 4. Stop
docker compose down
```

The SQLite database is persisted to the host directory `./data` via the volume mount defined in
[`docker-compose.yml`](./docker-compose.yml).

---

## Configuration

All runtime configuration is read from environment variables (or a `.env` file at the repo root).

| Variable         | Required | Default              | Description                                             |
| ---------------- | :------: | -------------------- | ------------------------------------------------------- |
| `TELOXIDE_TOKEN` | ✅       | —                    | Bot token from @BotFather                               |
| `OWNER_ID`       | ✅       | —                    | Telegram user ID of the bot owner                       |
| `BOT_NAME`       | ❌       | `FerrisBot`          | Display name used in messages                           |
| `SUDO_USERS`     | ❌       | *(empty)*            | Comma-separated user IDs with sudo privileges           |
| `DATABASE_URL`   | ❌       | `sqlite://ferris.db` | SQLite connection string                                |
| `RUST_LOG`       | ❌       | `info`               | Log level: `trace`, `debug`, `info`, `warn`, `error`    |

Example `.env`:

```env
TELOXIDE_TOKEN=123456:ABC-DEF...
BOT_NAME=FerrisBot
OWNER_ID=123456789
SUDO_USERS=111111111,222222222
DATABASE_URL=sqlite://ferris.db
RUST_LOG=info
```

> 💡 To find your Telegram user ID, send `/id` to the bot in a private chat after it is running,
> or use a service like [@userinfobot](https://t.me/userinfobot).

---

## Running the Bot

After configuration, the bot connects to Telegram via long polling — no public webhook URL is
required. On first launch, the database file is created automatically and migrations from
[`migrations/`](./migrations) are applied.

```bash
# Development (debug build, hot logs)
RUST_LOG=debug cargo run

# Release
cargo run --release
```

Once running, open a chat with the bot and send `/start` to access the inline help menu.

---

## Internationalization (i18n)

Translations live in [`locales/`](./locales) as Fluent (`.ftl`) files. Each module typically owns
one file per language. Currently shipped:

- 🇬🇧 English (`locales/en/`)
- 🇮🇩 Indonesian (`locales/id/`)

Users can switch language per-chat with `/setlang`. To add a new language, create a new directory
under `locales/<lang-code>/` mirroring the structure of `locales/en/` and translate the strings.

---

## Development

```bash
# Type-check without building
cargo check

# Run with debug logs
RUST_LOG=debug cargo run

# Format & lint
cargo fmt
cargo clippy --all-targets -- -D warnings

# Build a release binary
cargo build --release
```

When modifying the database schema, add a new file under `migrations/` (e.g. `002_*.sql`) — it will
be applied automatically on next startup.

---

## How It Works

1. **Dispatcher** (`src/main.rs`) receives updates from Telegram via long polling.
2. **Command handler** (`src/handlers/mod.rs`) routes commands to the matching module.
3. **Message handler** evaluates each message against filters, blacklists, antiflood, sed, etc.
4. **Callback handler** processes inline keyboard interactions (help menus, settings, undo buttons).
5. **Chat-member handler** triggers welcome / goodbye / captcha flows on join & leave events.
6. **Permission utilities** gate every privileged action (owner / sudo / dev / chat admin).
7. **State** is persisted in SQLite; hot reads are cached with a small TTL cache.

---

## Contributing

Contributions are welcome! If you'd like to add a feature, fix a bug, or translate the bot:

1. Fork the repository and create a feature branch.
2. Run `cargo fmt` and `cargo clippy` before committing.
3. Open a pull request describing your changes.

For larger changes, please open an issue first to discuss the design.

---

## Credits

Inspired by the great work of:

- [Alita_Robot](https://github.com/divkix/Alita_Robot) (Go)
- [EmiliaHikari](https://github.com/AyraHikari/EmiliaHikari) (Python)
- [tg_bot](https://github.com/SaitamaRobot/tgbot) (Python)

---

## License

This project is licensed under the **MIT License** — see the [LICENSE](./LICENSE) file for details.
