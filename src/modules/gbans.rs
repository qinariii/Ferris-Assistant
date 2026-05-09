use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::config::AppConfig;
use crate::db;
use crate::utils::{extraction, formatting, formatting::uid_to_i64, i18n, permissions};

/// Global gban set cache: all gbanned user IDs, refreshed every 60s.
static GBAN_CACHE: once_cell::sync::Lazy<Arc<GbanCache>> =
    once_cell::sync::Lazy::new(|| Arc::new(GbanCache::new()));

struct GbanCache {
    inner: Mutex<(HashSet<i64>, Instant)>,
}

impl GbanCache {
    fn new() -> Self {
        Self {
            inner: Mutex::new((HashSet::new(), Instant::now() - Duration::from_secs(120))),
        }
    }

    fn is_gbanned(&self, user_id: i64) -> Option<bool> {
        let guard = self.inner.lock();
        if guard.1.elapsed() < Duration::from_secs(60) {
            Some(guard.0.contains(&user_id))
        } else {
            None
        }
    }

    fn refresh(&self, ids: HashSet<i64>) {
        let mut guard = self.inner.lock();
        *guard = (ids, Instant::now());
    }

    fn invalidate(&self) {
        let mut guard = self.inner.lock();
        guard.1 = Instant::now() - Duration::from_secs(120);
    }
}

