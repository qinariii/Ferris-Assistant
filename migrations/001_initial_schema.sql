-- Initial schema: all tables from the original inline migrations

CREATE TABLE IF NOT EXISTS chats (
    chat_id INTEGER PRIMARY KEY,
    chat_name TEXT NOT NULL DEFAULT '',
    welcome_enabled INTEGER NOT NULL DEFAULT 1,
    welcome_text TEXT NOT NULL DEFAULT 'Hey {first_name}, welcome to {chat_name}!',
    goodbye_enabled INTEGER NOT NULL DEFAULT 1,
    goodbye_text TEXT NOT NULL DEFAULT 'Goodbye {first_name}!',
    rules TEXT NOT NULL DEFAULT '',
    warn_limit INTEGER NOT NULL DEFAULT 3,
    warn_mode TEXT NOT NULL DEFAULT 'ban',
    antiflood_count INTEGER NOT NULL DEFAULT 0,
    antiflood_mode TEXT NOT NULL DEFAULT 'mute',
    language TEXT NOT NULL DEFAULT 'en',
    log_channel INTEGER
);

CREATE TABLE IF NOT EXISTS users (
    user_id INTEGER PRIMARY KEY,
    username TEXT NOT NULL DEFAULT '',
    first_name TEXT NOT NULL DEFAULT '',
    last_name TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS warns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    reason TEXT NOT NULL DEFAULT '',
    warned_by INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (chat_id) REFERENCES chats(chat_id)
);

CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    content TEXT NOT NULL,
    media_type TEXT,
    media_id TEXT,
    FOREIGN KEY (chat_id) REFERENCES chats(chat_id),
    UNIQUE(chat_id, name)
);

CREATE TABLE IF NOT EXISTS filters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    keyword TEXT NOT NULL,
    reply_text TEXT NOT NULL DEFAULT '',
    media_type TEXT,
    media_id TEXT,
    FOREIGN KEY (chat_id) REFERENCES chats(chat_id),
    UNIQUE(chat_id, keyword)
);

CREATE TABLE IF NOT EXISTS blacklist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    trigger_word TEXT NOT NULL,
    mode TEXT NOT NULL DEFAULT 'delete',
    FOREIGN KEY (chat_id) REFERENCES chats(chat_id),
    UNIQUE(chat_id, trigger_word)
);

CREATE TABLE IF NOT EXISTS disabled_commands (
    chat_id INTEGER NOT NULL,
    command TEXT NOT NULL,
    PRIMARY KEY (chat_id, command)
);

CREATE TABLE IF NOT EXISTS locks (
    chat_id INTEGER NOT NULL,
    lock_type TEXT NOT NULL,
    PRIMARY KEY (chat_id, lock_type)
);

CREATE TABLE IF NOT EXISTS gbans (
    user_id INTEGER PRIMARY KEY,
    reason TEXT NOT NULL DEFAULT '',
    banned_by INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS connections (
    user_id INTEGER PRIMARY KEY,
    chat_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS report_settings (
    chat_id INTEGER PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS afk_users (
    user_id INTEGER PRIMARY KEY,
    reason TEXT NOT NULL DEFAULT '',
    is_afk INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS blacklist_stickers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    sticker_set TEXT NOT NULL,
    mode INTEGER NOT NULL DEFAULT 1,
    UNIQUE(chat_id, sticker_set)
);

CREATE TABLE IF NOT EXISTS blsticker_settings (
    chat_id INTEGER PRIMARY KEY,
    mode INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS user_chats (
    user_id INTEGER NOT NULL,
    chat_id INTEGER NOT NULL,
    PRIMARY KEY (user_id, chat_id)
);

CREATE TABLE IF NOT EXISTS captcha_settings (
    chat_id INTEGER PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,
    captcha_mode TEXT NOT NULL DEFAULT 'math',
    timeout_min INTEGER NOT NULL DEFAULT 5,
    failure_action TEXT NOT NULL DEFAULT 'kick',
    max_attempts INTEGER NOT NULL DEFAULT 3
);

CREATE TABLE IF NOT EXISTS captcha_attempts (
    user_id INTEGER NOT NULL,
    chat_id INTEGER NOT NULL,
    answer TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    message_id INTEGER NOT NULL DEFAULT 0,
    expires_at TEXT NOT NULL,
    PRIMARY KEY (user_id, chat_id)
);

CREATE TABLE IF NOT EXISTS dev_team (
    user_id INTEGER PRIMARY KEY,
    role TEXT NOT NULL DEFAULT 'sudo'
);

CREATE TABLE IF NOT EXISTS federations (
    fed_id TEXT PRIMARY KEY,
    fed_name TEXT NOT NULL,
    owner_id INTEGER NOT NULL,
    fed_rules TEXT NOT NULL DEFAULT '',
    fed_log INTEGER
);

CREATE TABLE IF NOT EXISTS fed_chats (
    chat_id INTEGER PRIMARY KEY,
    fed_id TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS fed_admins (
    fed_id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    PRIMARY KEY (fed_id, user_id)
);

CREATE TABLE IF NOT EXISTS fed_bans (
    fed_id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    user_name TEXT NOT NULL DEFAULT '',
    reason TEXT NOT NULL DEFAULT '',
    banned_at INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (fed_id, user_id)
);

CREATE TABLE IF NOT EXISTS user_info (
    user_id INTEGER PRIMARY KEY,
    bio TEXT NOT NULL DEFAULT '',
    me_info TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS cleaner_settings (
    chat_id INTEGER PRIMARY KEY,
    clean_service INTEGER NOT NULL DEFAULT 0,
    clean_bluetext INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS reactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
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
