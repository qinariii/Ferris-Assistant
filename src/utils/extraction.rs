use teloxide::prelude::*;
use teloxide::types::Message;

use crate::db;

/// Extract command arguments (everything after the command itself).
#[allow(dead_code)]
pub fn cmd_args(msg: &Message) -> Vec<String> {
    msg.text()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect()
}

/// Extract user ID from message - either from reply or from arguments.
/// Resolves @username via local database lookup.
pub async fn extract_user_and_reason(
    _bot: &Bot,
    msg: &Message,
    args: &[String],
    pool: &db::Pool,
) -> (Option<UserId>, Option<String>) {
    // Try to get user from reply
    if let Some(reply) = msg.reply_to_message() {
        if let Some(user) = reply.from.as_ref() {
            let reason = if args.is_empty() {
                None
            } else {
                Some(args.join(" "))
            };
            return (Some(user.id), reason);
        }
    }

    // Try to get user from arguments
    if args.is_empty() {
        return (None, None);
    }

    let first_arg = &args[0];

    // Check if it's a user ID
    if let Ok(id) = first_arg.parse::<u64>() {
        let reason = if args.len() > 1 {
            Some(args[1..].join(" "))
        } else {
            None
        };
        return (Some(UserId(id)), reason);
    }

    // Check if it's a username (starts with @) — resolve via DB
    if first_arg.starts_with('@') {
        let username = first_arg.trim_start_matches('@');
        let reason = if args.len() > 1 {
            Some(args[1..].join(" "))
        } else {
            None
        };
        if let Ok(Some(user)) = db::queries::get_user_by_username(pool, username).await {
            return (Some(UserId(user.user_id as u64)), reason);
        }
        return (None, reason);
    }

    (None, None)
}

/// Extract user from reply message only
pub fn extract_user_from_reply(msg: &Message) -> Option<UserId> {
    msg.reply_to_message()
        .and_then(|reply| reply.from.as_ref())
        .map(|user| user.id)
}

/// Extract text after command from message
#[allow(dead_code)]
pub fn extract_text_after_command(text: &str) -> Option<String> {
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    if parts.len() > 1 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Split arguments: first is target, rest is reason
#[allow(dead_code)]
pub fn split_user_and_reason(args: &[String]) -> (Option<String>, Option<String>) {
    if args.is_empty() {
        return (None, None);
    }
    let target = Some(args[0].clone());
    let reason = if args.len() > 1 {
        Some(args[1..].join(" "))
    } else {
        None
    };
    (target, reason)
}
