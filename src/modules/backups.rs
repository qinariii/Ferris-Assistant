use teloxide::prelude::*;
use teloxide::types::{InputFile, ParseMode};

use crate::db;
use crate::utils::permissions;

pub async fn export(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_owner(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ Only the group owner can export settings.")
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");

    // Gather all data
    let chat_data = db::queries::get_chat(&pool, chat_id.0).await.ok().flatten();
    let notes = db::queries::get_all_notes(&pool, chat_id.0).await.unwrap_or_default();
    let filters = db::queries::get_filters(&pool, chat_id.0).await.unwrap_or_default();
    let blacklist = db::queries::get_blacklist(&pool, chat_id.0).await.unwrap_or_default();
    let disabled = db::queries::get_disabled_commands(&pool, chat_id.0).await.unwrap_or_default();
    let locks = db::queries::get_locks(&pool, chat_id.0).await.unwrap_or_default();
    let warns = db::queries::get_warns_all(&pool, chat_id.0).await.unwrap_or_default();

    let backup = serde_json::json!({
        "bot": "FerrisBot",
        "version": 1,
        "chat_id": chat_id.0,
        "chat_name": chat_name,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "settings": {
            "welcome_enabled": chat_data.as_ref().map(|c| c.welcome_enabled).unwrap_or(true),
            "welcome_text": chat_data.as_ref().map(|c| c.welcome_text.clone()).unwrap_or_default(),
            "goodbye_enabled": chat_data.as_ref().map(|c| c.goodbye_enabled).unwrap_or(true),
            "goodbye_text": chat_data.as_ref().map(|c| c.goodbye_text.clone()).unwrap_or_default(),
            "rules": chat_data.as_ref().map(|c| c.rules.clone()).unwrap_or_default(),
            "warn_limit": chat_data.as_ref().map(|c| c.warn_limit).unwrap_or(3),
            "warn_mode": chat_data.as_ref().map(|c| c.warn_mode.clone()).unwrap_or_else(|| "ban".to_string()),
            "antiflood_count": chat_data.as_ref().map(|c| c.antiflood_count).unwrap_or(0),
            "antiflood_mode": chat_data.as_ref().map(|c| c.antiflood_mode.clone()).unwrap_or_else(|| "mute".to_string()),
            "language": chat_data.as_ref().map(|c| c.language.clone()).unwrap_or_else(|| "en".to_string()),
        },
        "notes": notes.iter().map(|n| serde_json::json!({
            "name": n.name,
            "content": n.content,
        })).collect::<Vec<_>>(),
        "filters": filters.iter().map(|f| serde_json::json!({
            "keyword": f.keyword,
            "reply_text": f.reply_text,
        })).collect::<Vec<_>>(),
        "blacklist": blacklist.iter().map(|b| serde_json::json!({
            "trigger": b.trigger_word,
            "mode": b.mode,
        })).collect::<Vec<_>>(),
        "disabled_commands": disabled,
        "locks": locks,
        "warns": warns.iter().map(|w| serde_json::json!({
            "user_id": w.user_id,
            "reason": w.reason,
        })).collect::<Vec<_>>(),
    });

    let json_str = serde_json::to_string_pretty(&backup).unwrap_or_default();
    let filename = format!("ferris_backup_{}.json", chat_id.0);

    bot.send_document(
        chat_id,
        InputFile::memory(json_str.into_bytes()).file_name(filename),
    )
    .caption("✅ Backup exported successfully!\nUse /import to restore from this file.")
    .await?;

    Ok(())
}

pub async fn import(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_owner(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ Only the group owner can import settings.")
            .await?;
        return Ok(());
    }

    let reply = match msg.reply_to_message() {
        Some(r) => r,
        None => {
            bot.send_message(chat_id, "❌ Reply to a backup file (JSON document) with /import.")
                .await?;
            return Ok(());
        }
    };

    let doc = match reply.document() {
        Some(d) => d,
        None => {
            bot.send_message(chat_id, "❌ The replied message must be a document/file.")
                .await?;
            return Ok(());
        }
    };

    // Check file extension
    let filename = doc.file_name.as_deref().unwrap_or("");
    if !filename.ends_with(".json") && !filename.ends_with(".backup") {
        bot.send_message(chat_id, "❌ Invalid file format. Only .json or .backup files are accepted.")
            .await?;
        return Ok(());
    }

    // Download file
    let file = bot.get_file(doc.file.id.clone()).await?;
    let mut buf = Vec::new();
    teloxide::net::Download::download_file(&bot, &file.path, &mut buf).await?;

    let json_str = match String::from_utf8(buf) {
        Ok(s) => s,
        Err(_) => {
            bot.send_message(chat_id, "❌ Failed to read file content.")
                .await?;
            return Ok(());
        }
    };

    let backup: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => {
            bot.send_message(chat_id, "❌ Invalid JSON format.")
                .await?;
            return Ok(());
        }
    };

    let status_msg = bot
        .send_message(chat_id, "⏳ Importing backup...")
        .await?;

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.ok();

    // Import settings
    if let Some(settings) = backup.get("settings") {
        if let Some(welcome_text) = settings.get("welcome_text").and_then(|v| v.as_str()) {
            db::queries::set_welcome(&pool, chat_id.0, welcome_text).await.ok();
        }
        if let Some(welcome_enabled) = settings.get("welcome_enabled").and_then(|v| v.as_bool()) {
            db::queries::toggle_welcome(&pool, chat_id.0, welcome_enabled).await.ok();
        }
        if let Some(goodbye_text) = settings.get("goodbye_text").and_then(|v| v.as_str()) {
            db::queries::set_goodbye(&pool, chat_id.0, goodbye_text).await.ok();
        }
        if let Some(goodbye_enabled) = settings.get("goodbye_enabled").and_then(|v| v.as_bool()) {
            db::queries::toggle_goodbye(&pool, chat_id.0, goodbye_enabled).await.ok();
        }
        if let Some(rules) = settings.get("rules").and_then(|v| v.as_str()) {
            db::queries::set_rules(&pool, chat_id.0, rules).await.ok();
        }
        if let Some(warn_limit) = settings.get("warn_limit").and_then(|v| v.as_i64()) {
            db::queries::set_warn_limit(&pool, chat_id.0, warn_limit as i32).await.ok();
        }
        if let Some(warn_mode) = settings.get("warn_mode").and_then(|v| v.as_str()) {
            db::queries::set_warn_mode(&pool, chat_id.0, warn_mode).await.ok();
        }
        if let Some(af_count) = settings.get("antiflood_count").and_then(|v| v.as_i64()) {
            db::queries::set_antiflood(&pool, chat_id.0, af_count as i32).await.ok();
        }
        if let Some(af_mode) = settings.get("antiflood_mode").and_then(|v| v.as_str()) {
            db::queries::set_antiflood_mode(&pool, chat_id.0, af_mode).await.ok();
        }
        if let Some(lang) = settings.get("language").and_then(|v| v.as_str()) {
            db::queries::set_language(&pool, chat_id.0, lang).await.ok();
        }
    }

    // Import notes
    if let Some(notes) = backup.get("notes").and_then(|v| v.as_array()) {
        for note in notes {
            let name = note.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let content = note.get("content").and_then(|v| v.as_str()).unwrap_or("");
            if !name.is_empty() && !content.is_empty() {
                db::queries::save_note(&pool, chat_id.0, name, content).await.ok();
            }
        }
    }

    // Import filters
    if let Some(filters) = backup.get("filters").and_then(|v| v.as_array()) {
        for filter in filters {
            let keyword = filter.get("keyword").and_then(|v| v.as_str()).unwrap_or("");
            let reply_text = filter.get("reply_text").and_then(|v| v.as_str()).unwrap_or("");
            if !keyword.is_empty() && !reply_text.is_empty() {
                db::queries::add_filter(&pool, chat_id.0, keyword, reply_text).await.ok();
            }
        }
    }

    // Import blacklist
    if let Some(blacklist) = backup.get("blacklist").and_then(|v| v.as_array()) {
        for item in blacklist {
            let trigger = item.get("trigger").and_then(|v| v.as_str()).unwrap_or("");
            let mode = item.get("mode").and_then(|v| v.as_str()).unwrap_or("delete");
            if !trigger.is_empty() {
                db::queries::add_blacklist(&pool, chat_id.0, trigger, mode).await.ok();
            }
        }
    }

    // Import disabled commands
    if let Some(disabled) = backup.get("disabled_commands").and_then(|v| v.as_array()) {
        for cmd in disabled {
            if let Some(c) = cmd.as_str() {
                db::queries::disable_command(&pool, chat_id.0, c).await.ok();
            }
        }
    }

    // Import locks
    if let Some(locks) = backup.get("locks").and_then(|v| v.as_array()) {
        for lock in locks {
            if let Some(l) = lock.as_str() {
                db::queries::add_lock(&pool, chat_id.0, l).await.ok();
            }
        }
    }

    // Import warns
    if let Some(warns) = backup.get("warns").and_then(|v| v.as_array()) {
        for warn in warns {
            let user_id = warn.get("user_id").and_then(|v| v.as_i64()).unwrap_or(0);
            let reason = warn.get("reason").and_then(|v| v.as_str()).unwrap_or("");
            let warned_by = warn.get("warned_by").and_then(|v| v.as_i64()).unwrap_or(0);
            if user_id != 0 {
                db::queries::add_warn(&pool, chat_id.0, user_id, reason, warned_by).await.ok();
            }
        }
    }

    bot.edit_message_text(chat_id, status_msg.id, "✅ Backup imported successfully!")
        .await?;

    Ok(())
}