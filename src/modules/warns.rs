use teloxide::prelude::*;
use teloxide::types::{ChatPermissions, ParseMode};

use crate::config::AppConfig;
use crate::db;
use crate::keyboards::inline;
use crate::utils::{extraction, formatting, formatting::uid_to_i64, i18n, permissions, LogErrExt};

pub async fn warn(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_restrict(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "warns-no-perm", None))
            .await?;
        return Ok(());
    }

    if !permissions::can_bot_restrict(&bot, chat_id).await {
        bot.send_message(chat_id, i18n::t(&lang, "warns-bot-no-restrict", None))
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

    let (target_user, reason) = extraction::extract_user_and_reason(&bot, &msg, &args, &pool).await;

    let target_id = match target_user {
        Some(id) => id,
        None => {
            bot.send_message(chat_id, i18n::t(&lang, "warns-no-user", None))
                .await?;
            return Ok(());
        }
    };

    if permissions::is_user_ban_protected(&bot, chat_id, target_id, &cfg).await {
        bot.send_message(chat_id, i18n::t(&lang, "warns-cant-warn-admin", None))
            .await?;
        return Ok(());
    }

    // Ensure chat exists in DB
    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("warn::upsert_chat");

    let reason_str = reason.as_deref().unwrap_or("No reason given");
    let warn_count = db::queries::add_warn(&pool, chat_id.0, uid_to_i64(target_id), reason_str, uid_to_i64(from.id))
        .await
        .unwrap_or(0);

    let chat_data = db::queries::get_chat(&pool, chat_id.0).await.ok().flatten();
    let warn_limit = chat_data.as_ref().map(|c| c.warn_limit).unwrap_or(3);
    let warn_mode = chat_data
        .as_ref()
        .map(|c| c.warn_mode.clone())
        .unwrap_or_else(|| "ban".to_string());

    let member = bot.get_chat_member(chat_id, target_id).await.ok();
    let name = member
        .map(|m| m.user.first_name.clone())
        .unwrap_or_else(|| target_id.to_string());
    let escaped_name = formatting::html_escape(&name);
    let count_str = warn_count.to_string();
    let limit_str = warn_limit.to_string();

    if warn_count >= warn_limit as i64 {
        // User exceeded warn limit - take action
        let action_text = match warn_mode.as_str() {
            "ban" => {
                bot.ban_chat_member(chat_id, target_id).await.ok();
                "banned"
            }
            "kick" => {
                bot.ban_chat_member(chat_id, target_id).await.ok();
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                bot.unban_chat_member(chat_id, target_id).await.ok();
                "kicked"
            }
            "mute" => {
                let perms = ChatPermissions::empty();
                bot.restrict_chat_member(chat_id, target_id, perms)
                    .await
                    .ok();
                "muted"
            }
            _ => "banned",
        };

        // Reset warns after action
        db::queries::reset_warns(&pool, chat_id.0, uid_to_i64(target_id))
            .await
            .log_err("warn::reset_warns");

        let text = i18n::t(&lang, "warns-exceeded", Some(&[
            ("name", &escaped_name),
            ("action", action_text),
            ("count", &count_str),
            ("limit", &limit_str),
        ]));

        bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .await?;
    } else {
        let escaped_reason = formatting::html_escape(reason_str);
        let mut text = i18n::t(&lang, "warns-warn-success", Some(&[
            ("name", &escaped_name),
            ("count", &count_str),
            ("limit", &limit_str),
        ]));
        text.push_str(&format!("\n{}", i18n::t(&lang, "warns-warn-reason", Some(&[("reason", &escaped_reason)]))));

        bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(inline::warn_keyboard(target_id.0))
            .await?;
    }
    Ok(())
}

