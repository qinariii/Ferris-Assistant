use teloxide::prelude::*;
use teloxide::types::{ChatMemberUpdated, ParseMode};

use crate::db;
use crate::utils::{formatting, formatting::uid_to_i64, i18n, permissions, LogErrExt};

/// Handle new chat members (welcome message)
pub async fn welcome_new_member(
    bot: Bot,
    update: ChatMemberUpdated,
    pool: db::Pool,
) -> ResponseResult<()> {
    let chat_id = update.chat.id;
    let new_member = &update.new_chat_member;
    let user = &new_member.user;

    // Only trigger on new members joining
    if !new_member.is_present() || update.old_chat_member.is_present() {
        return Ok(());
    }

    // Don't welcome bots
    if user.is_bot {
        return Ok(());
    }

    let chat_name = update.chat.title().unwrap_or("this chat");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("welcome::upsert_chat");

    let chat_data = db::queries::get_chat(&pool, chat_id.0).await.log_err("welcome::get_chat").flatten();

    if let Some(chat) = chat_data {
        if chat.welcome_enabled {
            let welcome_text = formatting::format_greeting(&chat.welcome_text, user, chat_name);
            bot.send_message(chat_id, welcome_text)
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    // Update user in DB
    db::queries::upsert_user(
        &pool,
        uid_to_i64(user.id),
        user.username.as_deref().unwrap_or(""),
        &user.first_name,
        user.last_name.as_deref().unwrap_or(""),
    )
    .await
    .log_err("welcome::upsert_user");

    Ok(())
}

/// Handle members leaving (goodbye message)
pub async fn goodbye_member(
    bot: Bot,
    update: ChatMemberUpdated,
    pool: db::Pool,
) -> ResponseResult<()> {
    let chat_id = update.chat.id;
    let old_member = &update.old_chat_member;
    let new_member = &update.new_chat_member;
    let user = &new_member.user;

    // Only trigger on members leaving
    if new_member.is_present() || !old_member.is_present() {
        return Ok(());
    }

    // Don't say goodbye to bots
    if user.is_bot {
        return Ok(());
    }

    let chat_name = update.chat.title().unwrap_or("this chat");
    let chat_data = db::queries::get_chat(&pool, chat_id.0).await.ok().flatten();

    if let Some(chat) = chat_data {
        if chat.goodbye_enabled {
            let goodbye_text = formatting::format_greeting(&chat.goodbye_text, user, chat_name);
            bot.send_message(chat_id, goodbye_text)
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}

pub async fn welcome_toggle(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "welcome-no-permission", None))
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

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("welcome::upsert_chat");

    if args.is_empty() {
        let chat_data = db::queries::get_chat(&pool, chat_id.0).await.log_err("welcome::get_chat").flatten();
        let enabled = chat_data.map(|c| c.welcome_enabled).unwrap_or(true);
        let key = if enabled { "welcome-status-on" } else { "welcome-status-off" };
        let mut text = i18n::t(&lang, key, None);
        text.push_str(&format!("\n\n{}", i18n::t(&lang, "welcome-usage", None)));
        bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    match args[0].to_lowercase().as_str() {
        "on" | "yes" | "true" | "1" => {
            db::queries::toggle_welcome(&pool, chat_id.0, true).await.log_err("welcome::toggle");
            bot.send_message(chat_id, i18n::t(&lang, "welcome-enabled", None))
                .await?;
        }
        "off" | "no" | "false" | "0" => {
            db::queries::toggle_welcome(&pool, chat_id.0, false).await.log_err("welcome::toggle");
            bot.send_message(chat_id, i18n::t(&lang, "welcome-disabled", None))
                .await?;
        }
        _ => {
            bot.send_message(chat_id, i18n::t(&lang, "welcome-usage", None))
                .await?;
        }
    }
    Ok(())
}

pub async fn set_welcome(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "welcome-no-permission", None))
            .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let content = text.split_once(' ').map(|x| x.1);

    let welcome_text = if let Some(content) = content {
        content.to_string()
    } else if let Some(reply) = msg.reply_to_message() {
        reply.text().unwrap_or("").to_string()
    } else {
        let mut usage = i18n::t(&lang, "welcome-set-usage", None);
        usage.push_str(&format!("\n\n{}", i18n::t(&lang, "welcome-placeholders", None)));
        bot.send_message(chat_id, usage)
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    };

    if welcome_text.is_empty() {
        bot.send_message(chat_id, i18n::t(&lang, "welcome-text-empty", None))
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("welcome::upsert_chat");
    db::queries::set_welcome(&pool, chat_id.0, &welcome_text).await.log_err("welcome::set_welcome");

    bot.send_message(chat_id, i18n::t(&lang, "welcome-updated", None))
        .await?;
    Ok(())
}

pub async fn goodbye_toggle(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "goodbye-no-permission", None))
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

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("goodbye::upsert_chat");

    if args.is_empty() {
        let chat_data = db::queries::get_chat(&pool, chat_id.0).await.log_err("goodbye::get_chat").flatten();
        let enabled = chat_data.map(|c| c.goodbye_enabled).unwrap_or(true);
        let key = if enabled { "goodbye-status-on" } else { "goodbye-status-off" };
        let mut text = i18n::t(&lang, key, None);
        text.push_str(&format!("\n\n{}", i18n::t(&lang, "goodbye-usage", None)));
        bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    match args[0].to_lowercase().as_str() {
        "on" | "yes" | "true" | "1" => {
            db::queries::toggle_goodbye(&pool, chat_id.0, true).await.log_err("goodbye::toggle");
            bot.send_message(chat_id, i18n::t(&lang, "goodbye-enabled", None))
                .await?;
        }
        "off" | "no" | "false" | "0" => {
            db::queries::toggle_goodbye(&pool, chat_id.0, false).await.log_err("goodbye::toggle");
            bot.send_message(chat_id, i18n::t(&lang, "goodbye-disabled", None))
                .await?;
        }
        _ => {
            bot.send_message(chat_id, i18n::t(&lang, "goodbye-usage", None))
                .await?;
        }
    }
    Ok(())
}

pub async fn set_goodbye(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "goodbye-no-permission", None))
            .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let content = text.split_once(' ').map(|x| x.1);

    let goodbye_text = if let Some(content) = content {
        content.to_string()
    } else if let Some(reply) = msg.reply_to_message() {
        reply.text().unwrap_or("").to_string()
    } else {
        bot.send_message(chat_id, i18n::t(&lang, "goodbye-set-usage", None))
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    };

    if goodbye_text.is_empty() {
        bot.send_message(chat_id, i18n::t(&lang, "goodbye-text-empty", None))
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("goodbye::upsert_chat");
    db::queries::set_goodbye(&pool, chat_id.0, &goodbye_text).await.log_err("goodbye::set_goodbye");

    bot.send_message(chat_id, i18n::t(&lang, "goodbye-updated", None))
        .await?;
    Ok(())
}
