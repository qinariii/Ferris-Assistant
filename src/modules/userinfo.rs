use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::config::AppConfig;
use crate::db;
use crate::utils::{extraction, formatting, formatting::uid_to_i64};

const MAX_INFO_LEN: usize = 1024;


pub async fn setme(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let text = msg.text().unwrap_or("");
    let info = text.split_once(' ').map(|x| x.1).unwrap_or("").trim();

    if info.is_empty() {
        bot.send_message(chat_id, "❌ Usage: /setme <text>\n\nSet info about yourself that others can see.")
            .await?;
        return Ok(());
    }

    if info.len() > MAX_INFO_LEN {
        bot.send_message(
            chat_id,
            format!("❌ Info must be under {} characters! You have {}.", MAX_INFO_LEN, info.len()),
        )
        .await?;
        return Ok(());
    }

    db::queries::set_user_me(&pool, uid_to_i64(from.id), info).await.ok();
    bot.send_message(chat_id, "✅ Your info has been updated!")
        .await?;
    Ok(())
}


pub async fn me(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    let (target_user, _) = extraction::extract_user_and_reason(&bot, &msg, &args, &pool).await;
    let target_id = target_user.unwrap_or_else(|| msg.from.as_ref().map(|u| u.id).unwrap_or(UserId(0)));

    if target_id.0 == 0 {
        return Ok(());
    }

    let info = db::queries::get_user_me(&pool, uid_to_i64(target_id))
        .await
        .unwrap_or_default();

    if info.is_empty() {
        if target_id == msg.from.as_ref().map(|u| u.id).unwrap_or(UserId(0)) {
            bot.send_message(chat_id, "ℹ️ You haven't set any info about yourself yet! Use /setme <text>")
                .await?;
        } else {
            bot.send_message(chat_id, "ℹ️ This user hasn't set any info about themselves yet.")
                .await?;
        }
        return Ok(());
    }

    let member = bot.get_chat_member(chat_id, target_id).await.ok();
    let name = member
        .map(|m| m.user.first_name.clone())
        .unwrap_or_else(|| target_id.to_string());

    bot.send_message(
        chat_id,
        format!(
            "👤 <b>{}:</b>\n{}",
            formatting::html_escape(&name),
            formatting::html_escape(&info),
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}


pub async fn setbio(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let reply = match msg.reply_to_message() {
        Some(r) => r,
        None => {
            bot.send_message(chat_id, "❌ Reply to someone's message to set their bio!")
                .await?;
            return Ok(());
        }
    };

    let target = match reply.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    // Can't set your own bio
    if target.id == from.id {
        bot.send_message(
            chat_id,
            "❌ You can't set your own bio! You're at the mercy of others here...",
        )
        .await?;
        return Ok(());
    }

    // Can't set bot's bio unless sudo
    if target.is_bot && !cfg.is_sudo(uid_to_i64(from.id)) {
        bot.send_message(chat_id, "❌ Only sudo users can set the bot's bio.")
            .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let bio = text.split_once(' ').map(|x| x.1).unwrap_or("").trim();

    if bio.is_empty() {
        bot.send_message(chat_id, "❌ Usage: Reply to a user with /setbio <text>")
            .await?;
        return Ok(());
    }

    if bio.len() > MAX_INFO_LEN {
        bot.send_message(
            chat_id,
            format!("❌ Bio must be under {} characters! You have {}.", MAX_INFO_LEN, bio.len()),
        )
        .await?;
        return Ok(());
    }

    db::queries::set_user_bio(&pool, uid_to_i64(target.id), bio).await.ok();
    bot.send_message(
        chat_id,
        format!("✅ Updated {}'s bio!", formatting::html_escape(&target.first_name)),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}


pub async fn bio(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    let (target_user, _) = extraction::extract_user_and_reason(&bot, &msg, &args, &pool).await;
    let target_id = target_user.unwrap_or_else(|| msg.from.as_ref().map(|u| u.id).unwrap_or(UserId(0)));

    if target_id.0 == 0 {
        return Ok(());
    }

    let bio_text = db::queries::get_user_bio(&pool, uid_to_i64(target_id))
        .await
        .unwrap_or_default();

    if bio_text.is_empty() {
        if target_id == msg.from.as_ref().map(|u| u.id).unwrap_or(UserId(0)) {
            bot.send_message(chat_id, "ℹ️ You don't have a bio set yet! Someone needs to reply to you with /setbio <text>")
                .await?;
        } else {
            bot.send_message(chat_id, "ℹ️ This user doesn't have a bio set yet.")
                .await?;
        }
        return Ok(());
    }

    let member = bot.get_chat_member(chat_id, target_id).await.ok();
    let name = member
        .map(|m| m.user.first_name.clone())
        .unwrap_or_else(|| target_id.to_string());

    bot.send_message(
        chat_id,
        format!(
            "📝 <b>{}:</b>\n{}",
            formatting::html_escape(&name),
            formatting::html_escape(&bio_text),
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}
