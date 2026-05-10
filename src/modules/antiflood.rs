use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use once_cell::sync::Lazy;
use teloxide::prelude::*;
use teloxide::types::{ChatPermissions, ParseMode};

use crate::config::AppConfig;
use crate::db;
use crate::utils::{cache::TtlCache, formatting, i18n, kick::safe_kick, permissions, LogErrExt};

// flood settings cache: chat_id -> (limit, mode), refreshed every 30s
static FLOOD_SETTINGS_CACHE: Lazy<Arc<TtlCache<i64, (i32, String)>>> =
    Lazy::new(|| TtlCache::new(Duration::from_secs(30)));

pub type FloodTracker = Arc<Mutex<HashMap<(i64, u64), (u32, std::time::Instant)>>>;

pub fn new_flood_tracker() -> FloodTracker {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn spawn_flood_tracker_cleanup(
    tracker: FloodTracker,
    interval_secs: u64,
    expire_secs: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            let now = std::time::Instant::now();
            let mut map = tracker.lock().await;
            let before = map.len();
            map.retain(|_, entry| now.duration_since(entry.1).as_secs() <= expire_secs);
            let removed = before - map.len();
            if removed > 0 {
                log::debug!(
                    "flood_tracker cleanup: removed {} expired entries, {} remaining",
                    removed,
                    map.len()
                );
            }
        }
    })
}

