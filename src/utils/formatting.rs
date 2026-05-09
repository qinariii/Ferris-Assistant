use teloxide::types::{User, UserId};

/// Safely convert a UserId (u64) to i64 for database storage.
/// Telegram user IDs currently fit in i64, but this logs a warning if overflow ever occurs.
#[inline]
pub fn uid_to_i64(id: UserId) -> i64 {
    if id.0 > i64::MAX as u64 {
        log::warn!("UserId {} overflows i64! Stored value will be incorrect.", id.0);
    }
    id.0 as i64
}

/// Safely convert a u64 to i64 for database storage.
#[inline]
pub fn u64_to_i64(val: u64) -> i64 {
    if val > i64::MAX as u64 {
        log::warn!("u64 value {} overflows i64! Stored value will be incorrect.", val);
    }
    val as i64
}

/// Format user mention as HTML link
pub fn mention_html(user: &User) -> String {
    let name = html_escape(&user.first_name);
    format!("<a href=\"tg://user?id={}\">{}</a>", user.id, name)
}

/// Format user display name
pub fn user_display_name(user: &User) -> String {
    if let Some(ref username) = user.username {
        format!("@{}", username)
    } else {
        mention_html(user)
    }
}

/// Escape HTML special characters
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Parse duration string like "30s", "5m", "1h", "1d" into seconds.
/// Returns None if the format is invalid.
pub fn parse_duration(s: &str) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    let s = s.trim().to_lowercase();
    let (num_str, multiplier) = if s.ends_with('s') {
        (&s[..s.len() - 1], 1u64)
    } else if s.ends_with('m') {
        (&s[..s.len() - 1], 60u64)
    } else if s.ends_with('h') {
        (&s[..s.len() - 1], 3600u64)
    } else if s.ends_with('d') {
        (&s[..s.len() - 1], 86400u64)
    } else {
        return None;
    };

    num_str.parse::<u64>().ok().map(|n| n * multiplier)
}

/// Format welcome/goodbye text with placeholders
pub fn format_greeting(text: &str, user: &User, chat_title: &str) -> String {
    let first_name = html_escape(&user.first_name);
    let last_name = user
        .last_name
        .as_deref()
        .map(html_escape)
        .unwrap_or_default();
    let full_name = if last_name.is_empty() {
        first_name.clone()
    } else {
        format!("{} {}", first_name, last_name)
    };
    let username = user
        .username
        .as_deref()
        .map(|u| format!("@{}", u))
        .unwrap_or_else(|| mention_html(user));
    let mention = mention_html(user);
    let user_id = user.id.to_string();

    text.replace("{first_name}", &first_name)
        .replace("{last_name}", &last_name)
        .replace("{full_name}", &full_name)
        .replace("{username}", &username)
        .replace("{mention}", &mention)
        .replace("{id}", &user_id)
        .replace("{chat_name}", &html_escape(chat_title))
}
