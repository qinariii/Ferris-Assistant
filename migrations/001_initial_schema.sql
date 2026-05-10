-- Initial schema: PostgreSQL

CREATE TABLE IF NOT EXISTS chats (
    chat_id BIGINT PRIMARY KEY,
    chat_name TEXT NOT NULL DEFAULT '',
    welcome_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    welcome_text TEXT NOT NULL DEFAULT 'Hey {first_name}, welcome to {chat_name}!',
    goodbye_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    goodbye_text TEXT NOT NULL DEFAULT 'Goodbye {first_name}!',
    rules TEXT NOT NULL DEFAULT '',
    warn_limit INT NOT NULL DEFAULT 3,
    warn_mode TEXT NOT NULL DEFAULT 'ban',
    antiflood_count INT NOT NULL DEFAULT 0,
    antiflood_mode TEXT NOT NULL DEFAULT 'mute',
    language TEXT NOT NULL DEFAULT 'en',
    log_channel BIGINT
);

CREATE TABLE IF NOT EXISTS users (
    user_id BIGINT PRIMARY KEY,
    username TEXT NOT NULL DEFAULT '',
    first_name TEXT NOT NULL DEFAULT '',
    last_name TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS warns (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(chat_id),
    user_id BIGINT NOT NULL,
    reason TEXT NOT NULL DEFAULT '',
    warned_by BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS notes (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(chat_id),
    name TEXT NOT NULL,
    content TEXT NOT NULL,
    media_type TEXT,
    media_id TEXT,
    UNIQUE(chat_id, name)
);

CREATE TABLE IF NOT EXISTS filters (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(chat_id),
    keyword TEXT NOT NULL,
    reply_text TEXT NOT NULL DEFAULT '',
    media_type TEXT,
    media_id TEXT,
    UNIQUE(chat_id, keyword)
);

CREATE TABLE IF NOT EXISTS blacklist (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(chat_id),
    trigger_word TEXT NOT NULL,
    mode TEXT NOT NULL DEFAULT 'delete',
    UNIQUE(chat_id, trigger_word)
);

CREATE TABLE IF NOT EXISTS disabled_commands (
    chat_id BIGINT NOT NULL,
    command TEXT NOT NULL,
    PRIMARY KEY (chat_id, command)
);

CREATE TABLE IF NOT EXISTS locks (
    chat_id BIGINT NOT NULL,
    lock_type TEXT NOT NULL,
    PRIMARY KEY (chat_id, lock_type)
);

CREATE TABLE IF NOT EXISTS gbans (
    user_id BIGINT PRIMARY KEY,
    reason TEXT NOT NULL DEFAULT '',
    banned_by BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS connections (
    user_id BIGINT PRIMARY KEY,
    chat_id BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS report_settings (
    chat_id BIGINT PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS afk_users (
    user_id BIGINT PRIMARY KEY,
    reason TEXT NOT NULL DEFAULT '',
    is_afk BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS blacklist_stickers (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    sticker_set TEXT NOT NULL,
    mode INT NOT NULL DEFAULT 1,
    UNIQUE(chat_id, sticker_set)
);

CREATE TABLE IF NOT EXISTS blsticker_settings (
    chat_id BIGINT PRIMARY KEY,
    mode INT NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS user_chats (
    user_id BIGINT NOT NULL,
    chat_id BIGINT NOT NULL,
    PRIMARY KEY (user_id, chat_id)
);

CREATE TABLE IF NOT EXISTS captcha_settings (
    chat_id BIGINT PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    captcha_mode TEXT NOT NULL DEFAULT 'math',
    timeout_min BIGINT NOT NULL DEFAULT 5,
    failure_action TEXT NOT NULL DEFAULT 'kick',
    max_attempts BIGINT NOT NULL DEFAULT 3
);

CREATE TABLE IF NOT EXISTS captcha_attempts (
    user_id BIGINT NOT NULL,
    chat_id BIGINT NOT NULL,
    answer TEXT NOT NULL,
    attempts BIGINT NOT NULL DEFAULT 0,
    message_id BIGINT NOT NULL DEFAULT 0,
    expires_at TEXT NOT NULL,
    PRIMARY KEY (user_id, chat_id)
);

CREATE TABLE IF NOT EXISTS dev_team (
    user_id BIGINT PRIMARY KEY,
    role TEXT NOT NULL DEFAULT 'sudo'
);

CREATE TABLE IF NOT EXISTS federations (
    fed_id TEXT PRIMARY KEY,
    fed_name TEXT NOT NULL,
    owner_id BIGINT NOT NULL,
    fed_rules TEXT NOT NULL DEFAULT '',
    fed_log BIGINT
);

CREATE TABLE IF NOT EXISTS fed_chats (
    chat_id BIGINT PRIMARY KEY,
    fed_id TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS fed_admins (
    fed_id TEXT NOT NULL,
    user_id BIGINT NOT NULL,
    PRIMARY KEY (fed_id, user_id)
);

CREATE TABLE IF NOT EXISTS fed_bans (
    fed_id TEXT NOT NULL,
    user_id BIGINT NOT NULL,
    user_name TEXT NOT NULL DEFAULT '',
    reason TEXT NOT NULL DEFAULT '',
    banned_at BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (fed_id, user_id)
);

CREATE TABLE IF NOT EXISTS user_info (
    user_id BIGINT PRIMARY KEY,
    bio TEXT NOT NULL DEFAULT '',
    me_info TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS cleaner_settings (
    chat_id BIGINT PRIMARY KEY,
    clean_service BOOLEAN NOT NULL DEFAULT FALSE,
    clean_bluetext BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS reactions (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    keyword TEXT NOT NULL,
    emoji TEXT NOT NULL,
    UNIQUE(chat_id, keyword)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_warns_chat_user ON warns(chat_id, user_id);
CREATE INDEX IF NOT EXISTS idx_notes_chat ON notes(chat_id);
CREATE INDEX IF NOT EXISTS idx_filters_chat ON filters(chat_id);
CREATE INDEX IF NOT EXISTS idx_blacklist_chat ON blacklist(chat_id);
CREATE INDEX IF NOT EXISTS idx_locks_chat ON locks(chat_id);
CREATE INDEX IF NOT EXISTS idx_fed_bans_fed ON fed_bans(fed_id);
CREATE INDEX IF NOT EXISTS idx_fed_bans_user ON fed_bans(user_id);
CREATE INDEX IF NOT EXISTS idx_user_chats_user ON user_chats(user_id);
CREATE INDEX IF NOT EXISTS idx_blacklist_stickers_chat ON blacklist_stickers(chat_id);
