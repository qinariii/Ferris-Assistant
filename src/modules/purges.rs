use teloxide::prelude::*;

use crate::utils::permissions;

pub async fn purge(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_delete(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You don't have permission to delete messages.")
            .await?;
        return Ok(());
    }

    if !permissions::can_bot_delete(&bot, chat_id).await {
        bot.send_message(chat_id, "❌ I don't have permission to delete messages.")
            .await?;
        return Ok(());
    }

    let reply = match msg.reply_to_message() {
        Some(r) => r,
        None => {
            bot.send_message(chat_id, "❌ Reply to a message to start purging from.")
                .await?;
            return Ok(());
        }
    };

    let from_id = reply.id.0;
    let to_id = msg.id.0;

    if from_id >= to_id {
        bot.send_message(chat_id, "❌ Invalid message range.")
            .await?;
        return Ok(());
    }

    let mut deleted = 0;
    let mut failed = 0;

    let msg_ids: Vec<i32> = (from_id..=to_id).collect();
    for chunk in msg_ids.chunks(20) {
        for &msg_id in chunk {
            match bot
                .delete_message(chat_id, teloxide::types::MessageId(msg_id))
                .await
            {
                Ok(_) => deleted += 1,
                Err(_) => failed += 1,
            }
        }
        if chunk.len() == 20 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    let status = bot
        .send_message(
            chat_id,
            format!("🧹 Purge complete!\n✅ Deleted: {}\n❌ Failed: {}", deleted, failed),
        )
        .await?;

    // Auto-delete the status message after 5 seconds
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        bot.delete_message(chat_id, status.id).await.ok();
    });

    Ok(())
}

pub async fn del(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_delete(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You don't have permission to delete messages.")
            .await?;
        return Ok(());
    }

    if !permissions::can_bot_delete(&bot, chat_id).await {
        bot.send_message(chat_id, "❌ I don't have permission to delete messages.")
            .await?;
        return Ok(());
    }

    let reply = match msg.reply_to_message() {
        Some(r) => r,
        None => {
            bot.send_message(chat_id, "❌ Reply to a message to delete it.")
                .await?;
            return Ok(());
        }
    };

    bot.delete_message(chat_id, reply.id).await.ok();
    bot.delete_message(chat_id, msg.id).await.ok();
    Ok(())
}
