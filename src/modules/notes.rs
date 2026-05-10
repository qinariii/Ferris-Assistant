use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::db;
use crate::utils::{formatting, permissions};

pub async fn save(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to save notes.")
            .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let args: Vec<&str> = text.splitn(3, ' ').collect();

    if args.len() < 3 {
        // Check if replying to a message
        if let Some(reply) = msg.reply_to_message() {
            if args.len() >= 2 {
                let name = args[1];
                let content = reply.text().unwrap_or("").to_string();
                if content.is_empty() {
                    bot.send_message(chat_id, "❌ The replied message has no text content.")
                        .await?;
                    return Ok(());
                }

                let chat_name = msg.chat.title().unwrap_or("Private");
                db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();
                db::queries::save_note(&pool, chat_id.0, name, &content)
                    .await
                    .ok();

                bot.send_message(
                    chat_id,
                    format!("✅ Note <b>{}</b> saved!", formatting::html_escape(name)),
                )
                .parse_mode(ParseMode::Html)
                .await?;
                return Ok(());
            }
        }

        bot.send_message(chat_id, "❌ Usage: /save <name> <content> or reply to a message with /save <name>")
            .await?;
        return Ok(());
    }

    let name = args[1];
    let content = args[2];

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();
    db::queries::save_note(&pool, chat_id.0, name, content)
        .await
        .ok();

    bot.send_message(
        chat_id,
        format!("✅ Note <b>{}</b> saved!", formatting::html_escape(name)),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn get(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    if args.is_empty() {
        bot.send_message(chat_id, "❌ Usage: /get <name>")
            .await?;
        return Ok(());
    }

    let name = &args[0];
    match db::queries::get_note(&pool, chat_id.0, name).await {
        Ok(Some(note)) => {
            bot.send_message(chat_id, &note.content)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        _ => {
            bot.send_message(chat_id, format!("❌ Note <b>{}</b> not found.", formatting::html_escape(name)))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }
    Ok(())
}

/// Handle #notename hash trigger
pub async fn hash_get(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or("");

    if !text.starts_with('#') || text.len() < 2 {
        return Ok(());
    }

    let name = &text[1..].split_whitespace().next().unwrap_or("");
    if name.is_empty() {
        return Ok(());
    }

    if let Ok(Some(note)) = db::queries::get_note(&pool, chat_id.0, name).await {
        bot.send_message(chat_id, &note.content)
            .parse_mode(ParseMode::Html)
            .await?;
    }
    Ok(())
}

pub async fn notes_list(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    let notes = db::queries::get_all_notes(&pool, chat_id.0)
        .await
        .unwrap_or_default();

    if notes.is_empty() {
        bot.send_message(chat_id, "📝 No notes saved in this chat.")
            .await?;
        return Ok(());
    }

    let mut text = "<b>📝 Notes in this chat:</b>\n".to_string();
    for note in &notes {
        text.push_str(&format!("\n• <code>#{}</code>", formatting::html_escape(&note.name)));
    }
    text.push_str("\n\n<i>Use /get &lt;name&gt; or #name to retrieve a note.</i>");

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn clear(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to delete notes.")
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
        bot.send_message(chat_id, "❌ Usage: /clear <name>")
            .await?;
        return Ok(());
    }

    let name = &args[0];
    match db::queries::delete_note(&pool, chat_id.0, name).await {
        Ok(true) => {
            bot.send_message(
                chat_id,
                format!("✅ Note <b>{}</b> deleted!", formatting::html_escape(name)),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {
            bot.send_message(
                chat_id,
                format!("❌ Note <b>{}</b> not found.", formatting::html_escape(name)),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }
    Ok(())
}
