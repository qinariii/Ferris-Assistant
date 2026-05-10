use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Chat {
    pub chat_id: i64,
    pub chat_name: String,
    pub welcome_enabled: bool,
    pub welcome_text: String,
    pub goodbye_enabled: bool,
    pub goodbye_text: String,
    pub rules: String,
    pub warn_limit: i32,
    pub warn_mode: String,
    pub antiflood_count: i32,
    pub antiflood_mode: String,
    pub language: String,
    pub log_channel: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub user_id: i64,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Warn {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: i64,
    pub reason: String,
    pub warned_by: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    pub chat_id: i64,
    pub name: String,
    pub content: String,
    pub media_type: Option<String>,
    pub media_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Filter {
    pub id: i64,
    pub chat_id: i64,
    pub keyword: String,
    pub reply_text: String,
    pub media_type: Option<String>,
    pub media_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Blacklist {
    pub id: i64,
    pub chat_id: i64,
    pub trigger_word: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DisabledCommand {
    pub chat_id: i64,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Lock {
    pub chat_id: i64,
    pub lock_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Gban {
    pub user_id: i64,
    pub reason: String,
    pub banned_by: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Connection {
    pub user_id: i64,
    pub chat_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReportSetting {
    pub chat_id: i64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AfkUser {
    pub user_id: i64,
    pub reason: String,
    pub is_afk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlacklistSticker {
    pub id: i64,
    pub chat_id: i64,
    pub sticker_set: String,
    pub mode: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlstickerSetting {
    pub chat_id: i64,
    pub mode: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserChat {
    pub user_id: i64,
    pub chat_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CaptchaSettings {
    pub chat_id: i64,
    pub enabled: bool,
    pub captcha_mode: String,
    pub timeout_min: i64,
    pub failure_action: String,
    pub max_attempts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CaptchaAttempt {
    pub user_id: i64,
    pub chat_id: i64,
    pub answer: String,
    pub attempts: i64,
    pub message_id: i64,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DevTeamMember {
    pub user_id: i64,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Federation {
    pub fed_id: String,
    pub fed_name: String,
    pub owner_id: i64,
    pub fed_rules: String,
    pub fed_log: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FedChat {
    pub chat_id: i64,
    pub fed_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FedAdmin {
    pub fed_id: String,
    pub user_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FedBan {
    pub fed_id: String,
    pub user_id: i64,
    pub user_name: String,
    pub reason: String,
    pub banned_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserInfoRow {
    pub user_id: i64,
    pub bio: String,
    pub me_info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CleanerSettings {
    pub chat_id: i64,
    pub clean_service: bool,
    pub clean_bluetext: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Reaction {
    pub id: i64,
    pub chat_id: i64,
    pub keyword: String,
    pub emoji: String,
}
