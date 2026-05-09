use teloxide::prelude::*;

use crate::utils::permissions;

pub async fn pin(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_pin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You don't have permission to pin messages.")
            .await?;
        return Ok(());
    }

    if !permissions::can_bot_pin(&bot, chat_id).await {
        bot.send_message(chat_id, "❌ I don't have permission to pin messages.")
            .await?;
        return Ok(());
    }

    let reply = match msg.reply_to_message() {
        Some(r) => r,
        None => {
            bot.send_message(chat_id, "❌ Reply to a message to pin it.")
                .await?;
            return Ok(());
        }
    };

    // Check if "silent" or "quiet" in args
    let text = msg.text().unwrap_or("");
    let silent = text.contains("silent") || text.contains("quiet");

    match bot
        .pin_chat_message(chat_id, reply.id)
        .disable_notification(silent)
        .await
    {
        Ok(_) => {
            bot.send_message(chat_id, "📌 Message pinned!")
                .await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Failed to pin: {}", e))
                .await?;
        }
    }
    Ok(())
}

pub async fn unpin(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_pin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You don't have permission to unpin messages.")
            .await?;
        return Ok(());
    }

    match bot.unpin_chat_message(chat_id).await {
        Ok(_) => {
            bot.send_message(chat_id, "📌 Message unpinned!")
                .await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Failed to unpin: {}", e))
                .await?;
        }
    }
    Ok(())
}

pub async fn unpinall(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_owner(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ Only the group owner can unpin all messages.")
            .await?;
        return Ok(());
    }

    match bot.unpin_all_chat_messages(chat_id).await {
        Ok(_) => {
            bot.send_message(chat_id, "📌 All messages unpinned!")
                .await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Failed to unpin all: {}", e))
                .await?;
        }
    }
    Ok(())
}
