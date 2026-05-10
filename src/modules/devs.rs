use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::config::AppConfig;
use crate::db;
use crate::utils::{extraction, formatting, formatting::uid_to_i64, LogErrExt};


fn is_owner_or_dev(cfg: &AppConfig, user_id: i64, dev_member: &Option<db::models::DevTeamMember>) -> bool {
    cfg.is_owner(user_id) || dev_member.as_ref().map(|m| m.role == "dev").unwrap_or(false)
}

fn is_team_member(cfg: &AppConfig, user_id: i64, dev_member: &Option<db::models::DevTeamMember>) -> bool {
    cfg.is_sudo(user_id) || dev_member.is_some()
}


pub async fn addsudo(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !cfg.is_owner(uid_to_i64(from.id)) {
        bot.send_message(chat_id, "❌ Only the bot owner can use this command.")
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
            bot.send_message(chat_id, "❌ Please specify a user to add as sudo.")
                .await?;
            return Ok(());
        }
    };

    // Check if already exists
    if let Ok(Some(existing)) = db::queries::get_dev_team_member(&pool, uid_to_i64(target_id)).await {
        if existing.role == "sudo" {
            bot.send_message(chat_id, "⚠️ This user is already a sudo user.")
                .await?;
            return Ok(());
        }
    }

    db::queries::add_dev_team(&pool, uid_to_i64(target_id), "sudo").await.log_err("devs::add_sudo");

    let member = bot.get_chat_member(chat_id, target_id).await.ok();
    let name = member
        .map(|m| m.user.first_name.clone())
        .unwrap_or_else(|| target_id.to_string());

    bot.send_message(
        chat_id,
        format!("✅ {} has been added as a <b>sudo</b> user.", formatting::html_escape(&name)),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}


pub async fn remsudo(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !cfg.is_owner(uid_to_i64(from.id)) {
        bot.send_message(chat_id, "❌ Only the bot owner can use this command.")
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
            bot.send_message(chat_id, "❌ Please specify a user to remove from sudo.")
                .await?;
            return Ok(());
        }
    };

    match db::queries::remove_dev_team(&pool, uid_to_i64(target_id)).await {
        Ok(true) => {
            bot.send_message(chat_id, format!("✅ User <code>{}</code> removed from team.", target_id.0))
                .parse_mode(ParseMode::Html)
                .await?;
        }
        _ => {
            bot.send_message(chat_id, "❌ This user is not in the team.")
                .await?;
        }
    }
    Ok(())
}


pub async fn adddev(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !cfg.is_owner(uid_to_i64(from.id)) {
        bot.send_message(chat_id, "❌ Only the bot owner can use this command.")
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
            bot.send_message(chat_id, "❌ Please specify a user to add as dev.")
                .await?;
            return Ok(());
        }
    };

    if let Ok(Some(existing)) = db::queries::get_dev_team_member(&pool, uid_to_i64(target_id)).await {
        if existing.role == "dev" {
            bot.send_message(chat_id, "⚠️ This user is already a dev.")
                .await?;
            return Ok(());
        }
    }

    db::queries::add_dev_team(&pool, uid_to_i64(target_id), "dev").await.log_err("devs::add_dev");

    let member = bot.get_chat_member(chat_id, target_id).await.ok();
    let name = member
        .map(|m| m.user.first_name.clone())
        .unwrap_or_else(|| target_id.to_string());

    bot.send_message(
        chat_id,
        format!("✅ {} has been added as a <b>dev</b>.", formatting::html_escape(&name)),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}


pub async fn remdev(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    // Alias for remsudo — both remove from team
    remsudo(bot, msg, cfg, pool).await
}


pub async fn teamusers(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let dev_member = db::queries::get_dev_team_member(&pool, uid_to_i64(from.id)).await.ok().flatten();
    if !is_team_member(&cfg, uid_to_i64(from.id), &dev_member) {
        bot.send_message(chat_id, "❌ Only team members can view this.")
            .await?;
        return Ok(());
    }

    let team = db::queries::get_dev_team(&pool).await.unwrap_or_default();

    let mut devs = Vec::new();
    let mut sudos = Vec::new();

    for member in &team {
        let name = db::queries::get_user_by_id(&pool, member.user_id)
            .await
            .ok()
            .flatten()
            .map(|u| {
                if u.first_name.is_empty() {
                    u.user_id.to_string()
                } else {
                    u.first_name
                }
            })
            .unwrap_or_else(|| member.user_id.to_string());
        let entry = format!(
            "• <a href=\"tg://user?id={}\">{}</a>",
            member.user_id,
            formatting::html_escape(&name)
        );
        match member.role.as_str() {
            "dev" => devs.push(entry),
            "sudo" => sudos.push(entry),
            _ => sudos.push(entry),
        }
    }

    let owner_name = db::queries::get_user_by_id(&pool, cfg.owner_id)
        .await
        .ok()
        .flatten()
        .map(|u| if u.first_name.is_empty() { "Owner".to_string() } else { u.first_name })
        .unwrap_or_else(|| cfg.owner_id.to_string());

    let mut text = format!(
        "👥 <b>Bot Team</b>\n\n\
        👑 <b>Owner:</b>\n• <a href=\"tg://user?id={}\">{}</a>\n\n",
        cfg.owner_id,
        formatting::html_escape(&owner_name),
    );

    text.push_str("<b>🔧 Developers:</b>\n");
    if devs.is_empty() {
        text.push_str("  <i>None</i>\n");
    } else {
        for d in &devs {
            text.push_str(&format!("{}\n", d));
        }
    }

    text.push_str("\n<b>⚡ Sudo Users:</b>\n");
    if sudos.is_empty() {
        text.push_str("  <i>None</i>\n");
    } else {
        for s in &sudos {
            text.push_str(&format!("{}\n", s));
        }
    }

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}


