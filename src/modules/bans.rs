use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::config::AppConfig;
use crate::db;
use crate::keyboards::inline;
use crate::utils::{extraction, formatting, i18n, permissions, LogErrExt};

/// Kick a user safely: ban lalu unban dengan retry, tanpa sleep arbitrary.
/// Ini menggantikan pola `ban -> sleep(1) -> unban` yang rawan race condition.
async fn safe_kick(bot: &Bot, chat_id: ChatId, user_id: UserId) -> Result<(), teloxide::RequestError> {
    bot.ban_chat_member(chat_id, user_id).await?;

    // Retry unban hingga 3x dengan backoff singkat
    for attempt in 0u32..3 {
        match bot.unban_chat_member(chat_id, user_id).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt == 2 {
                    return Err(e);
                }
                // Backoff eksponensial: 200ms, 400ms
                let delay = tokio::time::Duration::from_millis(200 * (1u64 << attempt));
                tokio::time::sleep(delay).await;
            }
        }
    }
    Ok(())
}

pub async fn ban(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_restrict(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-no-restrict-perm", None))
            .await?;
        return Ok(());
    }

    if !permissions::can_bot_restrict(&bot, chat_id).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-bot-no-restrict", None))
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
            bot.send_message(chat_id, i18n::t(&lang, "bans-no-user", None))
                .await?;
            return Ok(());
        }
    };

    if permissions::is_user_ban_protected(&bot, chat_id, target_id, &cfg).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-cant-ban-admin", None))
            .await?;
        return Ok(());
    }

    // Ensure chat exists in DB
    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("ban::upsert_chat");

    match bot.ban_chat_member(chat_id, target_id).await {
        Ok(_) => {
            let member = bot.get_chat_member(chat_id, target_id).await.ok();
            let name = member
                .map(|m| m.user.first_name.clone())
                .unwrap_or_else(|| target_id.to_string());
            let escaped_name = formatting::html_escape(&name);
            let admin_name = formatting::user_display_name(from);

            let mut text = i18n::t(&lang, "bans-ban-success", Some(&[("name", &escaped_name)]));
            if let Some(ref r) = reason {
                let escaped_r = formatting::html_escape(r);
                text.push_str(&format!("\n{}", i18n::t(&lang, "bans-ban-reason", Some(&[("reason", &escaped_r)]))));
            }
            text.push_str(&format!("\n{}", i18n::t(&lang, "bans-ban-by", Some(&[("admin", &admin_name)]))));

            bot.send_message(chat_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(inline::unban_keyboard(target_id.0))
                .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "bans-ban-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}

pub async fn unban(bot: Bot, msg: Message, _cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_restrict(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-unban-no-perm", None))
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
            bot.send_message(chat_id, i18n::t(&lang, "bans-unban-no-user", None))
                .await?;
            return Ok(());
        }
    };

    match bot.unban_chat_member(chat_id, target_id).await {
        Ok(_) => {
            bot.send_message(chat_id, i18n::t(&lang, "bans-unban-success", None))
                .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "bans-unban-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}

pub async fn kick(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_restrict(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-kick-no-perm", None))
            .await?;
        return Ok(());
    }

    if !permissions::can_bot_restrict(&bot, chat_id).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-kick-bot-no-perm", None))
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
            bot.send_message(chat_id, i18n::t(&lang, "bans-kick-no-user", None))
                .await?;
            return Ok(());
        }
    };

    if permissions::is_user_ban_protected(&bot, chat_id, target_id, &cfg).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-cant-kick-admin", None))
            .await?;
        return Ok(());
    }

    // FIX: Gunakan safe_kick menggantikan ban -> sleep(1) -> unban
    match safe_kick(&bot, chat_id, target_id).await {
        Ok(_) => {
            let mut text = i18n::t(&lang, "bans-kick-success", None);
            if let Some(ref r) = reason {
                let escaped_r = formatting::html_escape(r);
                text.push_str(&format!("\n{}", i18n::t(&lang, "bans-ban-reason", Some(&[("reason", &escaped_r)]))));
            }
            bot.send_message(chat_id, text)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "bans-kick-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}

pub async fn dban(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_restrict(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-no-restrict-perm", None))
            .await?;
        return Ok(());
    }

    let reply = match msg.reply_to_message() {
        Some(r) => r,
        None => {
            bot.send_message(chat_id, i18n::t(&lang, "bans-dban-reply", None))
                .await?;
            return Ok(());
        }
    };

    let target_id = match reply.from.as_ref() {
        Some(u) => u.id,
        None => return Ok(()),
    };

    if permissions::is_user_ban_protected(&bot, chat_id, target_id, &cfg).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-cant-ban-admin", None))
            .await?;
        return Ok(());
    }

    // Delete the replied message
    bot.delete_message(chat_id, reply.id).await.ok();

    match bot.ban_chat_member(chat_id, target_id).await {
        Ok(_) => {
            bot.send_message(chat_id, i18n::t(&lang, "bans-dban-success", None))
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "bans-ban-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}

pub async fn dkick(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_restrict(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-kick-no-perm", None))
            .await?;
        return Ok(());
    }

    let reply = match msg.reply_to_message() {
        Some(r) => r,
        None => {
            bot.send_message(chat_id, i18n::t(&lang, "bans-dkick-reply", None))
                .await?;
            return Ok(());
        }
    };

    let target_id = match reply.from.as_ref() {
        Some(u) => u.id,
        None => return Ok(()),
    };

    if permissions::is_user_ban_protected(&bot, chat_id, target_id, &cfg).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-cant-kick-admin", None))
            .await?;
        return Ok(());
    }

    // Delete the replied message
    bot.delete_message(chat_id, reply.id).await.ok();

    // FIX: Gunakan safe_kick menggantikan ban -> sleep(1) -> unban
    match safe_kick(&bot, chat_id, target_id).await {
        Ok(_) => {
            bot.send_message(chat_id, i18n::t(&lang, "bans-dkick-success", None))
                .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "bans-dkick-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}

pub async fn tban(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_restrict(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-no-restrict-perm", None))
            .await?;
        return Ok(());
    }

    if !permissions::can_bot_restrict(&bot, chat_id).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-bot-no-restrict", None))
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

    let (target_user, time_reason) = extraction::extract_user_and_reason(&bot, &msg, &args, &pool).await;

    let target_id = match target_user {
        Some(id) => id,
        None => {
            bot.send_message(chat_id, i18n::t(&lang, "bans-tban-usage", None))
                .await?;
            return Ok(());
        }
    };

    if permissions::is_user_ban_protected(&bot, chat_id, target_id, &cfg).await {
        bot.send_message(chat_id, i18n::t(&lang, "bans-cant-ban-admin", None))
            .await?;
        return Ok(());
    }

    let time_str = time_reason.as_deref().unwrap_or("");
    let parts: Vec<&str> = time_str.splitn(2, ' ').collect();
    let duration_secs = match formatting::parse_duration(parts.first().unwrap_or(&"")) {
        Some(d) => d,
        None => {
            bot.send_message(chat_id, i18n::t(&lang, "bans-tban-invalid-time", None))
                .await?;
            return Ok(());
        }
    };

    let until_date = chrono::Utc::now() + chrono::Duration::seconds(duration_secs as i64);

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("tban::upsert_chat");

    match bot
        .ban_chat_member(chat_id, target_id)
        .until_date(until_date)
        .await
    {
        Ok(_) => {
            let time_text = parts.first().unwrap_or(&"");
            let reason = if parts.len() > 1 { Some(parts[1]) } else { None };

            let mut text = i18n::t(&lang, "bans-tban-success", Some(&[("time", time_text)]));
            if let Some(r) = reason {
                let escaped_r = formatting::html_escape(r);
                text.push_str(&format!("\n{}", i18n::t(&lang, "bans-ban-reason", Some(&[("reason", &escaped_r)]))));
            }

            bot.send_message(chat_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(inline::unban_keyboard(target_id.0))
                .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "bans-tban-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}

pub async fn unban_callback(bot: Bot, q: CallbackQuery, _cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let data = q.data.as_deref().unwrap_or("");
    let user_id_str = data.strip_prefix("unban_").unwrap_or("");
    let user_id: u64 = match user_id_str.parse() {
        Ok(id) => id,
        Err(_) => return Ok(()),
    };

    let chat_id = msg.chat().id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    if !permissions::can_user_restrict(&bot, chat_id, q.from.id).await {
        bot.answer_callback_query(q.id.clone())
            .text(i18n::t(&lang, "bans-unban-cb-no-perm", None))
            .await?;
        return Ok(());
    }

    match bot.unban_chat_member(chat_id, UserId(user_id)).await {
        Ok(_) => {
            bot.answer_callback_query(q.id.clone())
                .text(i18n::t(&lang, "bans-unban-cb-success", None))
                .await?;
            bot.edit_message_text(msg.chat().id, msg.id(), i18n::t(&lang, "bans-unban-done", None))
                .await?;
        }
        Err(_) => {
            bot.answer_callback_query(q.id.clone())
                .text(i18n::t(&lang, "bans-unban-cb-failed", None))
                .await?;
        }
    }
    Ok(())
}