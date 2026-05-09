use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::db;
use crate::utils::permissions;

pub async fn log_channel(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to view log settings.")
            .await?;
        return Ok(());
    }

    let log_ch = db::queries::get_log_channel(&pool, chat_id.0)
        .await
        .unwrap_or(None);

    match log_ch {
        Some(channel_id) => {
            let ch_info = bot.get_chat(ChatId(channel_id)).await;
            let ch_name = ch_info
                .ok()
                .and_then(|c| c.title().map(|t| t.to_string()))
                .unwrap_or_else(|| channel_id.to_string());
            bot.send_message(
                chat_id,
                format!("📋 Log channel: <b>{}</b> (<code>{}</code>)", ch_name, channel_id),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        None => {
            bot.send_message(chat_id, "📋 No log channel set.\nUse /setlogchannel in a channel to set one.")
                .await?;
        }
    }
    Ok(())
}

pub async fn set_log_channel(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to set a log channel.")
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
        bot.send_message(
            chat_id,
            "❌ Usage: /setlogchannel <channel_id>\n\nOr forward a message from the channel and use /setlogchannel in your group.",
        )
        .await?;
        return Ok(());
    }

    let channel_id: i64 = match args[0].parse() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(chat_id, "❌ Invalid channel ID. Use a numeric ID.")
                .await?;
            return Ok(());
        }
    };

    // Verify bot is admin in the channel
    let bot_member = bot.get_chat_member(ChatId(channel_id), bot.get_me().await?.id).await;
    match bot_member {
        Ok(member) if member.is_privileged() => {}
        _ => {
            bot.send_message(
                chat_id,
                "❌ I need to be an admin in that channel to send log messages.",
            )
            .await?;
            return Ok(());
        }
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();
    db::queries::set_log_channel(&pool, chat_id.0, Some(channel_id)).await.ok();

    bot.send_message(chat_id, "✅ Log channel set successfully!")
        .await?;

    // Send a test message to the channel
    bot.send_message(
        ChatId(channel_id),
        format!(
            "📋 This channel has been linked as log channel for <b>{}</b>.",
            crate::utils::formatting::html_escape(chat_name)
        ),
    )
    .parse_mode(ParseMode::Html)
    .await
    .ok();

    Ok(())
}

pub async fn unset_log_channel(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to unset the log channel.")
            .await?;
        return Ok(());
    }

    db::queries::set_log_channel(&pool, chat_id.0, None).await.ok();

    bot.send_message(chat_id, "✅ Log channel has been removed.")
        .await?;
    Ok(())
}

/// Send a log entry to the configured log channel
pub async fn send_log(
    bot: &Bot,
    pool: &db::Pool,
    chat_id: i64,
    log_text: &str,
) {
    if let Ok(Some(channel_id)) = db::queries::get_log_channel(pool, chat_id).await {
        bot.send_message(ChatId(channel_id), log_text)
            .parse_mode(ParseMode::Html)
            .await
            .ok();
    }
}

/// Format a log entry for admin actions
#[allow(dead_code)]
pub fn format_log(
    action: &str,
    chat_name: &str,
    admin_name: &str,
    target_name: &str,
    reason: Option<&str>,
) -> String {
    let mut log = format!(
        "📋 <b>#{}</b>\n\n\
        <b>Chat:</b> {}\n\
        <b>Admin:</b> {}\n\
        <b>User:</b> {}",
        crate::utils::formatting::html_escape(action),
        crate::utils::formatting::html_escape(chat_name),
        crate::utils::formatting::html_escape(admin_name),
        crate::utils::formatting::html_escape(target_name),
    );

    if let Some(r) = reason {
        if !r.is_empty() {
            log.push_str(&format!(
                "\n<b>Reason:</b> {}",
                crate::utils::formatting::html_escape(r)
            ));
        }
    }

    log
}
