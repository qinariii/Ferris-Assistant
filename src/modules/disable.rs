use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::db;
use crate::utils::{cache::TtlCache, permissions};

static DISABLE_CACHE: Lazy<Arc<TtlCache<i64, Vec<String>>>> =
    Lazy::new(|| TtlCache::new(Duration::from_secs(60)));

const DISABLEABLE_COMMANDS: &[&str] = &[
    "adminlist", "ban", "unban", "kick", "mute", "unmute", "tmute", "warn", "warns",
    "resetwarns", "rules", "notes", "filters", "blacklist", "purge", "del", "pin",
    "unpin", "flood", "id", "info",
];

pub async fn disable(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to disable commands.")
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
        bot.send_message(chat_id, "❌ Usage: /disable <command>\n\nDisableable commands: adminlist, ban, unban, kick, mute, unmute, warn, warns, rules, notes, filters, blacklist, purge, del, pin, unpin, flood, id, info")
            .await?;
        return Ok(());
    }

    let cmd = args[0].to_lowercase();
    let cmd = cmd.strip_prefix('/').unwrap_or(&cmd);

    if !DISABLEABLE_COMMANDS.contains(&cmd) {
        bot.send_message(chat_id, format!("❌ <code>{}</code> is not a disableable command.", cmd))
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();
    db::queries::disable_command(&pool, chat_id.0, cmd).await.ok();
    DISABLE_CACHE.invalidate(&chat_id.0);

    bot.send_message(
        chat_id,
        format!("✅ Command <code>/{}</code> has been disabled.", cmd),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn enable(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to enable commands.")
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
        bot.send_message(chat_id, "❌ Usage: /enable <command>")
            .await?;
        return Ok(());
    }

    let cmd = args[0].to_lowercase();
    let cmd = cmd.strip_prefix('/').unwrap_or(&cmd);

    match db::queries::enable_command(&pool, chat_id.0, cmd).await {
        Ok(true) => {
            DISABLE_CACHE.invalidate(&chat_id.0);
            bot.send_message(
                chat_id,
                format!("✅ Command <code>/{}</code> has been enabled.", cmd),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {
            bot.send_message(
                chat_id,
                format!("❌ <code>/{}</code> was not disabled.", cmd),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }
    Ok(())
}

pub async fn disabled_list(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    let disabled = db::queries::get_disabled_commands(&pool, chat_id.0)
        .await
        .unwrap_or_default();

    if disabled.is_empty() {
        bot.send_message(chat_id, "✅ No commands are disabled in this chat.")
            .await?;
        return Ok(());
    }

    let mut text = "<b>🔒 Disabled commands:</b>\n".to_string();
    for cmd in &disabled {
        text.push_str(&format!("\n• <code>/{}</code>", cmd));
    }
    text.push_str("\n\n<i>Use /enable &lt;command&gt; to re-enable.</i>");

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

/// Check if a command is disabled in the current chat.
/// Uses a TTL cache (60s) to avoid a DB hit on every command invocation.
pub async fn is_disabled(pool: &db::Pool, chat_id: i64, command: &str) -> bool {
    let disabled = if let Some(cached) = DISABLE_CACHE.get(&chat_id) {
        cached
    } else {
        let list = db::queries::get_disabled_commands(pool, chat_id)
            .await
            .unwrap_or_default();
        DISABLE_CACHE.set(chat_id, list.clone());
        list
    };
    disabled.iter().any(|c| c == command)
}
