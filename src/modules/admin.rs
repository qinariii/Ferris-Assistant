use teloxide::prelude::*;
use teloxide::types::{ChatMemberKind, ParseMode};

use crate::config::AppConfig;
use crate::db;
use crate::utils::{extraction, formatting, i18n, permissions};

pub async fn promote(bot: Bot, msg: Message, _cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "admin-need-admin", None))
            .await?;
        return Ok(());
    }

    if !permissions::can_user_promote(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "admin-no-promote-perm", None))
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
            bot.send_message(chat_id, i18n::t(&lang, "admin-promote-usage", None))
                .await?;
            return Ok(());
        }
    };

    // Ensure chat exists in DB
    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();

    match bot
        .promote_chat_member(chat_id, target_id)
        .can_change_info(true)
        .can_delete_messages(true)
        .can_restrict_members(true)
        .can_pin_messages(true)
        .can_invite_users(true)
        .can_manage_video_chats(true)
        .await
    {
        Ok(_) => {
            permissions::invalidate_user_perms(chat_id, target_id);
            let member = bot.get_chat_member(chat_id, target_id).await?;
            let name = formatting::html_escape(&member.user.first_name);
            bot.send_message(
                chat_id,
                i18n::t(&lang, "admin-promote-success", Some(&[("name", &name)])),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "admin-promote-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}

pub async fn demote(bot: Bot, msg: Message, _cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "admin-need-admin", None))
            .await?;
        return Ok(());
    }

    if !permissions::can_user_promote(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "admin-no-promote-perm", None))
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
            bot.send_message(chat_id, i18n::t(&lang, "admin-demote-usage", None))
                .await?;
            return Ok(());
        }
    };

    match bot
        .promote_chat_member(chat_id, target_id)
        .can_change_info(false)
        .can_delete_messages(false)
        .can_restrict_members(false)
        .can_pin_messages(false)
        .can_invite_users(false)
        .can_manage_video_chats(false)
        .can_promote_members(false)
        .await
    {
        Ok(_) => {
            permissions::invalidate_user_perms(chat_id, target_id);
            let member = bot.get_chat_member(chat_id, target_id).await?;
            let name = formatting::html_escape(&member.user.first_name);
            bot.send_message(
                chat_id,
                i18n::t(&lang, "admin-demote-success", Some(&[("name", &name)])),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "admin-demote-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}

pub async fn adminlist(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let chat_title = msg.chat.title().unwrap_or("this chat");

    let admins = match bot.get_chat_administrators(chat_id).await {
        Ok(a) => a,
        Err(_) => {
            bot.send_message(chat_id, i18n::t(&lang, "admin-adminlist-failed", None))
                .await?;
            return Ok(());
        }
    };

    let escaped_title = formatting::html_escape(chat_title);
    let mut text = i18n::t(&lang, "admin-adminlist-header", Some(&[("chat", &escaped_title)]));
    text.push('\n');
    let mut owner_text = String::new();
    let mut admin_list = Vec::new();

    for admin in &admins {
        if admin.user.is_bot {
            continue;
        }
        let name = if let Some(ref username) = admin.user.username {
            format!("@{}", username)
        } else {
            formatting::mention_html(&admin.user)
        };

        match admin.kind {
            ChatMemberKind::Owner(_) => {
                owner_text = format!("\n{}", i18n::t(&lang, "admin-adminlist-owner", Some(&[("name", &name)])));
            }
            _ => {
                admin_list.push(format!("  • {}", name));
            }
        }
    }

    text.push_str(&owner_text);
    if !admin_list.is_empty() {
        text.push_str(&format!("\n\n{}\n", i18n::t(&lang, "admin-adminlist-admins", None)));
        text.push_str(&admin_list.join("\n"));
    }

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn set_title(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "admin-need-admin", None))
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

    let title = args.join(" ");
    if title.is_empty() {
        bot.send_message(chat_id, i18n::t(&lang, "admin-title-usage", None))
            .await?;
        return Ok(());
    }

    let target_id = match extraction::extract_user_from_reply(&msg) {
        Some(id) => id,
        None => {
            bot.send_message(chat_id, i18n::t(&lang, "admin-title-reply", None))
                .await?;
            return Ok(());
        }
    };

    if title.len() > 16 {
        bot.send_message(chat_id, i18n::t(&lang, "admin-title-too-long", None))
            .await?;
        return Ok(());
    }

    match bot.set_chat_administrator_custom_title(chat_id, target_id, &title).await {
        Ok(_) => {
            let escaped = formatting::html_escape(&title);
            bot.send_message(chat_id, i18n::t(&lang, "admin-title-set", Some(&[("title", &escaped)])))
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            let err = e.to_string();
            bot.send_message(chat_id, i18n::t(&lang, "admin-title-failed", Some(&[("error", &err)])))
                .await?;
        }
    }
    Ok(())
}
