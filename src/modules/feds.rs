use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db;
use crate::utils::{cache::TtlCache, extraction, formatting, formatting::uid_to_i64, permissions};

/// Cache: chat_id → Option<fed_id> (None = not in any fed)
static FED_CHAT_CACHE: Lazy<Arc<TtlCache<i64, Option<String>>>> =
    Lazy::new(|| TtlCache::new(Duration::from_secs(60)));


async fn is_fed_owner_or_admin(pool: &db::Pool, fed_id: &str, user_id: i64) -> bool {
    if let Ok(Some(fed)) = db::queries::get_federation(pool, fed_id).await {
        if fed.owner_id == user_id {
            return true;
        }
    }
    db::queries::is_fed_admin(pool, fed_id, user_id)
        .await
        .unwrap_or(false)
}

async fn is_fed_owner(pool: &db::Pool, fed_id: &str, user_id: i64) -> bool {
    if let Ok(Some(fed)) = db::queries::get_federation(pool, fed_id).await {
        return fed.owner_id == user_id;
    }
    false
}


pub async fn newfed(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let text = msg.text().unwrap_or("");
    let fed_name = text.splitn(2, ' ').nth(1).unwrap_or("").trim();

    if fed_name.is_empty() {
        bot.send_message(chat_id, "❌ Usage: /newfed <federation name>")
            .await?;
        return Ok(());
    }

    // Check if user already owns a federation
    if let Ok(Some(_)) = db::queries::get_fed_by_owner(&pool, uid_to_i64(from.id)).await {
        bot.send_message(chat_id, "❌ You already own a federation. Delete it first with /delfed.")
            .await?;
        return Ok(());
    }

    let fed_id = Uuid::new_v4().to_string();
    match db::queries::create_federation(&pool, &fed_id, fed_name, uid_to_i64(from.id)).await {
        Ok(_) => {
            bot.send_message(
                chat_id,
                format!(
                    "✅ <b>Federation created!</b>\n\n\
                    <b>Name:</b> {}\n\
                    <b>ID:</b> <code>{}</code>\n\n\
                    Use <code>/joinfed {}</code> in a group to join this federation.",
                    formatting::html_escape(fed_name),
                    fed_id,
                    fed_id,
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Failed to create federation: {}", e))
                .await?;
        }
    }
    Ok(())
}


pub async fn delfed(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    if args.is_empty() {
        bot.send_message(chat_id, "❌ Usage: /delfed <fed_id>")
            .await?;
        return Ok(());
    }

    let fed_id = &args[0];
    let fed = match db::queries::get_federation(&pool, fed_id).await {
        Ok(Some(f)) => f,
        _ => {
            bot.send_message(chat_id, "❌ Federation not found.")
                .await?;
            return Ok(());
        }
    };

    if fed.owner_id != uid_to_i64(from.id) {
        bot.send_message(chat_id, "❌ Only the federation owner can delete it.")
            .await?;
        return Ok(());
    }

    let fed_name = fed.fed_name.clone();
    db::queries::delete_federation(&pool, fed_id).await.ok();

    bot.send_message(
        chat_id,
        format!(
            "✅ Federation <b>{}</b> has been deleted.",
            formatting::html_escape(&fed_name)
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}


pub async fn joinfed(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if msg.chat.is_private() {
        bot.send_message(chat_id, "❌ This command is for groups only.")
            .await?;
        return Ok(());
    }

    // Only group owner or sudo can join fed
    if !permissions::is_user_owner(&bot, chat_id, from.id).await && !cfg.is_sudo(uid_to_i64(from.id)) {
        bot.send_message(chat_id, "❌ Only the group creator or sudo users can join a federation.")
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
        bot.send_message(chat_id, "❌ Usage: /joinfed <fed_id>")
            .await?;
        return Ok(());
    }

    let fed_id = &args[0];

    // Check if already in a fed
    if let Ok(Some(existing)) = db::queries::get_fed_chat(&pool, chat_id.0).await {
        bot.send_message(
            chat_id,
            format!(
                "❌ This chat is already in a federation (ID: <code>{}</code>). Leave it first with /leavefed.",
                existing.fed_id
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let fed = match db::queries::get_federation(&pool, fed_id).await {
        Ok(Some(f)) => f,
        _ => {
            bot.send_message(chat_id, "❌ Federation not found.")
                .await?;
            return Ok(());
        }
    };

    db::queries::join_federation(&pool, chat_id.0, fed_id).await.ok();
    FED_CHAT_CACHE.invalidate(&chat_id.0);

    bot.send_message(
        chat_id,
        format!(
            "✅ This chat has joined federation <b>{}</b>!",
            formatting::html_escape(&fed.fed_name)
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}


pub async fn leavefed(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if msg.chat.is_private() {
        bot.send_message(chat_id, "❌ This command is for groups only.")
            .await?;
        return Ok(());
    }

    if !permissions::is_user_owner(&bot, chat_id, from.id).await && !cfg.is_sudo(uid_to_i64(from.id)) {
        bot.send_message(chat_id, "❌ Only the group creator or sudo users can leave a federation.")
            .await?;
        return Ok(());
    }

    match db::queries::leave_federation(&pool, chat_id.0).await {
        Ok(true) => {
            FED_CHAT_CACHE.invalidate(&chat_id.0);
            bot.send_message(chat_id, "✅ This chat has left its federation.")
                .await?;
        }
        _ => {
            bot.send_message(chat_id, "❌ This chat is not in any federation.")
                .await?;
        }
    }
    Ok(())
}


pub async fn fedinfo(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    // If no args, check if this chat is in a fed
    let fed_id = if args.is_empty() {
        match db::queries::get_fed_chat(&pool, chat_id.0).await {
            Ok(Some(fc)) => fc.fed_id,
            _ => {
                // Check if user owns a fed
                match db::queries::get_fed_by_owner(&pool, uid_to_i64(from.id)).await {
                    Ok(Some(f)) => f.fed_id,
                    _ => {
                        bot.send_message(chat_id, "❌ Usage: /fedinfo <fed_id> or use in a federated chat.")
                            .await?;
                        return Ok(());
                    }
                }
            }
        }
    } else {
        args[0].clone()
    };

    let fed = match db::queries::get_federation(&pool, &fed_id).await {
        Ok(Some(f)) => f,
        _ => {
            bot.send_message(chat_id, "❌ Federation not found.")
                .await?;
            return Ok(());
        }
    };

    let admins = db::queries::get_fed_admins(&pool, &fed_id).await.unwrap_or_default();
    let chats = db::queries::get_fed_chats(&pool, &fed_id).await.unwrap_or_default();
    let bans = db::queries::get_fed_bans(&pool, &fed_id).await.unwrap_or_default();

    let owner_name = match bot.get_chat(ChatId(fed.owner_id)).await {
        Ok(c) => c.first_name().unwrap_or("Unknown").to_string(),
        Err(_) => fed.owner_id.to_string(),
    };

    let text = format!(
        "🏛 <b>Federation Info</b>\n\n\
        <b>Name:</b> {}\n\
        <b>ID:</b> <code>{}</code>\n\
        <b>Owner:</b> <a href=\"tg://user?id={}\">{}</a>\n\
        <b>Admins:</b> {}\n\
        <b>Chats:</b> {}\n\
        <b>Bans:</b> {}",
        formatting::html_escape(&fed.fed_name),
        fed.fed_id,
        fed.owner_id,
        formatting::html_escape(&owner_name),
        admins.len(),
        chats.len(),
        bans.len(),
    );

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}


pub async fn fedpromote(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    // Get fed_id from chat
    let fed_id = match db::queries::get_fed_chat(&pool, chat_id.0).await {
        Ok(Some(fc)) => fc.fed_id,
        _ => {
            bot.send_message(chat_id, "❌ This chat is not in a federation.")
                .await?;
            return Ok(());
        }
    };

    if !is_fed_owner(&pool, &fed_id, uid_to_i64(from.id)).await {
        bot.send_message(chat_id, "❌ Only the federation owner can promote admins.")
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
            bot.send_message(chat_id, "❌ Please specify a user to promote.")
                .await?;
            return Ok(());
        }
    };

    db::queries::add_fed_admin(&pool, &fed_id, uid_to_i64(target_id)).await.ok();

    let member = bot.get_chat_member(chat_id, target_id).await.ok();
    let name = member
        .map(|m| m.user.first_name.clone())
        .unwrap_or_else(|| target_id.to_string());

    bot.send_message(
        chat_id,
        format!(
            "✅ {} is now a federation admin.",
            formatting::html_escape(&name)
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}


pub async fn feddemote(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let fed_id = match db::queries::get_fed_chat(&pool, chat_id.0).await {
        Ok(Some(fc)) => fc.fed_id,
        _ => {
            bot.send_message(chat_id, "❌ This chat is not in a federation.")
                .await?;
            return Ok(());
        }
    };

    if !is_fed_owner(&pool, &fed_id, uid_to_i64(from.id)).await {
        bot.send_message(chat_id, "❌ Only the federation owner can demote admins.")
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
            bot.send_message(chat_id, "❌ Please specify a user to demote.")
                .await?;
            return Ok(());
        }
    };

    match db::queries::remove_fed_admin(&pool, &fed_id, uid_to_i64(target_id)).await {
        Ok(true) => {
            bot.send_message(chat_id, "✅ User has been demoted from federation admin.")
                .await?;
        }
        _ => {
            bot.send_message(chat_id, "❌ This user is not a federation admin.")
                .await?;
        }
    }
    Ok(())
}


pub async fn fban(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let fed_id = match db::queries::get_fed_chat(&pool, chat_id.0).await {
        Ok(Some(fc)) => fc.fed_id,
        _ => {
            bot.send_message(chat_id, "❌ This chat is not in a federation.")
                .await?;
            return Ok(());
        }
    };

    if !is_fed_owner_or_admin(&pool, &fed_id, uid_to_i64(from.id)).await && !cfg.is_sudo(uid_to_i64(from.id)) {
        bot.send_message(chat_id, "❌ Only federation owners/admins can fban users.")
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
            bot.send_message(chat_id, "❌ Please specify a user to fban.")
                .await?;
            return Ok(());
        }
    };

    // Can't fban sudo
    if cfg.is_sudo(uid_to_i64(target_id)) {
        bot.send_message(chat_id, "❌ I won't fban a sudo user!")
            .await?;
        return Ok(());
    }

    let reason_str = reason.as_deref().unwrap_or("No reason given");

    let member = bot.get_chat_member(chat_id, target_id).await.ok();
    let name = member
        .as_ref()
        .map(|m| m.user.first_name.clone())
        .unwrap_or_else(|| target_id.to_string());

    let status_msg = bot
        .send_message(chat_id, "🏛 Starting federation ban...")
        .await?;

    // Add to fban list
    db::queries::fban_user(&pool, &fed_id, uid_to_i64(target_id), &name, reason_str)
        .await
        .ok();

    // Ban in all fed chats
    let fed_chats = db::queries::get_fed_chats(&pool, &fed_id).await.unwrap_or_default();
    let mut success = 0;
    let mut fail = 0;

    for fc in &fed_chats {
        match bot.ban_chat_member(ChatId(fc.chat_id), target_id).await {
            Ok(_) => success += 1,
            Err(_) => fail += 1,
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    let fed = db::queries::get_federation(&pool, &fed_id).await.ok().flatten();
    let fed_name = fed.map(|f| f.fed_name).unwrap_or_else(|| "Unknown".to_string());

    let text = format!(
        "🏛 <b>New Federation Ban</b>\n\n\
        <b>Federation:</b> {}\n\
        <b>User:</b> {} [<code>{}</code>]\n\
        <b>Reason:</b> {}\n\
        <b>By:</b> {}\n\n\
        ✅ Banned in {} chats\n❌ Failed in {} chats",
        formatting::html_escape(&fed_name),
        formatting::html_escape(&name),
        target_id.0,
        formatting::html_escape(reason_str),
        formatting::mention_html(from),
        success,
        fail,
    );

    bot.edit_message_text(chat_id, status_msg.id, &text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}


pub async fn unfban(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let fed_id = match db::queries::get_fed_chat(&pool, chat_id.0).await {
        Ok(Some(fc)) => fc.fed_id,
        _ => {
            bot.send_message(chat_id, "❌ This chat is not in a federation.")
                .await?;
            return Ok(());
        }
    };

    if !is_fed_owner_or_admin(&pool, &fed_id, uid_to_i64(from.id)).await && !cfg.is_sudo(uid_to_i64(from.id)) {
        bot.send_message(chat_id, "❌ Only federation owners/admins can unfban users.")
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
            bot.send_message(chat_id, "❌ Please specify a user to unfban.")
                .await?;
            return Ok(());
        }
    };

    match db::queries::unfban_user(&pool, &fed_id, uid_to_i64(target_id)).await {
        Ok(true) => {
            let status_msg = bot
                .send_message(chat_id, "🏛 Removing federation ban...")
                .await?;

            // Unban in all fed chats
            let fed_chats = db::queries::get_fed_chats(&pool, &fed_id).await.unwrap_or_default();
            let mut success = 0;
            for fc in &fed_chats {
                if bot.unban_chat_member(ChatId(fc.chat_id), target_id).await.is_ok() {
                    success += 1;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }

            bot.edit_message_text(
                chat_id,
                status_msg.id,
                format!(
                    "✅ User <code>{}</code> has been un-fbanned.\nUnbanned in {} chats.",
                    target_id.0, success
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {
            bot.send_message(chat_id, "❌ This user is not fbanned in this federation.")
                .await?;
        }
    }
    Ok(())
}


pub async fn fbanlist(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    let fed_id = if args.is_empty() {
        match db::queries::get_fed_chat(&pool, chat_id.0).await {
            Ok(Some(fc)) => fc.fed_id,
            _ => {
                match db::queries::get_fed_by_owner(&pool, uid_to_i64(from.id)).await {
                    Ok(Some(f)) => f.fed_id,
                    _ => {
                        bot.send_message(chat_id, "❌ Usage: /fbanlist <fed_id>")
                            .await?;
                        return Ok(());
                    }
                }
            }
        }
    } else {
        args[0].clone()
    };

    if !is_fed_owner_or_admin(&pool, &fed_id, uid_to_i64(from.id)).await {
        bot.send_message(chat_id, "❌ Only federation owners/admins can view the ban list.")
            .await?;
        return Ok(());
    }

    let bans = db::queries::get_fed_bans(&pool, &fed_id).await.unwrap_or_default();

    if bans.is_empty() {
        bot.send_message(chat_id, "✅ No users are banned in this federation.")
            .await?;
        return Ok(());
    }

    let mut text = format!("🏛 <b>Federation Ban List ({}):</b>\n", bans.len());
    for (i, ban) in bans.iter().enumerate().take(50) {
        text.push_str(&format!(
            "\n{}. {} [<code>{}</code>] — {}",
            i + 1,
            formatting::html_escape(&ban.user_name),
            ban.user_id,
            formatting::html_escape(&ban.reason),
        ));
    }

    if bans.len() > 50 {
        text.push_str(&format!("\n\n<i>...and {} more</i>", bans.len() - 50));
    }

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}


pub async fn fedchat(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match db::queries::get_fed_chat(&pool, chat_id.0).await {
        Ok(Some(fc)) => {
            let fed = db::queries::get_federation(&pool, &fc.fed_id).await.ok().flatten();
            let fed_name = fed
                .map(|f| f.fed_name)
                .unwrap_or_else(|| "Unknown".to_string());

            bot.send_message(
                chat_id,
                format!(
                    "🏛 This chat is part of:\n<b>{}</b>\n(ID: <code>{}</code>)",
                    formatting::html_escape(&fed_name),
                    fc.fed_id
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {
            bot.send_message(chat_id, "❌ This chat is not in any federation.")
                .await?;
        }
    }
    Ok(())
}


pub async fn fedrules(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    let fed_id = match db::queries::get_fed_chat(&pool, chat_id.0).await {
        Ok(Some(fc)) => fc.fed_id,
        _ => {
            match db::queries::get_fed_by_owner(&pool, uid_to_i64(from.id)).await {
                Ok(Some(f)) => f.fed_id,
                _ => {
                    bot.send_message(chat_id, "❌ This chat is not in a federation.")
                        .await?;
                    return Ok(());
                }
            }
        }
    };

    let fed = match db::queries::get_federation(&pool, &fed_id).await {
        Ok(Some(f)) => f,
        _ => {
            bot.send_message(chat_id, "❌ Federation not found.")
                .await?;
            return Ok(());
        }
    };

    if args.is_empty() {
        // View rules
        if fed.fed_rules.is_empty() {
            bot.send_message(chat_id, "📏 No federation rules have been set.")
                .await?;
        } else {
            bot.send_message(
                chat_id,
                format!(
                    "📏 <b>Rules for {}</b>\n\n{}",
                    formatting::html_escape(&fed.fed_name),
                    formatting::html_escape(&fed.fed_rules)
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    } else {
        // Set rules (owner only)
        if !is_fed_owner(&pool, &fed_id, uid_to_i64(from.id)).await {
            bot.send_message(chat_id, "❌ Only the federation owner can set rules.")
                .await?;
            return Ok(());
        }

        let rules_text = args.join(" ");
        db::queries::set_fed_rules(&pool, &fed_id, &rules_text).await.ok();
        bot.send_message(chat_id, "✅ Federation rules updated!")
            .await?;
    }
    Ok(())
}


pub async fn check_fban(
    bot: Bot,
    msg: Message,
    pool: db::Pool,
) -> ResponseResult<()> {
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let chat_id = msg.chat.id;

    // Check if this chat is in a federation (cached)
    let fed_id = if let Some(cached) = FED_CHAT_CACHE.get(&chat_id.0) {
        match cached {
            Some(fid) => fid,
            None => return Ok(()),
        }
    } else {
        match db::queries::get_fed_chat(&pool, chat_id.0).await {
            Ok(Some(fc)) => {
                FED_CHAT_CACHE.set(chat_id.0, Some(fc.fed_id.clone()));
                fc.fed_id
            }
            _ => {
                FED_CHAT_CACHE.set(chat_id.0, None);
                return Ok(());
            }
        }
    };

    // Check if user is fbanned
    if let Ok(Some(ban)) = db::queries::get_fban(&pool, &fed_id, uid_to_i64(from.id)).await {
        // Don't fban admins
        if !permissions::is_user_admin(&bot, chat_id, from.id).await {
            bot.ban_chat_member(chat_id, from.id).await.ok();

            let fed = db::queries::get_federation(&pool, &fed_id)
                .await
                .ok()
                .flatten();
            let fed_name = fed
                .map(|f| f.fed_name)
                .unwrap_or_else(|| "Unknown".to_string());

            bot.send_message(
                chat_id,
                format!(
                    "🏛 <b>Federation banned user detected!</b>\n\n\
                    <b>User:</b> {} [<code>{}</code>]\n\
                    <b>Federation:</b> {}\n\
                    <b>Reason:</b> {}\n\n\
                    <i>Banned automatically.</i>",
                    formatting::mention_html(from),
                    from.id.0,
                    formatting::html_escape(&fed_name),
                    formatting::html_escape(&ban.reason),
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }

    Ok(())
}
