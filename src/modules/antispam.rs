use std::collections::HashMap;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use teloxide::prelude::*;

use crate::config::AppConfig;
use crate::utils::formatting::uid_to_i64;
use crate::utils::permissions;

const SPAM_LIMIT: u32 = 18;
const SPAM_WINDOW: Duration = Duration::from_secs(1);
const CLEANUP_THRESHOLD: usize = 5000;

#[derive(Clone)]
struct SpamEntry {
    count: u32,
    window_start: Instant,
}

static SPAM_MAP: Lazy<Mutex<HashMap<(i64, i64), SpamEntry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Check if a user is spamming. Returns true if rate-limited.
fn is_spamming(chat_id: i64, user_id: i64) -> bool {
    let mut map = SPAM_MAP.lock();

    // Periodic cleanup
    if map.len() > CLEANUP_THRESHOLD {
        let now = Instant::now();
        map.retain(|_, entry| now.duration_since(entry.window_start) < SPAM_WINDOW * 2);
    }

    let key = (chat_id, user_id);
    let now = Instant::now();

    let entry = map.entry(key).or_insert(SpamEntry {
        count: 0,
        window_start: now,
    });

    // Reset window if expired
    if now.duration_since(entry.window_start) >= SPAM_WINDOW {
        entry.count = 0;
        entry.window_start = now;
    }

    entry.count += 1;
    entry.count > SPAM_LIMIT
}

/// Check message against antispam. Should be called early in the message pipeline.
/// Returns true if the message should be ignored (user is spamming).
pub async fn check_antispam(
    bot: Bot,
    msg: Message,
    cfg: AppConfig,
) -> ResponseResult<bool> {
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(false),
    };

    // Skip admin/owner/sudo users
    if cfg.is_sudo(uid_to_i64(from.id))
        || permissions::is_user_admin(&bot, msg.chat.id, from.id).await
    {
        return Ok(false);
    }

    let chat_id = msg.chat.id.0;
    let user_id = uid_to_i64(from.id);

    if is_spamming(chat_id, user_id) {
        log::debug!(
            "[Antispam] Rate limited user={} chat={}",
            user_id,
            chat_id
        );
        return Ok(true);
    }

    Ok(false)
}