pub async fn gban(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    // Only owner/sudo can gban
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    if !cfg.is_sudo(uid_to_i64(from.id)) {
        bot.send_message(chat_id, i18n::t(&lang, "gban-no-permission", None))
            .await?;
        return Ok(());
    }

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    let (target_user, reason) = extraction::extract_user_and_reason(&bot, &msg, &args, &pool).await;

    let target_id = match target_user {
        Some(id) => id,
        None => {
            bot.send_message(chat_id, i18n::t(&lang, "gban-no-user", None))
                .await?;
            return Ok(());
        }
    };

    // Can't gban owner/sudo
    if cfg.is_sudo(uid_to_i64(target_id)) {
        bot.send_message(chat_id, i18n::t(&lang, "gban-sudo-cant", None))
            .await?;
        return Ok(());
    }

    let reason_str = reason.as_deref().unwrap_or("No reason given");

    // Check if already gbanned
    if db::queries::is_gbanned(&pool, uid_to_i64(target_id)).await.unwrap_or(false) {
        bot.send_message(chat_id, i18n::t(&lang, "gban-already", None))
            .await?;
        return Ok(());
    }

    // Add to gban list
    db::queries::gban_user(&pool, uid_to_i64(target_id), reason_str, uid_to_i64(from.id))
        .await
        .ok();
    GBAN_CACHE.invalidate();

    let status_msg = bot
        .send_message(chat_id, "🌐 Starting global ban...")
        .await?;

    // Ban in all known chats
    let all_chats = db::queries::get_all_chats(&pool).await.unwrap_or_default();
    let mut success_count = 0;
    let mut fail_count = 0;

    for chat in &all_chats {
        match bot.ban_chat_member(ChatId(chat.chat_id), target_id).await {
            Ok(_) => success_count += 1,
            Err(_) => fail_count += 1,
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    let member = bot.get_chat_member(chat_id, target_id).await.ok();
    let name = member
        .map(|m| m.user.first_name.clone())
        .unwrap_or_else(|| target_id.to_string());
    let escaped_name = formatting::html_escape(&name);
    let admin_name = formatting::mention_html(from);
    let success_str = success_count.to_string();
    let fail_str = fail_count.to_string();

    let mut text = i18n::t(&lang, "gban-success", Some(&[("name", &escaped_name)]));
    text.push_str(&format!("\n{}", i18n::t(&lang, "gban-reason", Some(&[("reason", &formatting::html_escape(reason_str))]))));
    text.push_str(&format!("\n{}", i18n::t(&lang, "gban-by", Some(&[("admin", &admin_name)]))));
    text.push_str(&format!("\n{}", i18n::t(&lang, "gban-stats", Some(&[("success", &success_str), ("fail", &fail_str)]))));

    bot.edit_message_text(chat_id, status_msg.id, &text)
        .parse_mode(ParseMode::Html)
        .await?;

    // Log
    crate::modules::log_channel::send_log(
        &bot,
        &pool,
        chat_id.0,
        &format!(
            "📋 <b>#GBAN</b>\n\n<b>User:</b> {} [<code>{}</code>]\n<b>Reason:</b> {}\n<b>By:</b> {}",
            formatting::html_escape(&name),
            target_id.0,
            formatting::html_escape(reason_str),
            formatting::html_escape(&formatting::user_display_name(from)),
        ),
    )
    .await;

    Ok(())
}

pub async fn ungban(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !cfg.is_sudo(uid_to_i64(from.id)) {
        bot.send_message(chat_id, i18n::t(&lang, "gban-no-permission", None))
            .await?;
        return Ok(());
    }

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    let (target_user, _) = extraction::extract_user_and_reason(&bot, &msg, &args, &pool).await;

    let target_id = match target_user {
        Some(id) => id,
        None => {
            bot.send_message(chat_id, i18n::t(&lang, "gban-no-user", None))
                .await?;
            return Ok(());
        }
    };

    match db::queries::ungban_user(&pool, uid_to_i64(target_id)).await {
        Ok(true) => {
            GBAN_CACHE.invalidate();
            let status_msg = bot
                .send_message(chat_id, "🌐 Removing global ban...")
                .await?;

            // Unban in all known chats
            let all_chats = db::queries::get_all_chats(&pool).await.unwrap_or_default();
            let mut success_count = 0;

            for chat in &all_chats {
                if bot
                    .unban_chat_member(ChatId(chat.chat_id), target_id)
                    .await
                    .is_ok()
                {
                    success_count += 1;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }

            let count_str = success_count.to_string();
            bot.edit_message_text(
                chat_id,
                status_msg.id,
                i18n::t(&lang, "ungban-success", Some(&[("count", &count_str)])),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {
            bot.send_message(chat_id, i18n::t(&lang, "ungban-not-banned", None))
                .await?;
        }
    }
    Ok(())
}

pub async fn gbanlist(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    if !cfg.is_sudo(uid_to_i64(from.id)) {
        bot.send_message(chat_id, i18n::t(&lang, "gban-no-permission", None))
            .await?;
        return Ok(());
    }

    let gbans = db::queries::get_all_gbans(&pool).await.unwrap_or_default();

    if gbans.is_empty() {
        bot.send_message(chat_id, i18n::t(&lang, "gbanlist-empty", None))
            .await?;
        return Ok(());
    }

    let count_str = gbans.len().to_string();
    let mut text = format!("{}\n", i18n::t(&lang, "gbanlist-header", Some(&[("count", &count_str)])));
    for (i, gban) in gbans.iter().enumerate().take(50) {
        text.push_str(&format!(
            "\n{}. <code>{}</code> — {}",
            i + 1,
            gban.user_id,
            formatting::html_escape(&gban.reason),
        ));
    }

    if gbans.len() > 50 {
        text.push_str(&format!("\n\n<i>...and {} more</i>", gbans.len() - 50));
    }

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

/// Check new members against gban list
pub async fn check_gban(
    bot: Bot,
    msg: Message,
    pool: db::Pool,
) -> ResponseResult<()> {
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let chat_id = msg.chat.id;

    let is_banned = match GBAN_CACHE.is_gbanned(uid_to_i64(from.id)) {
        Some(b) => b,
        None => {
            // Refresh cache
            let all = db::queries::get_all_gbans(&pool).await.unwrap_or_default();
            let set: HashSet<i64> = all.iter().map(|g| g.user_id).collect();
            let result = set.contains(&uid_to_i64(from.id));
            GBAN_CACHE.refresh(set);
            result
        }
    };

    if is_banned {
        // Don't gban admins
        if !permissions::is_user_admin(&bot, chat_id, from.id).await {
            bot.ban_chat_member(chat_id, from.id).await.ok();
            let gban_info = db::queries::get_gban(&pool, uid_to_i64(from.id))
                .await
                .ok()
                .flatten();
            let reason = gban_info
                .map(|g| g.reason)
                .unwrap_or_else(|| "No reason".to_string());

            let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
            let name = formatting::mention_html(from);
            bot.send_message(
                chat_id,
                i18n::t(&lang, "gban-enforced", Some(&[("name", &name), ("reason", &formatting::html_escape(&reason))])),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }

    Ok(())
}
