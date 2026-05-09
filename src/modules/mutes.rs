use teloxide::prelude::*;
use teloxide::types::{ChatPermissions, ParseMode};

use crate::config::AppConfig;
use crate::db;
use crate::keyboards::inline;
use crate::modules::chatpermissions;
use crate::utils::{extraction, formatting, i18n, permissions};

pub async fn mute(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_restrict(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "mutes-no-restrict-perm", None))
            .await?;
        return Ok(());
    }

    if !permissions::can_bot_restrict(&bot, chat_id).await {
        bot.send_message(chat_id, i18n::t(&lang, "mutes-bot-no-restrict", None))
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
            bot.send_message(chat_id, i18n::t(&lang, "mutes-no-user", None))
                .await?;
            return Ok(());
        }
    };

    if permissions::is_user_ban_protected(&bot, chat_id, target_id, &cfg).await {
        bot.send_message(chat_id, i18n::t(&lang, "mutes-cant-mute-admin", None))
            .await?;
        return Ok(());
    }

    let perms = ChatPermissions::empty();

    match bot.restrict_chat_member(chat_id, target_id, perms).await {
        Ok(_) => {
            let member = bot.get_chat_member(chat_id, target_id).await.ok();
            let name = member
                .map(|m| m.user.first_name.clone())
                .unwrap_or_else(|| target_id.to_string());
            let escaped_name = formatting::html_escape(&name);
            let admin_name = formatting::user_display_name(from);

            let mut text = i18n::t(&lang, "mutes-mute-success", Some(&[("name", &escaped_name)]));
            if let Some(ref r) = reason {
                let escaped_r = formatting::html_escape(r);
                text.push_str(&format!("\n{}", i18n::t(&lang, "mutes-mute-reason", Some(&[("reason", &escaped_r)]))));
            }
            text.push_str(&format!("\n{}", i18n::t(&lang, "mutes-mute-by", Some(&[("admin", &admin_name)]))));

            bot.send_message(chat_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(inline::mute_keyboard(target_id.0))
                .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "mutes-mute-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}

pub async fn unmute(bot: Bot, msg: Message, _cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_restrict(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "mutes-unmute-no-perm", None))
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
            bot.send_message(chat_id, i18n::t(&lang, "mutes-unmute-no-user", None))
                .await?;
            return Ok(());
        }
    };

    let perms = chatpermissions::resolve_unmute_permissions(&bot, chat_id).await;

    match bot.restrict_chat_member(chat_id, target_id, perms).await {
        Ok(_) => {
            bot.send_message(chat_id, i18n::t(&lang, "mutes-unmute-success", None))
                .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "mutes-unmute-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}

pub async fn tmute(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::can_user_restrict(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "mutes-no-restrict-perm", None))
            .await?;
        return Ok(());
    }

    if !permissions::can_bot_restrict(&bot, chat_id).await {
        bot.send_message(chat_id, i18n::t(&lang, "mutes-bot-no-restrict", None))
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
            bot.send_message(chat_id, i18n::t(&lang, "mutes-tmute-usage", None))
                .await?;
            return Ok(());
        }
    };

    if permissions::is_user_ban_protected(&bot, chat_id, target_id, &cfg).await {
        bot.send_message(chat_id, i18n::t(&lang, "mutes-cant-mute-admin", None))
            .await?;
        return Ok(());
    }

    // Parse time from reason string
    let time_str = time_reason.as_deref().unwrap_or("");
    let parts: Vec<&str> = time_str.splitn(2, ' ').collect();
    let duration = formatting::parse_duration(parts.first().unwrap_or(&""));

    let duration_secs = match duration {
        Some(d) => d,
        None => {
            bot.send_message(chat_id, i18n::t(&lang, "mutes-tmute-invalid-time", None))
                .await?;
            return Ok(());
        }
    };

    let until_date = chrono::Utc::now() + chrono::Duration::seconds(duration_secs as i64);
    let perms = ChatPermissions::empty();

    match bot
        .restrict_chat_member(chat_id, target_id, perms)
        .until_date(until_date)
        .await
    {
        Ok(_) => {
            let time_text = parts.first().unwrap_or(&"");
            let reason = if parts.len() > 1 { Some(parts[1]) } else { None };

            let mut text = i18n::t(&lang, "mutes-tmute-success", Some(&[("time", time_text)]));
            if let Some(r) = reason {
                let escaped_r = formatting::html_escape(r);
                text.push_str(&format!("\n{}", i18n::t(&lang, "mutes-mute-reason", Some(&[("reason", &escaped_r)]))));
            }

            bot.send_message(chat_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(inline::mute_keyboard(target_id.0))
                .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "mutes-tmute-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}

pub async fn unmute_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let data = q.data.as_deref().unwrap_or("");
    let user_id_str = data.strip_prefix("unmute_").unwrap_or("");
    let user_id: u64 = match user_id_str.parse() {
        Ok(id) => id,
        Err(_) => return Ok(()),
    };

    let chat_id = msg.chat().id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    if !permissions::can_user_restrict(&bot, chat_id, q.from.id).await {
        bot.answer_callback_query(q.id.clone())
            .text(i18n::t(&lang, "mutes-unmute-cb-no-perm", None))
            .await?;
        return Ok(());
    }

    let perms = chatpermissions::resolve_unmute_permissions(&bot, chat_id).await;
    match bot
        .restrict_chat_member(chat_id, UserId(user_id), perms)
        .await
    {
        Ok(_) => {
            bot.answer_callback_query(q.id.clone())
                .text(i18n::t(&lang, "mutes-unmute-cb-success", None))
                .await?;
            bot.edit_message_text(msg.chat().id, msg.id(), i18n::t(&lang, "mutes-unmute-done", None))
                .await?;
        }
        Err(_) => {
            bot.answer_callback_query(q.id.clone())
                .text(i18n::t(&lang, "mutes-unmute-cb-failed", None))
                .await?;
        }
    }
    Ok(())
}
