use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::config::AppConfig;
use crate::db;
use crate::utils::formatting::uid_to_i64;

/// Automatically track users and chats when messages are received
pub async fn log_users(_bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    // Track the sender
    if let Some(from) = msg.from.as_ref() {
        let username = from.username.as_deref().unwrap_or("");
        let last_name = from.last_name.as_deref().unwrap_or("");
        db::queries::upsert_user(&pool, uid_to_i64(from.id), username, &from.first_name, last_name)
            .await
            .ok();

        // Track user-chat association (only in groups)
        if msg.chat.is_group() || msg.chat.is_supergroup() {
            let chat_name = msg.chat.title().unwrap_or("");
            db::queries::upsert_chat(&pool, msg.chat.id.0, chat_name).await.ok();
            db::queries::track_user_chat(&pool, uid_to_i64(from.id), msg.chat.id.0)
                .await
                .ok();
        }
    }

    // Track replied-to user
    if let Some(reply) = msg.reply_to_message() {
        if let Some(reply_user) = reply.from.as_ref() {
            let username = reply_user.username.as_deref().unwrap_or("");
            let last_name = reply_user.last_name.as_deref().unwrap_or("");
            db::queries::upsert_user(
                &pool,
                uid_to_i64(reply_user.id),
                username,
                &reply_user.first_name,
                last_name,
            )
            .await
            .ok();
        }
    }

    Ok(())
}

pub async fn stats(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !cfg.is_sudo(uid_to_i64(from.id)) {
        bot.send_message(msg.chat.id, "❌ Only bot owner/sudo can view stats.")
            .await?;
        return Ok(());
    }

    let user_count = db::queries::get_user_count(&pool).await.unwrap_or(0);
    let chat_count = db::queries::get_chat_count(&pool).await.unwrap_or(0);

    let text = format!(
        "📊 <b>Bot Statistics</b>\n\n\
        👤 <b>Users:</b> {}\n\
        💬 <b>Chats:</b> {}",
        user_count, chat_count,
    );

    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn chatlist(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !cfg.is_sudo(uid_to_i64(from.id)) {
        bot.send_message(msg.chat.id, "❌ Only bot owner/sudo can view chat list.")
            .await?;
        return Ok(());
    }

    let chats = db::queries::get_all_chats(&pool).await.unwrap_or_default();

    if chats.is_empty() {
        bot.send_message(msg.chat.id, "📋 No known chats yet.")
            .await?;
        return Ok(());
    }

    let mut text = format!("📋 <b>Known Chats ({}):</b>\n", chats.len());
    for (i, chat) in chats.iter().enumerate().take(50) {
        let name = if chat.chat_name.is_empty() {
            chat.chat_id.to_string()
        } else {
            crate::utils::formatting::html_escape(&chat.chat_name)
        };
        text.push_str(&format!(
            "\n{}. {} (<code>{}</code>)",
            i + 1,
            name,
            chat.chat_id,
        ));
    }

    if chats.len() > 50 {
        text.push_str(&format!("\n\n<i>...and {} more</i>", chats.len() - 50));
    }

    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
