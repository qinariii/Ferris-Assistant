use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::db;
use crate::utils::{formatting, formatting::uid_to_i64, permissions};

pub async fn connect(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    // If in a group, connect to this group
    if msg.chat.is_group() || msg.chat.is_supergroup() {
        db::queries::connect_chat(&pool, uid_to_i64(from.id), chat_id.0)
            .await
            .ok();
        let chat_name = msg.chat.title().unwrap_or("this chat");
        bot.send_message(
            chat_id,
            format!(
                "✅ You are now connected to <b>{}</b>.\nYou can manage this group from my PM.",
                formatting::html_escape(chat_name)
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    // If in PM, connect to specified chat
    if args.is_empty() {
        bot.send_message(
            chat_id,
            "❌ Usage: /connect <chat_id>\n\nOr use /connect in a group directly.",
        )
        .await?;
        return Ok(());
    }

    let target_chat_id: i64 = match args[0].parse() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(chat_id, "❌ Invalid chat ID.")
                .await?;
            return Ok(());
        }
    };

    // Verify user is admin in that chat
    if !permissions::is_user_admin(&bot, ChatId(target_chat_id), from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin in that group to connect.")
            .await?;
        return Ok(());
    }

    let target_chat = bot.get_chat(ChatId(target_chat_id)).await;
    let chat_name = target_chat
        .ok()
        .and_then(|c| c.title().map(|t| t.to_string()))
        .unwrap_or_else(|| target_chat_id.to_string());

    db::queries::connect_chat(&pool, uid_to_i64(from.id), target_chat_id)
        .await
        .ok();

    bot.send_message(
        chat_id,
        format!(
            "✅ Connected to <b>{}</b>!\n\nYou can now use admin commands here to manage that group.",
            formatting::html_escape(&chat_name)
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn disconnect(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    match db::queries::disconnect_chat(&pool, uid_to_i64(from.id)).await {
        Ok(true) => {
            bot.send_message(msg.chat.id, "✅ Disconnected from the group.")
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "❌ You are not connected to any group.")
                .await?;
        }
    }
    Ok(())
}

pub async fn connection_status(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let connected_chat = db::queries::get_connection(&pool, uid_to_i64(from.id))
        .await
        .unwrap_or(None);

    match connected_chat {
        Some(target_id) => {
            let chat_info = bot.get_chat(ChatId(target_id)).await;
            let chat_name = chat_info
                .ok()
                .and_then(|c| c.title().map(|t| t.to_string()))
                .unwrap_or_else(|| target_id.to_string());

            bot.send_message(
                chat_id,
                format!(
                    "🔗 Currently connected to: <b>{}</b> (<code>{}</code>)\n\nUse /disconnect to disconnect.",
                    formatting::html_escape(&chat_name),
                    target_id
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        None => {
            bot.send_message(
                chat_id,
                "🔗 You are not connected to any group.\nUse /connect <chat_id> or /connect in a group.",
            )
            .await?;
        }
    }
    Ok(())
}

/// Get the effective chat ID (connected chat or current chat)
#[allow(dead_code)]
pub async fn get_effective_chat(
    pool: &db::Pool,
    msg: &Message,
) -> i64 {
    let chat_id = msg.chat.id.0;

    // Only use connection in private chats
    if msg.chat.is_private() {
        if let Some(from) = msg.from.as_ref() {
            if let Ok(Some(connected)) = db::queries::get_connection(pool, uid_to_i64(from.id)).await {
                return connected;
            }
        }
    }

    chat_id
}
