use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use teloxide::prelude::*;
use teloxide::types::{MessageEntityKind, ParseMode};

use crate::db;
use crate::utils::{cache::TtlCache, i18n, permissions};

static LOCK_CACHE: Lazy<Arc<TtlCache<i64, Vec<String>>>> =
    Lazy::new(|| TtlCache::new(Duration::from_secs(30)));

const LOCK_TYPES: &[&str] = &[
    "sticker", "audio", "voice", "document", "video", "videonote",
    "contact", "photo", "gif", "url", "forward", "game", "location",
    "rtl", "button", "inline", "media", "all",
];

pub async fn lock(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "locks-no-permission", None))
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
        let mut text = i18n::t(&lang, "locks-usage", None);
        text.push_str(&format!("\n\n{}", i18n::t(&lang, "locks-available", None)));
        bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let lock_type = args[0].to_lowercase();
    if !LOCK_TYPES.contains(&lock_type.as_str()) {
        bot.send_message(
            chat_id,
            i18n::t(&lang, "locks-invalid-type", Some(&[("type", &lock_type)])),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();
    db::queries::add_lock(&pool, chat_id.0, &lock_type).await.ok();
    LOCK_CACHE.invalidate(&chat_id.0);

    bot.send_message(
        chat_id,
        i18n::t(&lang, "locks-locked", Some(&[("type", &lock_type)])),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn unlock(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "locks-no-permission", None))
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
        bot.send_message(chat_id, i18n::t(&lang, "locks-usage", None))
            .await?;
        return Ok(());
    }

    let lock_type = args[0].to_lowercase();

    if lock_type == "all" {
        db::queries::remove_all_locks(&pool, chat_id.0).await.ok();
        LOCK_CACHE.invalidate(&chat_id.0);
        bot.send_message(chat_id, i18n::t(&lang, "locks-all-removed", None))
            .await?;
        return Ok(());
    }

    match db::queries::remove_lock(&pool, chat_id.0, &lock_type).await {
        Ok(true) => {
            LOCK_CACHE.invalidate(&chat_id.0);
            bot.send_message(
                chat_id,
                i18n::t(&lang, "locks-unlocked", Some(&[("type", &lock_type)])),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {
            bot.send_message(
                chat_id,
                i18n::t(&lang, "locks-not-locked", Some(&[("type", &lock_type)])),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }
    Ok(())
}

pub async fn locks_list(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    let locks = db::queries::get_locks(&pool, chat_id.0)
        .await
        .unwrap_or_default();

    if locks.is_empty() {
        bot.send_message(chat_id, i18n::t(&lang, "locks-none-active", None))
            .await?;
        return Ok(());
    }

    let mut text = format!("{}\n", i18n::t(&lang, "locks-header", None));
    for lock in &locks {
        text.push_str(&format!("\n• <code>{}</code>", lock));
    }

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn locktypes(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let lang = i18n::get_chat_lang(&pool, msg.chat.id.0).await;
    bot.send_message(msg.chat.id, i18n::t(&lang, "locks-available", None))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

/// Check incoming messages against locks.
/// Returns `true` if the message was consumed (deleted).
pub async fn check_locks(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<bool> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(false),
    };

    // Don't check admins
    if permissions::is_user_admin(&bot, chat_id, from.id).await {
        return Ok(false);
    }

    let locks = if let Some(cached) = LOCK_CACHE.get(&chat_id.0) {
        cached
    } else {
        let list = db::queries::get_locks(&pool, chat_id.0)
            .await
            .unwrap_or_default();
        LOCK_CACHE.set(chat_id.0, list.clone());
        list
    };

    if locks.is_empty() {
        return Ok(false);
    }

    let should_delete = locks.iter().any(|lock| match lock.as_str() {
        "all" => true,
        "sticker" => msg.sticker().is_some(),
        "audio" => msg.audio().is_some(),
        "voice" => msg.voice().is_some(),
        "document" => msg.document().is_some() && msg.animation().is_none(),
        "video" => msg.video().is_some(),
        "videonote" => msg.video_note().is_some(),
        "contact" => msg.contact().is_some(),
        "photo" => msg.photo().is_some(),
        "gif" => msg.animation().is_some(),
        "game" => msg.game().is_some(),
        "location" => msg.location().is_some(),
        "forward" => msg.forward_date().is_some(),
        "url" => has_url(&msg),
        "media" => {
            msg.photo().is_some()
                || msg.audio().is_some()
                || msg.video().is_some()
                || msg.document().is_some()
                || msg.voice().is_some()
                || msg.video_note().is_some()
                || msg.animation().is_some()
        }
        "button" | "inline" => msg.reply_markup().is_some(),
        "rtl" => has_rtl(msg.text().unwrap_or("")),
        _ => false,
    });

    if should_delete {
        bot.delete_message(chat_id, msg.id).await.ok();
        return Ok(true);
    }

    Ok(false)
}

fn has_url(msg: &Message) -> bool {
    if let Some(entities) = msg.entities() {
        for entity in entities {
            match entity.kind {
                MessageEntityKind::Url | MessageEntityKind::TextLink { .. } => return true,
                _ => {}
            }
        }
    }
    false
}

fn has_rtl(text: &str) -> bool {
    text.chars().any(|c| {
        ('\u{0590}'..='\u{05FF}').contains(&c) // Hebrew
            || ('\u{0600}'..='\u{06FF}').contains(&c) // Arabic
            || ('\u{0750}'..='\u{077F}').contains(&c) // Arabic Supplement
            || ('\u{FB50}'..='\u{FDFF}').contains(&c) // Arabic Presentation Forms-A
            || ('\u{FE70}'..='\u{FEFF}').contains(&c) // Arabic Presentation Forms-B
    })
}
