use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::db;
use crate::utils::{cache::TtlCache, formatting, permissions};

static FILTER_CACHE: Lazy<Arc<TtlCache<i64, Vec<db::models::Filter>>>> =
    Lazy::new(|| TtlCache::new(Duration::from_secs(30)));

pub async fn add_filter(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to add filters.")
            .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let args: Vec<&str> = text.splitn(3, ' ').collect();

    if args.len() < 3 {
        // Check reply
        if let Some(reply) = msg.reply_to_message() {
            if args.len() >= 2 {
                let keyword = args[1];
                let reply_text = reply.text().unwrap_or("").to_string();
                if reply_text.is_empty() {
                    bot.send_message(chat_id, "❌ The replied message has no text content.")
                        .await?;
                    return Ok(());
                }

                let chat_name = msg.chat.title().unwrap_or("Private");
                db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();
                db::queries::add_filter(&pool, chat_id.0, keyword, &reply_text)
                    .await
                    .ok();
                FILTER_CACHE.invalidate(&chat_id.0);

                bot.send_message(
                    chat_id,
                    format!(
                        "✅ Filter <b>{}</b> added! I'll reply when someone says it.",
                        formatting::html_escape(keyword)
                    ),
                )
                .parse_mode(ParseMode::Html)
                .await?;
                return Ok(());
            }
        }

        bot.send_message(chat_id, "❌ Usage: /filter <keyword> <reply text> or reply to a message with /filter <keyword>")
            .await?;
        return Ok(());
    }

    let keyword = args[1];
    let reply_text = args[2];

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();
    db::queries::add_filter(&pool, chat_id.0, keyword, reply_text)
        .await
        .ok();
    FILTER_CACHE.invalidate(&chat_id.0);

    bot.send_message(
        chat_id,
        format!(
            "✅ Filter <b>{}</b> added! I'll reply when someone says it.",
            formatting::html_escape(keyword)
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn list_filters(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    let filters = db::queries::get_filters(&pool, chat_id.0)
        .await
        .unwrap_or_default();

    if filters.is_empty() {
        bot.send_message(chat_id, "🔍 No filters set in this chat.")
            .await?;
        return Ok(());
    }

    let mut text = "<b>🔍 Filters in this chat:</b>\n".to_string();
    for f in &filters {
        text.push_str(&format!("\n• <code>{}</code>", formatting::html_escape(&f.keyword)));
    }
    text.push_str("\n\n<i>Use /stop &lt;keyword&gt; to remove a filter.</i>");

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn stop_filter(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to remove filters.")
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
        bot.send_message(chat_id, "❌ Usage: /stop <keyword>")
            .await?;
        return Ok(());
    }

    let keyword = &args[0];
    match db::queries::delete_filter(&pool, chat_id.0, keyword).await {
        Ok(true) => {
            FILTER_CACHE.invalidate(&chat_id.0);
            bot.send_message(
                chat_id,
                format!("✅ Filter <b>{}</b> removed!", formatting::html_escape(keyword)),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {
            bot.send_message(
                chat_id,
                format!("❌ Filter <b>{}</b> not found.", formatting::html_escape(keyword)),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }
    Ok(())
}

pub async fn stopall(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to remove all filters.")
            .await?;
        return Ok(());
    }

    db::queries::delete_all_filters(&pool, chat_id.0).await.ok();
    FILTER_CACHE.invalidate(&chat_id.0);

    bot.send_message(chat_id, "✅ All filters have been removed!")
        .await?;
    Ok(())
}

/// Handle incoming messages for filter matching
pub async fn check_filters(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or("").to_lowercase();

    if text.is_empty() || text.starts_with('/') {
        return Ok(());
    }

    let filters = if let Some(cached) = FILTER_CACHE.get(&chat_id.0) {
        cached
    } else {
        let list = db::queries::get_filters(&pool, chat_id.0)
            .await
            .unwrap_or_default();
        FILTER_CACHE.set(chat_id.0, list.clone());
        list
    };

    for f in &filters {
        if text.contains(&f.keyword) {
            bot.send_message(chat_id, &f.reply_text)
                .parse_mode(ParseMode::Html)
                .await?;
            break;
        }
    }
    Ok(())
}
