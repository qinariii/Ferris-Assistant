use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, ReactionType};

use crate::db;
use crate::utils::{cache::TtlCache, formatting, permissions};

static REACTION_CACHE: Lazy<Arc<TtlCache<i64, Vec<db::models::Reaction>>>> =
    Lazy::new(|| TtlCache::new(Duration::from_secs(30)));


pub async fn addreaction(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to use this command.")
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

    if args.len() < 2 {
        bot.send_message(
            chat_id,
            "❌ Usage: /addreaction <keyword> <emoji>\n\nExample: /addreaction hello 👋",
        )
        .await?;
        return Ok(());
    }

    let keyword = args[0].to_lowercase();
    let emoji = &args[1];

    db::queries::add_reaction(&pool, chat_id.0, &keyword, emoji).await.ok();
    REACTION_CACHE.invalidate(&chat_id.0);

    bot.send_message(
        chat_id,
        format!(
            "✅ Reaction added! When someone says <b>{}</b>, I'll react with {}",
            formatting::html_escape(&keyword),
            formatting::html_escape(emoji),
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}


pub async fn removereaction(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to use this command.")
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
        bot.send_message(chat_id, "❌ Usage: /removereaction <keyword>")
            .await?;
        return Ok(());
    }

    let keyword = args[0].to_lowercase();

    match db::queries::remove_reaction(&pool, chat_id.0, &keyword).await {
        Ok(true) => {
            REACTION_CACHE.invalidate(&chat_id.0);
            bot.send_message(
                chat_id,
                format!(
                    "✅ Reaction for <b>{}</b> removed.",
                    formatting::html_escape(&keyword),
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {
            bot.send_message(chat_id, "❌ No reaction found for that keyword.")
                .await?;
        }
    }
    Ok(())
}


pub async fn reactions_list(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    let reactions = db::queries::get_reactions(&pool, chat_id.0)
        .await
        .unwrap_or_default();

    if reactions.is_empty() {
        bot.send_message(chat_id, "📭 No reactions set for this chat.")
            .await?;
        return Ok(());
    }

    let mut text = format!("⚡ <b>Reactions ({}):</b>\n\n", reactions.len());
    for r in &reactions {
        text.push_str(&format!(
            "• <code>{}</code> → {}\n",
            formatting::html_escape(&r.keyword),
            formatting::html_escape(&r.emoji),
        ));
    }

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}


pub async fn resetreactions(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to use this command.")
            .await?;
        return Ok(());
    }

    db::queries::reset_reactions(&pool, chat_id.0).await.ok();
    REACTION_CACHE.invalidate(&chat_id.0);
    bot.send_message(chat_id, "✅ All reactions have been cleared.")
        .await?;
    Ok(())
}


pub async fn check_reactions(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let text = match msg.text() {
        Some(t) => t.to_lowercase(),
        None => return Ok(()),
    };

    let chat_id = msg.chat.id;

    let reactions = if let Some(cached) = REACTION_CACHE.get(&chat_id.0) {
        cached
    } else {
        let list = db::queries::get_reactions(&pool, chat_id.0)
            .await
            .unwrap_or_default();
        REACTION_CACHE.set(chat_id.0, list.clone());
        list
    };

    if reactions.is_empty() {
        return Ok(());
    }

    for reaction in &reactions {
        if text.contains(&reaction.keyword) {
            // Use Telegram's setMessageReaction API
            bot.set_message_reaction(chat_id, msg.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: reaction.emoji.clone(),
                }])
                .send()
                .await
                .ok();
            // Only react with first match to avoid rate limits
            break;
        }
    }

    Ok(())
}
