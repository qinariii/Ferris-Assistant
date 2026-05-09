use teloxide::prelude::*;
use teloxide::types::{ChatPermissions, ParseMode};

use crate::db;
use crate::utils::permissions;

/// Map a key name to the corresponding ChatPermissions flag
fn key_to_flag(key: &str) -> Option<ChatPermissions> {
    match key {
        "messages" | "msg" => Some(ChatPermissions::SEND_MESSAGES),
        "photos" => Some(ChatPermissions::SEND_PHOTOS),
        "videos" => Some(ChatPermissions::SEND_VIDEOS),
        "audios" | "audio" => Some(ChatPermissions::SEND_AUDIOS),
        "documents" | "docs" => Some(ChatPermissions::SEND_DOCUMENTS),
        "videonotes" => Some(ChatPermissions::SEND_VIDEO_NOTES),
        "voicenotes" => Some(ChatPermissions::SEND_VOICE_NOTES),
        "stickers" | "othermsg" => Some(ChatPermissions::SEND_OTHER_MESSAGES),
        "polls" => Some(ChatPermissions::SEND_POLLS),
        "preview" | "webpreview" => Some(ChatPermissions::ADD_WEB_PAGE_PREVIEWS),
        "info" | "changeinfo" => Some(ChatPermissions::CHANGE_INFO),
        "invite" => Some(ChatPermissions::INVITE_USERS),
        "pin" => Some(ChatPermissions::PIN_MESSAGES),
        "topics" => Some(ChatPermissions::MANAGE_TOPICS),
        "media" => Some(
            ChatPermissions::SEND_PHOTOS
                .union(ChatPermissions::SEND_VIDEOS)
                .union(ChatPermissions::SEND_AUDIOS)
                .union(ChatPermissions::SEND_DOCUMENTS)
                .union(ChatPermissions::SEND_VIDEO_NOTES)
                .union(ChatPermissions::SEND_VOICE_NOTES),
        ),
        _ => None,
    }
}

/// All permission flags with their display names
const PERM_ENTRIES: &[(&str, ChatPermissions)] = &[
    ("Messages", ChatPermissions::SEND_MESSAGES),
    ("Photos", ChatPermissions::SEND_PHOTOS),
    ("Videos", ChatPermissions::SEND_VIDEOS),
    ("Audios", ChatPermissions::SEND_AUDIOS),
    ("Documents", ChatPermissions::SEND_DOCUMENTS),
    ("Video Notes", ChatPermissions::SEND_VIDEO_NOTES),
    ("Voice Notes", ChatPermissions::SEND_VOICE_NOTES),
    ("Stickers/GIFs", ChatPermissions::SEND_OTHER_MESSAGES),
    ("Polls", ChatPermissions::SEND_POLLS),
    ("Web Preview", ChatPermissions::ADD_WEB_PAGE_PREVIEWS),
    ("Change Info", ChatPermissions::CHANGE_INFO),
    ("Invite Users", ChatPermissions::INVITE_USERS),
    ("Pin Messages", ChatPermissions::PIN_MESSAGES),
    ("Manage Topics", ChatPermissions::MANAGE_TOPICS),
];

pub async fn set_permissions(bot: Bot, msg: Message, _pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to change chat permissions.")
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
        show_permissions(bot, msg).await?;
        return Ok(());
    }

    let mut perms = get_current_permissions(&bot, chat_id).await;

    for arg in &args {
        let parts: Vec<&str> = arg.split('=').collect();
        if parts.len() != 2 {
            continue;
        }
        let key = parts[0].to_lowercase();
        let val = matches!(parts[1].to_lowercase().as_str(), "on" | "yes" | "true" | "1");

        if let Some(flag) = key_to_flag(&key) {
            if val {
                perms = perms.union(flag);
            } else {
                perms = perms.difference(flag);
            }
        }
    }

    match bot.set_chat_permissions(chat_id, perms).await {
        Ok(_) => {
            bot.send_message(chat_id, "✅ Chat permissions updated!")
                .await?;
        }
        Err(e) => {
            bot.send_message(
                chat_id,
                format!("❌ Failed to update permissions: {}", e),
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn view_permissions(bot: Bot, msg: Message) -> ResponseResult<()> {
    show_permissions(bot, msg).await
}

async fn show_permissions(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let chat_info = bot.get_chat(chat_id).await?;

    let text = if let Some(perms) = chat_info.permissions() {
        let mut lines = vec!["<b>📋 Chat Permissions</b>\n".to_string()];
        for (name, flag) in PERM_ENTRIES {
            let emoji = if perms.contains(flag.clone()) { "✅" } else { "❌" };
            lines.push(format!("• {}: {}", name, emoji));
        }
        lines.push(String::new());
        lines.push("<i>Use /setpermissions key=on/off to change.\nKeys: messages, media, photos, videos, audios, documents, stickers, polls, preview, info, invite, pin, topics</i>".to_string());
        lines.join("\n")
    } else {
        "❌ Could not retrieve chat permissions.".to_string()
    };

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

async fn get_current_permissions(bot: &Bot, chat_id: ChatId) -> ChatPermissions {
    if let Ok(chat_info) = bot.get_chat(chat_id).await {
        if let Some(perms) = chat_info.permissions() {
            return perms.clone();
        }
    }
    default_permissions()
}

/// Default safe permissions
pub fn default_permissions() -> ChatPermissions {
    ChatPermissions::SEND_MESSAGES
        .union(ChatPermissions::SEND_PHOTOS)
        .union(ChatPermissions::SEND_VIDEOS)
        .union(ChatPermissions::SEND_AUDIOS)
        .union(ChatPermissions::SEND_DOCUMENTS)
        .union(ChatPermissions::SEND_VIDEO_NOTES)
        .union(ChatPermissions::SEND_VOICE_NOTES)
        .union(ChatPermissions::SEND_OTHER_MESSAGES)
        .union(ChatPermissions::ADD_WEB_PAGE_PREVIEWS)
        .union(ChatPermissions::INVITE_USERS)
        .union(ChatPermissions::SEND_POLLS)
}

/// Muted permissions (all restricted)
#[allow(dead_code)]
pub fn muted_permissions() -> ChatPermissions {
    ChatPermissions::empty()
}

/// Resolve unmute permissions: use chat defaults or fallback
pub async fn resolve_unmute_permissions(bot: &Bot, chat_id: ChatId) -> ChatPermissions {
    get_current_permissions(bot, chat_id).await
}
