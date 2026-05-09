use teloxide::prelude::*;
use teloxide::types::{MessageEntityKind, ParseMode};

use crate::db;
use crate::utils::{formatting, formatting::uid_to_i64, i18n};

pub async fn afk(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let reason: String = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .collect::<Vec<_>>()
        .join(" ");

    db::queries::set_afk(&pool, uid_to_i64(from.id), &reason).await.ok();

    let lang = i18n::get_chat_lang(&pool, msg.chat.id.0).await;
    let name = formatting::html_escape(&from.first_name);
    let text = if reason.is_empty() {
        i18n::t(&lang, "afk-now", Some(&[("name", &name)]))
    } else {
        let escaped_reason = formatting::html_escape(&reason);
        i18n::t(&lang, "afk-now-reason", Some(&[("name", &name), ("reason", &escaped_reason)]))
    };

    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

/// Also trigger AFK on "brb" messages
#[allow(dead_code)]
pub async fn brb_afk(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let text = msg.text().unwrap_or("").to_lowercase();
    if text.contains("brb") {
        afk(bot, msg, pool).await?;
    }
    Ok(())
}

/// Check if a user is no longer AFK when they send a message
pub async fn check_no_longer_afk(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    // If user sends a message and is AFK, remove AFK status
    if db::queries::remove_afk(&pool, uid_to_i64(from.id))
        .await
        .unwrap_or(false)
    {
        let lang = i18n::get_chat_lang(&pool, msg.chat.id.0).await;
        let name = formatting::html_escape(&from.first_name);
        bot.send_message(
            msg.chat.id,
            i18n::t(&lang, "afk-back", Some(&[("name", &name)])),
        )
        .parse_mode(ParseMode::Html)
        .await?;
    }

    Ok(())
}

/// Check if anyone mentioned/replied to is AFK
pub async fn check_afk_reply(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let lang = i18n::get_chat_lang(&pool, msg.chat.id.0).await;

    // Check reply
    if let Some(reply) = msg.reply_to_message() {
        if let Some(reply_user) = reply.from.as_ref() {
            if let Ok(Some(afk_info)) = db::queries::get_afk(&pool, uid_to_i64(reply_user.id)).await {
                let name = formatting::html_escape(&reply_user.first_name);
                let text = if afk_info.reason.is_empty() {
                    i18n::t(&lang, "afk-is", Some(&[("name", &name)]))
                } else {
                    let escaped_reason = formatting::html_escape(&afk_info.reason);
                    i18n::t(&lang, "afk-is-reason", Some(&[("name", &name), ("reason", &escaped_reason)]))
                };
                bot.send_message(msg.chat.id, text)
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
        }
    }

    // Check mentions in message entities
    if let Some(entities) = msg.entities() {
        for entity in entities {
            match &entity.kind {
                MessageEntityKind::Mention => {
                    let text = msg.text().unwrap_or("");
                    let mention = &text[entity.offset..entity.offset + entity.length];
                    let username = mention.trim_start_matches('@');
                    // We can't easily resolve username->id without a lookup,
                    // so we skip pure @username mentions for AFK check
                    let _ = username;
                }
                MessageEntityKind::TextMention { user } => {
                    if let Ok(Some(afk_info)) = db::queries::get_afk(&pool, uid_to_i64(user.id)).await
                    {
                        let name = formatting::html_escape(&user.first_name);
                        let text = if afk_info.reason.is_empty() {
                            i18n::t(&lang, "afk-is", Some(&[("name", &name)]))
                        } else {
                            let escaped_reason = formatting::html_escape(&afk_info.reason);
                            i18n::t(&lang, "afk-is-reason", Some(&[("name", &name), ("reason", &escaped_reason)]))
                        };
                        bot.send_message(msg.chat.id, text)
                            .parse_mode(ParseMode::Html)
                            .await?;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
