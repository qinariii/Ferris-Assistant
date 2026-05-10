use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use teloxide::prelude::*;
use teloxide::types::{ChatPermissions, ParseMode};

use regex::Regex;

use crate::config::AppConfig;
use crate::db;
use crate::utils::{cache::TtlCache, formatting, i18n, kick::safe_kick, permissions};

static BLACKLIST_CACHE: Lazy<Arc<TtlCache<i64, Vec<db::models::Blacklist>>>> =
    Lazy::new(|| TtlCache::new(Duration::from_secs(30)));

pub async fn blacklist(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    let list = db::queries::get_blacklist(&pool, chat_id.0)
        .await
        .unwrap_or_default();

    if list.is_empty() {
        bot.send_message(chat_id, i18n::t(&lang, "blacklist-list-empty", None))
            .await?;
        return Ok(());
    }

    let mut text = format!("{}\n", i18n::t(&lang, "blacklist-list-header", None));
    for item in &list {
        text.push_str(&format!(
            "\n• <code>{}</code> [{}]",
            formatting::html_escape(&item.trigger_word),
            item.mode
        ));
    }

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn add_blacklist(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "blacklist-no-permission", None))
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
        bot.send_message(chat_id, i18n::t(&lang, "blacklist-add-usage", None))
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();

    let word = &args[0];
    db::queries::add_blacklist(&pool, chat_id.0, word, "delete")
        .await
        .ok();
    BLACKLIST_CACHE.invalidate(&chat_id.0);

    let escaped_word = formatting::html_escape(word);
    bot.send_message(
        chat_id,
        i18n::t(&lang, "blacklist-add-success", Some(&[("word", &escaped_word)])),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn rm_blacklist(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "blacklist-no-permission", None))
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
        bot.send_message(chat_id, i18n::t(&lang, "blacklist-remove-usage", None))
            .await?;
        return Ok(());
    }

    let word = &args[0];
    match db::queries::remove_blacklist(&pool, chat_id.0, word).await {
        Ok(true) => {
            BLACKLIST_CACHE.invalidate(&chat_id.0);
            let escaped_word = formatting::html_escape(word);
            bot.send_message(
                chat_id,
                i18n::t(&lang, "blacklist-remove-success", Some(&[("word", &escaped_word)])),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {
            bot.send_message(chat_id, i18n::t(&lang, "blacklist-remove-not-found", None))
                .await?;
        }
    }
    Ok(())
}

pub async fn blacklist_mode(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "blacklist-no-permission", None))
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
        bot.send_message(chat_id, i18n::t(&lang, "blacklist-mode-usage", None))
            .await?;
        return Ok(());
    }

    let mode = args[0].to_lowercase();
    if !["delete", "warn", "mute", "kick", "ban"].contains(&mode.as_str()) {
        bot.send_message(chat_id, i18n::t(&lang, "blacklist-mode-invalid", None))
            .await?;
        return Ok(());
    }

    db::queries::set_blacklist_mode(&pool, chat_id.0, &mode).await.ok();
    BLACKLIST_CACHE.invalidate(&chat_id.0);

    bot.send_message(
        chat_id,
        i18n::t(&lang, "blacklist-mode-set", Some(&[("mode", &mode)])),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

/// Check incoming messages against blacklist.
/// Returns `true` if the message was consumed (deleted/actioned).
pub async fn check_blacklist(
    bot: Bot,
    msg: Message,
    _cfg: AppConfig,
    pool: db::Pool,
) -> ResponseResult<bool> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(false),
    };

    // Don't check admins
    if permissions::is_user_admin(&bot, chat_id, from.id).await {
        return Ok(false);
    }

    let text = msg.text().or_else(|| msg.caption()).unwrap_or("").to_lowercase();
    if text.is_empty() {
        return Ok(false);
    }

    let blacklist = if let Some(cached) = BLACKLIST_CACHE.get(&chat_id.0) {
        cached
    } else {
        let list = db::queries::get_blacklist(&pool, chat_id.0)
            .await
            .unwrap_or_default();
        BLACKLIST_CACHE.set(chat_id.0, list.clone());
        list
    };

    for item in &blacklist {
        let pattern = format!(r"(?i)\b{}\b", regex::escape(&item.trigger_word));
        let matched = Regex::new(&pattern).map(|re| re.is_match(&text)).unwrap_or(false);
        if matched {
            let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
            let name = formatting::mention_html(from);
            match item.mode.as_str() {
                "delete" => {
                    bot.delete_message(chat_id, msg.id).await.ok();
                }
                "warn" => {
                    bot.delete_message(chat_id, msg.id).await.ok();
                    bot.send_message(
                        chat_id,
                        i18n::t(&lang, "blacklist-triggered-warn", Some(&[("name", &name)])),
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                }
                "mute" => {
                    bot.delete_message(chat_id, msg.id).await.ok();
                    let perms = ChatPermissions::empty();
                    bot.restrict_chat_member(chat_id, from.id, perms)
                        .await
                        .ok();
                    bot.send_message(
                        chat_id,
                        i18n::t(&lang, "blacklist-triggered-mute", Some(&[("name", &name)])),
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                }
                "kick" => {
                    bot.delete_message(chat_id, msg.id).await.ok();
                    safe_kick(&bot, chat_id, from.id).await.ok();
                }
                "ban" => {
                    bot.delete_message(chat_id, msg.id).await.ok();
                    bot.ban_chat_member(chat_id, from.id).await.ok();
                    bot.send_message(
                        chat_id,
                        i18n::t(&lang, "blacklist-triggered-ban", Some(&[("name", &name)])),
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                }
                _ => {
                    bot.delete_message(chat_id, msg.id).await.ok();
                }
            }
            return Ok(true);
        }
    }
    Ok(false)
}
