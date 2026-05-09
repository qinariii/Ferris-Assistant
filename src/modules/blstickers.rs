use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::db;
use crate::utils::{cache::TtlCache, formatting, permissions};

static BLSTICKER_CACHE: Lazy<Arc<TtlCache<i64, Vec<String>>>> =
    Lazy::new(|| TtlCache::new(Duration::from_secs(30)));

// Mode: 0=off, 1=delete, 2=warn, 3=mute, 4=kick, 5=ban

pub async fn blsticker_list(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let chat_name = msg.chat.title().unwrap_or("Private");

    let stickers = db::queries::get_blacklist_stickers(&pool, chat_id.0)
        .await
        .unwrap_or_default();

    if stickers.is_empty() {
        bot.send_message(
            chat_id,
            format!(
                "No blacklisted sticker sets in <b>{}</b>!",
                formatting::html_escape(chat_name)
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let mut text = format!(
        "<b>Blacklisted stickers in {}:</b>\n",
        formatting::html_escape(chat_name)
    );
    for s in &stickers {
        text.push_str(&format!("\n• <code>{}</code>", formatting::html_escape(s)));
    }

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn add_blsticker(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to manage sticker blacklist.")
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();

    // Try to get set name from reply or args
    let set_name = if let Some(reply) = msg.reply_to_message() {
        reply
            .sticker()
            .and_then(|s| s.set_name.clone())
    } else {
        msg.text()
            .unwrap_or("")
            .split_whitespace()
            .nth(1)
            .map(|s| s.replace("https://t.me/addstickers/", ""))
    };

    let set_name = match set_name {
        Some(s) if !s.is_empty() => s,
        _ => {
            bot.send_message(
                chat_id,
                "❌ Reply to a sticker or provide a sticker set name.\nUsage: /addblsticker <set_name>",
            )
            .await?;
            return Ok(());
        }
    };

    db::queries::add_blacklist_sticker(&pool, chat_id.0, &set_name).await.ok();
    BLSTICKER_CACHE.invalidate(&chat_id.0);

    bot.send_message(
        chat_id,
        format!(
            "✅ Sticker set <code>{}</code> added to blacklist in <b>{}</b>!",
            formatting::html_escape(&set_name),
            formatting::html_escape(chat_name),
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn rm_blsticker(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to manage sticker blacklist.")
            .await?;
        return Ok(());
    }

    let set_name = if let Some(reply) = msg.reply_to_message() {
        reply
            .sticker()
            .and_then(|s| s.set_name.clone())
    } else {
        msg.text()
            .unwrap_or("")
            .split_whitespace()
            .nth(1)
            .map(|s| s.replace("https://t.me/addstickers/", ""))
    };

    let set_name = match set_name {
        Some(s) if !s.is_empty() => s,
        _ => {
            bot.send_message(chat_id, "❌ Reply to a sticker or provide a sticker set name.")
                .await?;
            return Ok(());
        }
    };

    match db::queries::remove_blacklist_sticker(&pool, chat_id.0, &set_name).await {
        Ok(true) => {
            BLSTICKER_CACHE.invalidate(&chat_id.0);
            bot.send_message(
                chat_id,
                format!(
                    "✅ Sticker set <code>{}</code> removed from blacklist!",
                    formatting::html_escape(&set_name)
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {
            bot.send_message(chat_id, "❌ That sticker set is not in the blacklist.")
                .await?;
        }
    }
    Ok(())
}

pub async fn blsticker_mode(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to change sticker blacklist mode.")
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
        let mode = db::queries::get_blsticker_mode(&pool, chat_id.0).await.unwrap_or(1);
        let mode_name = mode_to_str(mode);
        bot.send_message(
            chat_id,
            format!(
                "Current sticker blacklist mode: <b>{}</b>\n\nUsage: /blstickermode &lt;off/del/warn/mute/kick/ban&gt;",
                mode_name
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let mode = match args[0].to_lowercase().as_str() {
        "off" | "nothing" | "no" => 0,
        "del" | "delete" => 1,
        "warn" => 2,
        "mute" => 3,
        "kick" => 4,
        "ban" => 5,
        _ => {
            bot.send_message(chat_id, "❌ Unknown mode. Use: off/del/warn/mute/kick/ban")
                .await?;
            return Ok(());
        }
    };

    db::queries::set_blsticker_mode(&pool, chat_id.0, mode).await.ok();

    bot.send_message(
        chat_id,
        format!("✅ Sticker blacklist mode set to <b>{}</b>!", mode_to_str(mode)),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

/// Check incoming stickers against blacklist.
/// Returns `true` if the message was consumed (deleted/actioned).
pub async fn check_blacklist_sticker(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<bool> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(false),
    };

    // Don't check admins
    if permissions::is_user_admin(&bot, chat_id, from.id).await {
        return Ok(false);
    }

    let sticker = match msg.sticker() {
        Some(s) => s,
        None => return Ok(false),
    };

    let set_name = match &sticker.set_name {
        Some(name) => name.to_lowercase(),
        None => return Ok(false),
    };

    let blacklisted = if let Some(cached) = BLSTICKER_CACHE.get(&chat_id.0) {
        cached
    } else {
        let list = db::queries::get_blacklist_stickers(&pool, chat_id.0)
            .await
            .unwrap_or_default();
        BLSTICKER_CACHE.set(chat_id.0, list.clone());
        list
    };

    if !blacklisted.iter().any(|b| b.to_lowercase() == set_name) {
        return Ok(false);
    }

    let mode = db::queries::get_blsticker_mode(&pool, chat_id.0).await.unwrap_or(1);

    let actioned = match mode {
        0 => false, // off
        1 => {
            // delete
            bot.delete_message(chat_id, msg.id).await.ok();
            true
        }
        2 => {
            // warn (delete + warn message)
            bot.delete_message(chat_id, msg.id).await.ok();
            bot.send_message(
                chat_id,
                format!(
                    "⚠️ {} warned for using blacklisted sticker set '{}'!",
                    formatting::mention_html(from),
                    formatting::html_escape(&set_name),
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            true
        }
        3 => {
            // mute
            bot.delete_message(chat_id, msg.id).await.ok();
            bot.restrict_chat_member(chat_id, from.id, teloxide::types::ChatPermissions::empty())
                .await
                .ok();
            bot.send_message(
                chat_id,
                format!(
                    "🔇 {} muted for using blacklisted sticker set '{}'!",
                    formatting::mention_html(from),
                    formatting::html_escape(&set_name),
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            true
        }
        4 => {
            // kick
            bot.delete_message(chat_id, msg.id).await.ok();
            bot.ban_chat_member(chat_id, from.id).await.ok();
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            bot.unban_chat_member(chat_id, from.id).await.ok();
            bot.send_message(
                chat_id,
                format!(
                    "👢 {} kicked for using blacklisted sticker set '{}'!",
                    formatting::mention_html(from),
                    formatting::html_escape(&set_name),
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            true
        }
        5 => {
            // ban
            bot.delete_message(chat_id, msg.id).await.ok();
            bot.ban_chat_member(chat_id, from.id).await.ok();
            bot.send_message(
                chat_id,
                format!(
                    "🚫 {} banned for using blacklisted sticker set '{}'!",
                    formatting::mention_html(from),
                    formatting::html_escape(&set_name),
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            true
        }
        _ => false,
    };

    Ok(actioned)
}

fn mode_to_str(mode: i32) -> &'static str {
    match mode {
        0 => "off",
        1 => "delete",
        2 => "warn",
        3 => "mute",
        4 => "kick",
        5 => "ban",
        _ => "unknown",
    }
}
