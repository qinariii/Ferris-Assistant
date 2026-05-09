use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::db;
use crate::utils::{formatting, permissions};

pub async fn rules(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();

    let rules_text = db::queries::get_rules(&pool, chat_id.0)
        .await
        .unwrap_or_default();

    if rules_text.is_empty() {
        bot.send_message(chat_id, "📏 No rules set for this chat yet.\nAdmins can use /setrules to set them.")
            .await?;
    } else {
        let text = format!(
            "<b>📏 Rules for {}:</b>\n\n{}",
            formatting::html_escape(chat_name),
            rules_text
        );
        bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .await?;
    }
    Ok(())
}

pub async fn setrules(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to set rules.")
            .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let content = text.splitn(2, ' ').nth(1);

    let rules_text = if let Some(content) = content {
        content.to_string()
    } else if let Some(reply) = msg.reply_to_message() {
        reply.text().unwrap_or("").to_string()
    } else {
        bot.send_message(chat_id, "❌ Usage: /setrules <text> or reply to a message with /setrules")
            .await?;
        return Ok(());
    };

    if rules_text.is_empty() {
        bot.send_message(chat_id, "❌ Rules text cannot be empty.")
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();
    db::queries::set_rules(&pool, chat_id.0, &rules_text).await.ok();

    bot.send_message(chat_id, "✅ Rules updated successfully!")
        .await?;
    Ok(())
}

pub async fn clearrules(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to clear rules.")
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();
    db::queries::set_rules(&pool, chat_id.0, "").await.ok();

    bot.send_message(chat_id, "✅ Rules have been cleared!")
        .await?;
    Ok(())
}

pub async fn rules_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let _msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let data = q.data.as_deref().unwrap_or("");
    let chat_id_str = data.strip_prefix("rules_").unwrap_or("");
    let target_chat_id: i64 = match chat_id_str.parse() {
        Ok(id) => id,
        Err(_) => return Ok(()),
    };

    let rules_text = db::queries::get_rules(&pool, target_chat_id)
        .await
        .unwrap_or_default();

    if rules_text.is_empty() {
        bot.answer_callback_query(q.id.clone())
            .text("No rules set for this chat.")
            .await?;
    } else {
        bot.answer_callback_query(q.id.clone()).await?;
        bot.send_message(ChatId(q.from.id.0 as i64), format!("<b>📏 Rules:</b>\n\n{}", rules_text))
            .parse_mode(ParseMode::Html)
            .await
            .ok();
    }
    Ok(())
}
