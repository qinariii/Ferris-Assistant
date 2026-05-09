use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::db;
use crate::utils::{formatting, formatting::uid_to_i64, permissions};

pub async fn report(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    // Don't let admins report
    if permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ Admins don't need to report. Handle it yourself!")
            .await?;
        return Ok(());
    }

    // Check if reporting is enabled
    let enabled = db::queries::get_report_setting(&pool, chat_id.0)
        .await
        .unwrap_or(true);

    if !enabled {
        return Ok(());
    }

    let reply = match msg.reply_to_message() {
        Some(r) => r,
        None => {
            bot.send_message(chat_id, "❌ Reply to a message to report it.")
                .await?;
            return Ok(());
        }
    };

    let reported_user = reply.from.as_ref();
    let reported_name = reported_user
        .map(|u| formatting::user_display_name(u))
        .unwrap_or_else(|| "Unknown".to_string());

    let reporter_name = formatting::user_display_name(from);
    let chat_name = msg.chat.title().unwrap_or("this chat");

    // Notify in chat
    let text = format!(
        "⚠️ <b>Report</b>\n\n\
        <b>Reported by:</b> {}\n\
        <b>Reported user:</b> {}\n\n\
        Admins have been notified!",
        formatting::mention_html(from),
        formatting::html_escape(&reported_name),
    );

    bot.send_message(chat_id, &text)
        .parse_mode(ParseMode::Html)
        .await?;

    // Try to notify admins via PM
    let admins = bot.get_chat_administrators(chat_id).await.unwrap_or_default();
    let admin_text = format!(
        "⚠️ <b>Report in {}</b>\n\n\
        <b>Reporter:</b> {} [<code>{}</code>]\n\
        <b>Reported:</b> {} [<code>{}</code>]\n\n\
        <a href=\"https://t.me/c/{}/{}\">Jump to message</a>",
        formatting::html_escape(chat_name),
        formatting::html_escape(&reporter_name),
        from.id.0,
        formatting::html_escape(&reported_name),
        reported_user.map(|u| u.id.0).unwrap_or(0),
        // For supergroups, remove -100 prefix
        chat_id.0.to_string().trim_start_matches("-100"),
        reply.id.0,
    );

    for admin in &admins {
        if admin.user.is_bot {
            continue;
        }
        bot.send_message(ChatId(uid_to_i64(admin.user.id)), &admin_text)
            .parse_mode(ParseMode::Html)
            .await
            .ok();
    }

    // Log the action
    crate::modules::log_channel::send_log(
        &bot,
        &pool,
        chat_id.0,
        &format!(
            "📋 <b>#REPORT</b>\n\n\
            <b>Chat:</b> {}\n\
            <b>Reporter:</b> {}\n\
            <b>Reported:</b> {}",
            formatting::html_escape(chat_name),
            formatting::html_escape(&reporter_name),
            formatting::html_escape(&reported_name),
        ),
    )
    .await;

    Ok(())
}

pub async fn report_setting(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to change report settings.")
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
        let enabled = db::queries::get_report_setting(&pool, chat_id.0)
            .await
            .unwrap_or(true);
        let status = if enabled { "enabled ✅" } else { "disabled ❌" };
        bot.send_message(
            chat_id,
            format!(
                "📢 Reporting is currently <b>{}</b>.\n\nUsage: /reports &lt;on/off&gt;",
                status
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    match args[0].to_lowercase().as_str() {
        "on" | "yes" | "true" | "1" => {
            db::queries::set_report_setting(&pool, chat_id.0, true).await.ok();
            bot.send_message(chat_id, "✅ Reporting enabled! Users can now use /report.")
                .await?;
        }
        "off" | "no" | "false" | "0" => {
            db::queries::set_report_setting(&pool, chat_id.0, false).await.ok();
            bot.send_message(chat_id, "❌ Reporting disabled.")
                .await?;
        }
        _ => {
            bot.send_message(chat_id, "❌ Usage: /reports <on/off>")
                .await?;
        }
    }
    Ok(())
}

/// Handle @admin mentions as reports
pub async fn check_admin_tag(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let text = msg.text().unwrap_or("");
    if !text.contains("@admin") {
        return Ok(());
    }

    // Treat @admin the same as /report
    report(bot, msg, pool).await
}