pub async fn warns(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    let (target_user, _) = extraction::extract_user_and_reason(&bot, &msg, &args, &pool).await;

    let target_id = match target_user {
        Some(id) => id,
        None => {
            // If no user specified, show warns of the sender
            match msg.from.as_ref() {
                Some(u) => u.id,
                None => return Ok(()),
            }
        }
    };

    let user_warns = db::queries::get_warns(&pool, chat_id.0, uid_to_i64(target_id))
        .await
        .unwrap_or_default();

    if user_warns.is_empty() {
        bot.send_message(chat_id, i18n::t(&lang, "warns-none", None))
            .await?;
        return Ok(());
    }

    let chat_data = db::queries::get_chat(&pool, chat_id.0).await.ok().flatten();
    let warn_limit = chat_data.as_ref().map(|c| c.warn_limit).unwrap_or(3);
    let count_str = user_warns.len().to_string();
    let limit_str = warn_limit.to_string();

    let mut text = i18n::t(&lang, "warns-header", Some(&[("count", &count_str), ("limit", &limit_str)]));
    text.push('\n');
    for (i, w) in user_warns.iter().enumerate() {
        text.push_str(&format!(
            "\n{}. {} <i>({})</i>",
            i + 1,
            formatting::html_escape(&w.reason),
            w.created_at
        ));
    }

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn resetwarns(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_restrict(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "warns-reset-no-perm", None))
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

    let (target_user, _) = extraction::extract_user_and_reason(&bot, &msg, &args, &pool).await;

    let target_id = match target_user {
        Some(id) => id,
        None => {
            bot.send_message(chat_id, i18n::t(&lang, "warns-reset-no-user", None))
                .await?;
            return Ok(());
        }
    };

    db::queries::reset_warns(&pool, chat_id.0, uid_to_i64(target_id))
        .await
        .log_err("warns::reset_warns");

    bot.send_message(chat_id, i18n::t(&lang, "warns-reset-success", None))
        .await?;
    Ok(())
}

pub async fn setwarnlimit(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "warns-need-admin", None))
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
        bot.send_message(chat_id, i18n::t(&lang, "warns-limit-usage", None))
            .await?;
        return Ok(());
    }

    let limit: i32 = match args[0].parse() {
        Ok(n) if n >= 1 && n <= 100 => n,
        _ => {
            bot.send_message(chat_id, i18n::t(&lang, "warns-limit-invalid", None))
                .await?;
            return Ok(());
        }
    };

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("warns::upsert_chat");
    db::queries::set_warn_limit(&pool, chat_id.0, limit).await.log_err("warns::set_warn_limit");

    let limit_str = limit.to_string();
    bot.send_message(chat_id, i18n::t(&lang, "warns-limit-set", Some(&[("limit", &limit_str)])))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn setwarnmode(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "warns-need-admin", None))
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
        bot.send_message(chat_id, i18n::t(&lang, "warns-mode-choose", None))
            .reply_markup(inline::warn_mode_keyboard())
            .await?;
        return Ok(());
    }

    let mode = args[0].to_lowercase();
    if !["ban", "kick", "mute"].contains(&mode.as_str()) {
        bot.send_message(chat_id, i18n::t(&lang, "warns-mode-invalid", None))
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("warns::upsert_chat");
    db::queries::set_warn_mode(&pool, chat_id.0, &mode).await.log_err("warns::set_warn_mode");

    bot.send_message(chat_id, i18n::t(&lang, "warns-mode-set", Some(&[("mode", &mode)])))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn rmwarn_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let data = q.data.as_deref().unwrap_or("");
    let user_id_str = data.strip_prefix("rmwarn_").unwrap_or("");
    let user_id: u64 = match user_id_str.parse() {
        Ok(id) => id,
        Err(_) => return Ok(()),
    };

    let chat_id = msg.chat().id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    if !permissions::can_user_restrict(&bot, chat_id, q.from.id).await {
        bot.answer_callback_query(q.id.clone())
            .text(i18n::t(&lang, "warns-remove-no-perm", None))
            .await?;
        return Ok(());
    }

    db::queries::remove_last_warn(&pool, chat_id.0, formatting::u64_to_i64(user_id))
        .await
        .log_err("warns::remove_last_warn");

    bot.answer_callback_query(q.id.clone())
        .text(i18n::t(&lang, "warns-remove-done", None))
        .await?;
    bot.edit_message_text(msg.chat().id, msg.id(), i18n::t(&lang, "warns-remove-last", None))
        .await?;
    Ok(())
}

pub async fn warnmode_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let data = q.data.as_deref().unwrap_or("");
    let mode = data.strip_prefix("warnmode_").unwrap_or("");

    if !["ban", "kick", "mute"].contains(&mode) {
        return Ok(());
    }

    let chat_id = msg.chat().id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    if !permissions::is_user_admin(&bot, chat_id, q.from.id).await {
        bot.answer_callback_query(q.id.clone())
            .text(i18n::t(&lang, "warns-mode-cb-no-perm", None))
            .await?;
        return Ok(());
    }

    db::queries::set_warn_mode(&pool, chat_id.0, mode).await.log_err("warns::set_warn_mode_cb");

    bot.answer_callback_query(q.id.clone())
        .text(i18n::t(&lang, "warns-mode-set", Some(&[("mode", mode)])))
        .await?;
    bot.edit_message_text(
        msg.chat().id,
        msg.id(),
        i18n::t(&lang, "warns-mode-set", Some(&[("mode", mode)])),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}