pub async fn chatinfo(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let dev_member = db::queries::get_dev_team_member(&pool, uid_to_i64(from.id)).await.ok().flatten();
    if !is_owner_or_dev(&cfg, uid_to_i64(from.id), &dev_member) {
        return Ok(());
    }

    let args: Vec<String> = msg
        .text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect();

    let target_chat_id: i64 = if args.is_empty() {
        chat_id.0
    } else {
        match args[0].parse() {
            Ok(id) => id,
            Err(_) => {
                bot.send_message(chat_id, "❌ Please provide a valid chat ID.")
                    .await?;
                return Ok(());
            }
        }
    };

    match bot.get_chat(ChatId(target_chat_id)).await {
        Ok(chat) => {
            let member_count = bot
                .get_chat_member_count(ChatId(target_chat_id))
                .await
                .unwrap_or(0);

            let text = format!(
                "ℹ️ <b>Chat Info</b>\n\n\
                • <b>Name:</b> {}\n\
                • <b>ID:</b> <code>{}</code>\n\
                • <b>Members:</b> {}\n\
                • <b>Type:</b> {:?}",
                formatting::html_escape(chat.title().unwrap_or("N/A")),
                target_chat_id,
                member_count,
                chat.kind,
            );
            bot.send_message(chat_id, text)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Failed to get chat info: {}", e))
                .await?;
        }
    }
    Ok(())
}


pub async fn leavechat(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let dev_member = db::queries::get_dev_team_member(&pool, uid_to_i64(from.id)).await.ok().flatten();
    if !is_owner_or_dev(&cfg, uid_to_i64(from.id), &dev_member) {
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
        bot.send_message(chat_id, "❌ Usage: /leavechat <chat_id>")
            .await?;
        return Ok(());
    }

    let target_chat_id: i64 = match args[0].parse() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(chat_id, "❌ Please provide a valid chat ID.")
                .await?;
            return Ok(());
        }
    };

    match bot.leave_chat(ChatId(target_chat_id)).await {
        Ok(_) => {
            bot.send_message(chat_id, format!("✅ Left chat <code>{}</code>.", target_chat_id))
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Failed to leave chat: {}", e))
                .await?;
        }
    }
    Ok(())
}


pub async fn botstats(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let dev_member = db::queries::get_dev_team_member(&pool, uid_to_i64(from.id)).await.ok().flatten();
    if !is_owner_or_dev(&cfg, uid_to_i64(from.id), &dev_member) {
        return Ok(());
    }

    let status_msg = bot.send_message(chat_id, "📊 Fetching stats...").await?;

    let user_count = db::queries::get_user_count(&pool).await.unwrap_or(0);
    let chat_count = db::queries::get_chat_count(&pool).await.unwrap_or(0);
    let gban_count = db::queries::get_all_gbans(&pool).await.map(|g| g.len()).unwrap_or(0);
    let team_count = db::queries::get_dev_team(&pool).await.map(|t| t.len()).unwrap_or(0);

    let text = format!(
        "📊 <b>Bot Statistics</b>\n\n\
        • <b>Users:</b> {}\n\
        • <b>Chats:</b> {}\n\
        • <b>Global bans:</b> {}\n\
        • <b>Team members:</b> {}",
        user_count, chat_count, gban_count, team_count
    );

    bot.edit_message_text(chat_id, status_msg.id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}


pub async fn broadcast(bot: Bot, msg: Message, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !cfg.is_owner(uid_to_i64(from.id)) {
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let content = text.split_once(' ').map(|x| x.1);
    let broadcast_text = if let Some(content) = content {
        content.to_string()
    } else if let Some(reply) = msg.reply_to_message() {
        reply.text().unwrap_or("").to_string()
    } else {
        bot.send_message(chat_id, "❌ Usage: /broadcast <text> or reply to a message.")
            .await?;
        return Ok(());
    };

    if broadcast_text.is_empty() {
        bot.send_message(chat_id, "❌ Broadcast text cannot be empty.")
            .await?;
        return Ok(());
    }

    let status_msg = bot
        .send_message(chat_id, "📢 Broadcasting...")
        .await?;

    let all_chats = db::queries::get_all_chats(&pool).await.unwrap_or_default();
    let mut success = 0;
    let mut fail = 0;

    for chat in &all_chats {
        match bot
            .send_message(ChatId(chat.chat_id), &broadcast_text)
            .parse_mode(ParseMode::Html)
            .await
        {
            Ok(_) => success += 1,
            Err(_) => fail += 1,
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    bot.edit_message_text(
        chat_id,
        status_msg.id,
        format!(
            "📢 <b>Broadcast Complete</b>\n\n✅ Success: {}\n❌ Failed: {}",
            success, fail
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}
