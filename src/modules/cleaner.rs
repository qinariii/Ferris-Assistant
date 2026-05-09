use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::config::AppConfig;
use crate::db;
use crate::utils::permissions;


pub async fn cleanservice(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
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

    let settings = db::queries::get_cleaner_settings(&pool, chat_id.0)
        .await
        .unwrap_or(db::models::CleanerSettings {
            chat_id: chat_id.0,
            clean_service: false,
            clean_bluetext: false,
        });

    if args.is_empty() {
        let status = if settings.clean_service { "enabled ✅" } else { "disabled ❌" };
        bot.send_message(
            chat_id,
            format!(
                "🧹 <b>Clean Service Messages:</b> {}\n\nUsage: /cleanservice <on/off>",
                status
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    match args[0].to_lowercase().as_str() {
        "on" | "yes" | "enable" => {
            db::queries::set_clean_service(&pool, chat_id.0, true).await.ok();
            bot.send_message(chat_id, "✅ Service message cleaning enabled! Join/leave and other service messages will be auto-deleted.")
                .await?;
        }
        "off" | "no" | "disable" => {
            db::queries::set_clean_service(&pool, chat_id.0, false).await.ok();
            bot.send_message(chat_id, "❌ Service message cleaning disabled.")
                .await?;
        }
        _ => {
            bot.send_message(chat_id, "❌ Usage: /cleanservice <on/off>")
                .await?;
        }
    }
    Ok(())
}


pub async fn cleanbluetext(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
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

    let settings = db::queries::get_cleaner_settings(&pool, chat_id.0)
        .await
        .unwrap_or(db::models::CleanerSettings {
            chat_id: chat_id.0,
            clean_service: false,
            clean_bluetext: false,
        });

    if args.is_empty() {
        let status = if settings.clean_bluetext { "enabled ✅" } else { "disabled ❌" };
        bot.send_message(
            chat_id,
            format!(
                "🧹 <b>Clean Blue Text:</b> {}\n\nUsage: /cleanbluetext <on/off>",
                status
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    match args[0].to_lowercase().as_str() {
        "on" | "yes" | "enable" => {
            db::queries::set_clean_bluetext(&pool, chat_id.0, true).await.ok();
            bot.send_message(chat_id, "✅ Blue text cleaning enabled! Unrecognized bot commands will be deleted.")
                .await?;
        }
        "off" | "no" | "disable" => {
            db::queries::set_clean_bluetext(&pool, chat_id.0, false).await.ok();
            bot.send_message(chat_id, "❌ Blue text cleaning disabled.")
                .await?;
        }
        _ => {
            bot.send_message(chat_id, "❌ Usage: /cleanbluetext <on/off>")
                .await?;
        }
    }
    Ok(())
}


/// Delete service messages (join, left, pin, etc.) if cleaning is enabled.
pub async fn check_service_message(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // Only handle in groups
    if msg.chat.is_private() {
        return Ok(());
    }

    let settings = db::queries::get_cleaner_settings(&pool, chat_id.0)
        .await
        .unwrap_or(db::models::CleanerSettings {
            chat_id: chat_id.0,
            clean_service: false,
            clean_bluetext: false,
        });

    // Clean service messages
    if settings.clean_service {
        let is_service = msg.new_chat_members().is_some()
            || msg.left_chat_member().is_some()
            || msg.pinned_message().is_some()
            || msg.new_chat_title().is_some()
            || msg.new_chat_photo().is_some()
            || msg.delete_chat_photo().is_some()
            || msg.group_chat_created().is_some()
            || msg.super_group_chat_created().is_some();

        if is_service {
            bot.delete_message(chat_id, msg.id).await.ok();
            return Ok(());
        }
    }

    // Clean blue text (unrecognized bot commands)
    if settings.clean_bluetext {
        if let Some(text) = msg.text() {
            // If it starts with / and contains @bot_username or is an unrecognized command
            if text.starts_with('/') && !text.starts_with("//") {
                // Only delete if it looks like an unknown command attempt
                // (bot commands that don't match any known handler will reach the general message handler)
                let cmd = text.split_whitespace().next().unwrap_or("");
                if cmd.contains('@') {
                    // Command directed at another bot — delete if bluetext cleaning is on
                    let parts: Vec<&str> = cmd.splitn(2, '@').collect();
                    if parts.len() == 2 {
                        let target_bot = parts[1].to_lowercase();
                        let my_username = cfg.bot_username.to_lowercase();

                        // Only delete commands not directed at us
                        if !target_bot.is_empty() && target_bot != my_username {
                            bot.delete_message(chat_id, msg.id).await.ok();
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