pub async fn set_flood(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "antiflood-no-permission", None))
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

    if args.is_empty() {
        bot.send_message(chat_id, i18n::t(&lang, "antiflood-set-usage", None))
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("antiflood::upsert_chat");

    match args[0].to_lowercase().as_str() {
        "off" | "0" | "no" => {
            db::queries::set_antiflood(&pool, chat_id.0, 0).await.log_err("antiflood::set");
            FLOOD_SETTINGS_CACHE.invalidate(&chat_id.0);
            bot.send_message(chat_id, i18n::t(&lang, "antiflood-disabled", None))
                .await?;
        }
        _ => {
            let count: i32 = match args[0].parse() {
                Ok(n) if (3..=100).contains(&n) => n,
                _ => {
                    bot.send_message(chat_id, i18n::t(&lang, "antiflood-set-invalid", None))
                        .await?;
                    return Ok(());
                }
            };
            db::queries::set_antiflood(&pool, chat_id.0, count).await.log_err("antiflood::set");
            FLOOD_SETTINGS_CACHE.invalidate(&chat_id.0);
            let count_str = count.to_string();
            bot.send_message(
                chat_id,
                i18n::t(&lang, "antiflood-set", Some(&[("count", &count_str)])),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }
    Ok(())
}

pub async fn flood_info(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    let chat_data = db::queries::get_chat(&pool, chat_id.0).await.ok().flatten();

    match chat_data {
        Some(chat) if chat.antiflood_count > 0 => {
            let count_str = chat.antiflood_count.to_string();
            bot.send_message(
                chat_id,
                i18n::t(&lang, "antiflood-info-on", Some(&[("count", &count_str), ("mode", &chat.antiflood_mode)])),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {
            bot.send_message(chat_id, i18n::t(&lang, "antiflood-info-off", None))
                .await?;
        }
    }
    Ok(())
}

pub async fn set_flood_mode(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "antiflood-no-permission", None))
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

    if args.is_empty() {
        bot.send_message(chat_id, i18n::t(&lang, "antiflood-mode-usage", None))
            .await?;
        return Ok(());
    }

    let mode = args[0].to_lowercase();
    if !["ban", "kick", "mute"].contains(&mode.as_str()) {
        bot.send_message(chat_id, i18n::t(&lang, "antiflood-mode-invalid", None))
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("antiflood::upsert_chat");
    db::queries::set_antiflood_mode(&pool, chat_id.0, &mode).await.log_err("antiflood::set_mode");
    FLOOD_SETTINGS_CACHE.invalidate(&chat_id.0);

    bot.send_message(
        chat_id,
        i18n::t(&lang, "antiflood-mode-set", Some(&[("mode", &mode)])),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

/// Check messages for flood detection
pub async fn check_flood(
    bot: Bot,
    msg: Message,
    _cfg: AppConfig,
    pool: db::Pool,
    tracker: FloodTracker,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    // Don't check admins
    if permissions::is_user_admin(&bot, chat_id, from.id).await {
        return Ok(());
    }

    let key = (chat_id.0, from.id.0);
    let now = std::time::Instant::now();

    let (flood_limit, flood_mode) = if let Some(cached) = FLOOD_SETTINGS_CACHE.get(&chat_id.0) {
        cached
    } else {
        let chat_data = db::queries::get_chat(&pool, chat_id.0).await.ok().flatten();
        let limit = chat_data.as_ref().map(|c| c.antiflood_count).unwrap_or(0);
        let mode = chat_data
            .as_ref()
            .map(|c| c.antiflood_mode.clone())
            .unwrap_or_else(|| "mute".to_string());
        FLOOD_SETTINGS_CACHE.set(chat_id.0, (limit, mode.clone()));
        (limit, mode)
    };

    if flood_limit == 0 {
        return Ok(());
    }

    let should_action;

    {
        let mut map = tracker.lock().await;

        let entry = map.entry(key).or_insert((0, now));

        // Reset counter if more than 10 seconds since the last message
        if now.duration_since(entry.1).as_secs() > 10 {
            entry.0 = 0;
        }

        entry.0 += 1;
        entry.1 = now;

        should_action = entry.0 >= flood_limit as u32;

        if should_action {
            entry.0 = 0; // Reset after action
        }
    }

    if should_action {
        let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
        let name = formatting::mention_html(from);
        match flood_mode.as_str() {
            "ban" => {
                bot.ban_chat_member(chat_id, from.id).await.ok();
                bot.send_message(
                    chat_id,
                    i18n::t(&lang, "antiflood-triggered-ban", Some(&[("name", &name)])),
                )
                .parse_mode(ParseMode::Html)
                .await?;
            }
            "kick" => {
                safe_kick(&bot, chat_id, from.id).await.ok();
                bot.send_message(
                    chat_id,
                    i18n::t(&lang, "antiflood-triggered-kick", Some(&[("name", &name)])),
                )
                .parse_mode(ParseMode::Html)
                .await?;
            }
            _ => {
                let perms = ChatPermissions::empty();
                bot.restrict_chat_member(chat_id, from.id, perms)
                    .await
                    .ok();
                bot.send_message(
                    chat_id,
                    i18n::t(&lang, "antiflood-triggered-mute", Some(&[("name", &name)])),
                )
                .parse_mode(ParseMode::Html)
                .await?;
            }
        }
    }

    Ok(())
}

/// Handle antiflood settings callbacks
pub async fn antiflood_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let chat_id = msg.chat().id;
    let data = q.data.as_deref().unwrap_or("");

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    if !permissions::is_user_admin(&bot, chat_id, q.from.id).await {
        bot.answer_callback_query(q.id.clone())
            .text(i18n::t(&lang, "antiflood-no-permission", None))
            .await?;
        return Ok(());
    }

    if let Some(count_str) = data.strip_prefix("af_set_") {
        if let Ok(count) = count_str.parse::<i32>() {
            let chat_name = msg.chat().title().unwrap_or("Unknown");
            db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("antiflood::upsert_chat_cb");
            db::queries::set_antiflood(&pool, chat_id.0, count).await.log_err("antiflood::set_cb");
            FLOOD_SETTINGS_CACHE.invalidate(&chat_id.0);
            let text = if count == 0 {
                i18n::t(&lang, "antiflood-disabled", None)
            } else {
                let c = count.to_string();
                i18n::t(&lang, "antiflood-set", Some(&[("count", &c)]))
            };
            bot.answer_callback_query(q.id.clone()).text(&text).await?;
            bot.edit_message_text(msg.chat().id, msg.id(), &text).await?;
        }
    } else if let Some(mode) = data.strip_prefix("af_mode_") {
        let chat_name = msg.chat().title().unwrap_or("Unknown");
        db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("antiflood::upsert_chat_mode_cb");
        db::queries::set_antiflood_mode(&pool, chat_id.0, mode).await.log_err("antiflood::set_mode_cb");
        FLOOD_SETTINGS_CACHE.invalidate(&chat_id.0);
        let text = i18n::t(&lang, "antiflood-mode-set", Some(&[("mode", mode)]));
        bot.answer_callback_query(q.id.clone()).text(&text).await?;
        bot.edit_message_text(msg.chat().id, msg.id(), &text).await?;
    }

    Ok(())
}