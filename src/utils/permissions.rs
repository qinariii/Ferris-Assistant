use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use teloxide::prelude::*;
use teloxide::types::{ChatMember, ChatMemberKind};

use crate::config::AppConfig;
use crate::utils::cache::TtlCache;
use crate::utils::formatting::uid_to_i64;


/// Cached bot user ID to avoid repeated `get_me()` API calls.
static BOT_USER_ID: once_cell::sync::OnceCell<UserId> = once_cell::sync::OnceCell::new();

/// Set the bot's own UserId. Call once at startup after `bot.get_me()`.
pub fn set_bot_id(id: UserId) {
    BOT_USER_ID.set(id).ok();
}

/// Resolve the bot's UserId — cached first, fallback to `get_me()` once.
async fn resolve_bot_id(bot: &Bot) -> Option<UserId> {
    if let Some(&id) = BOT_USER_ID.get() {
        return Some(id);
    }
    let me = bot.get_me().await.ok()?;
    BOT_USER_ID.set(me.id).ok();
    Some(me.id)
}


/// Lightweight snapshot of a user's permissions in a chat.
/// Extracted once from a `get_chat_member` API call and cached.
#[derive(Clone, Debug)]
struct CachedPerms {
    is_admin: bool,
    is_owner: bool,
    can_restrict: bool,
    can_delete: bool,
    can_pin: bool,
    can_promote: bool,
}

impl CachedPerms {
    fn from_member(member: &ChatMember) -> Self {
        match &member.kind {
            ChatMemberKind::Owner(_) => Self {
                is_admin: true,
                is_owner: true,
                can_restrict: true,
                can_delete: true,
                can_pin: true,
                can_promote: true,
            },
            ChatMemberKind::Administrator(admin) => Self {
                is_admin: true,
                is_owner: false,
                can_restrict: admin.can_restrict_members,
                can_delete: admin.can_delete_messages,
                can_pin: admin.can_pin_messages,
                can_promote: admin.can_promote_members,
            },
            _ => Self {
                is_admin: false,
                is_owner: false,
                can_restrict: false,
                can_delete: false,
                can_pin: false,
                can_promote: false,
            },
        }
    }

    fn non_admin() -> Self {
        Self {
            is_admin: false,
            is_owner: false,
            can_restrict: false,
            can_delete: false,
            can_pin: false,
            can_promote: false,
        }
    }
}

/// Cache admin permissions for 120 seconds.
/// Key: (chat_id, user_id) — both stored as i64 for uniformity.
static PERMS_CACHE: Lazy<Arc<TtlCache<(i64, u64), CachedPerms>>> =
    Lazy::new(|| TtlCache::new(Duration::from_secs(120)));

/// Resolve (and cache) permissions for a user in a chat.
async fn resolve_perms(bot: &Bot, chat_id: ChatId, user_id: UserId) -> CachedPerms {
    let key = (chat_id.0, user_id.0);

    if let Some(cached) = PERMS_CACHE.get(&key) {
        return cached;
    }

    let perms = match bot.get_chat_member(chat_id, user_id).await {
        Ok(member) => CachedPerms::from_member(&member),
        Err(_) => CachedPerms::non_admin(),
    };

    PERMS_CACHE.set(key, perms.clone());
    perms
}

/// Invalidate cached permissions for an entire chat.
/// Call after /promote, /demote, or any admin change.
#[allow(dead_code)]
pub fn invalidate_chat_perms(chat_id: ChatId) {
    let cid = chat_id.0;
    PERMS_CACHE.invalidate_by(|k| k.0 == cid);
}

/// Invalidate cached permissions for a specific user in a chat.
#[allow(dead_code)]
pub fn invalidate_user_perms(chat_id: ChatId, user_id: UserId) {
    PERMS_CACHE.invalidate(&(chat_id.0, user_id.0));
}


#[allow(dead_code)]
pub async fn is_bot_admin(bot: &Bot, chat_id: ChatId, bot_id: UserId) -> bool {
    resolve_perms(bot, chat_id, bot_id).await.is_admin
}

pub async fn is_user_admin(bot: &Bot, chat_id: ChatId, user_id: UserId) -> bool {
    resolve_perms(bot, chat_id, user_id).await.is_admin
}

pub async fn is_user_owner(bot: &Bot, chat_id: ChatId, user_id: UserId) -> bool {
    resolve_perms(bot, chat_id, user_id).await.is_owner
}

pub async fn can_user_restrict(bot: &Bot, chat_id: ChatId, user_id: UserId) -> bool {
    resolve_perms(bot, chat_id, user_id).await.can_restrict
}

pub async fn can_user_delete(bot: &Bot, chat_id: ChatId, user_id: UserId) -> bool {
    resolve_perms(bot, chat_id, user_id).await.can_delete
}

pub async fn can_user_pin(bot: &Bot, chat_id: ChatId, user_id: UserId) -> bool {
    resolve_perms(bot, chat_id, user_id).await.can_pin
}

pub async fn can_user_promote(bot: &Bot, chat_id: ChatId, user_id: UserId) -> bool {
    resolve_perms(bot, chat_id, user_id).await.can_promote
}

pub async fn can_bot_restrict(bot: &Bot, chat_id: ChatId) -> bool {
    let bot_id = match resolve_bot_id(bot).await {
        Some(id) => id,
        None => return false,
    };
    can_user_restrict(bot, chat_id, bot_id).await
}

pub async fn can_bot_delete(bot: &Bot, chat_id: ChatId) -> bool {
    let bot_id = match resolve_bot_id(bot).await {
        Some(id) => id,
        None => return false,
    };
    can_user_delete(bot, chat_id, bot_id).await
}

pub async fn can_bot_pin(bot: &Bot, chat_id: ChatId) -> bool {
    let bot_id = match resolve_bot_id(bot).await {
        Some(id) => id,
        None => return false,
    };
    can_user_pin(bot, chat_id, bot_id).await
}

pub async fn is_user_ban_protected(
    bot: &Bot,
    chat_id: ChatId,
    user_id: UserId,
    cfg: &AppConfig,
) -> bool {
    if cfg.is_sudo(uid_to_i64(user_id)) {
        return true;
    }
    is_user_admin(bot, chat_id, user_id).await
}

#[allow(dead_code)]
pub async fn get_chat_member(bot: &Bot, chat_id: ChatId, user_id: UserId) -> Option<ChatMember> {
    bot.get_chat_member(chat_id, user_id).await.ok()
}