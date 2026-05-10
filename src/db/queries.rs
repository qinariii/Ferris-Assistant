use super::models::*;
use super::Pool;
use crate::utils::redis_cache;
pub async fn upsert_chat(pool: &Pool, chat_id: i64, chat_name: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO chats (chat_id, chat_name) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET chat_name = excluded.chat_name",
    )
    .bind(chat_id)
    .bind(chat_name)
    .execute(pool)
    .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}

pub async fn get_chat(pool: &Pool, chat_id: i64) -> Result<Option<Chat>, sqlx::Error> {
    let key = redis_cache::chat_key(chat_id);
    if let Some(cached) = redis_cache::get(&key).await {
        if let Ok(chat) = serde_json::from_str::<Chat>(&cached) {
            return Ok(Some(chat));
        }
    }
    let chat = sqlx::query_as::<_, Chat>("SELECT * FROM chats WHERE chat_id = $1")
        .bind(chat_id)
        .fetch_optional(pool)
        .await?;
    if let Some(ref c) = chat {
        if let Ok(json) = serde_json::to_string(c) {
            redis_cache::set(&key, &json, 300).await;
        }
    }
    Ok(chat)
}

#[allow(dead_code)]
pub async fn get_or_create_chat(pool: &Pool, chat_id: i64, chat_name: &str) -> Result<Chat, sqlx::Error> {
    upsert_chat(pool, chat_id, chat_name).await?;
    get_chat(pool, chat_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}
pub async fn upsert_user(
    pool: &Pool,
    user_id: i64,
    username: &str,
    first_name: &str,
    last_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO users (user_id, username, first_name, last_name) VALUES ($1, $2, $3, $4)
         ON CONFLICT(user_id) DO UPDATE SET username = excluded.username, first_name = excluded.first_name, last_name = excluded.last_name",
    )
    .bind(user_id)
    .bind(username)
    .bind(first_name)
    .bind(last_name)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_user_by_username(pool: &Pool, username: &str) -> Result<Option<super::models::User>, sqlx::Error> {
    sqlx::query_as::<_, super::models::User>(
        "SELECT * FROM users WHERE LOWER(username) = LOWER($1)",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

pub async fn get_user_by_id(pool: &Pool, user_id: i64) -> Result<Option<super::models::User>, sqlx::Error> {
    sqlx::query_as::<_, super::models::User>("SELECT * FROM users WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}
pub async fn set_welcome(pool: &Pool, chat_id: i64, text: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET welcome_text = $1 WHERE chat_id = $2")
        .bind(text)
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}

pub async fn toggle_welcome(pool: &Pool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET welcome_enabled = $1 WHERE chat_id = $2")
        .bind(enabled)
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}

pub async fn set_goodbye(pool: &Pool, chat_id: i64, text: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET goodbye_text = $1 WHERE chat_id = $2")
        .bind(text)
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}

pub async fn toggle_goodbye(pool: &Pool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET goodbye_enabled = $1 WHERE chat_id = $2")
        .bind(enabled)
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}
pub async fn set_rules(pool: &Pool, chat_id: i64, rules: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET rules = $1 WHERE chat_id = $2")
        .bind(rules)
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}

pub async fn get_rules(pool: &Pool, chat_id: i64) -> Result<String, sqlx::Error> {
    let chat = get_chat(pool, chat_id).await?;
    Ok(chat.map(|c| c.rules).unwrap_or_default())
}
pub async fn add_warn(
    pool: &Pool,
    chat_id: i64,
    user_id: i64,
    reason: &str,
    warned_by: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        "INSERT INTO warns (chat_id, user_id, reason, warned_by) VALUES ($1, $2, $3, $4)",
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(reason)
    .bind(warned_by)
    .execute(pool)
    .await?;

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM warns WHERE chat_id = $1 AND user_id = $2")
            .bind(chat_id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(count.0)
}

pub async fn get_warns(pool: &Pool, chat_id: i64, user_id: i64) -> Result<Vec<Warn>, sqlx::Error> {
    sqlx::query_as::<_, Warn>("SELECT * FROM warns WHERE chat_id = $1 AND user_id = $2 ORDER BY created_at DESC")
        .bind(chat_id)
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn get_warns_all(pool: &Pool, chat_id: i64) -> Result<Vec<Warn>, sqlx::Error> {
    sqlx::query_as::<_, Warn>("SELECT * FROM warns WHERE chat_id = $1 ORDER BY created_at DESC")
        .bind(chat_id)
        .fetch_all(pool)
        .await
}

pub async fn reset_warns(pool: &Pool, chat_id: i64, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM warns WHERE chat_id = $1 AND user_id = $2")
        .bind(chat_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_last_warn(pool: &Pool, chat_id: i64, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM warns WHERE id = (SELECT id FROM warns WHERE chat_id = $1 AND user_id = $2 ORDER BY created_at DESC LIMIT 1)",
    )
    .bind(chat_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_warn_limit(pool: &Pool, chat_id: i64, limit: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET warn_limit = $1 WHERE chat_id = $2")
        .bind(limit)
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}

pub async fn set_warn_mode(pool: &Pool, chat_id: i64, mode: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET warn_mode = $1 WHERE chat_id = $2")
        .bind(mode)
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}
pub async fn save_note(
    pool: &Pool,
    chat_id: i64,
    name: &str,
    content: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO notes (chat_id, name, content) VALUES ($1, $2, $3)
         ON CONFLICT(chat_id, name) DO UPDATE SET content = excluded.content",
    )
    .bind(chat_id)
    .bind(name.to_lowercase())
    .bind(content)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_note(pool: &Pool, chat_id: i64, name: &str) -> Result<Option<Note>, sqlx::Error> {
    sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE chat_id = $1 AND name = $2")
        .bind(chat_id)
        .bind(name.to_lowercase())
        .fetch_optional(pool)
        .await
}

pub async fn get_all_notes(pool: &Pool, chat_id: i64) -> Result<Vec<Note>, sqlx::Error> {
    sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE chat_id = $1 ORDER BY name")
        .bind(chat_id)
        .fetch_all(pool)
        .await
}

pub async fn delete_note(pool: &Pool, chat_id: i64, name: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM notes WHERE chat_id = $1 AND name = $2")
        .bind(chat_id)
        .bind(name.to_lowercase())
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
pub async fn add_filter(
    pool: &Pool,
    chat_id: i64,
    keyword: &str,
    reply_text: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO filters (chat_id, keyword, reply_text) VALUES ($1, $2, $3)
         ON CONFLICT(chat_id, keyword) DO UPDATE SET reply_text = excluded.reply_text",
    )
    .bind(chat_id)
    .bind(keyword.to_lowercase())
    .bind(reply_text)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_filters(pool: &Pool, chat_id: i64) -> Result<Vec<Filter>, sqlx::Error> {
    sqlx::query_as::<_, Filter>("SELECT * FROM filters WHERE chat_id = $1 ORDER BY keyword")
        .bind(chat_id)
        .fetch_all(pool)
        .await
}

#[allow(dead_code)]
pub async fn get_filter_by_keyword(
    pool: &Pool,
    chat_id: i64,
    keyword: &str,
) -> Result<Option<Filter>, sqlx::Error> {
    sqlx::query_as::<_, Filter>("SELECT * FROM filters WHERE chat_id = $1 AND keyword = $2")
        .bind(chat_id)
        .bind(keyword.to_lowercase())
        .fetch_optional(pool)
        .await
}

pub async fn delete_filter(pool: &Pool, chat_id: i64, keyword: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM filters WHERE chat_id = $1 AND keyword = $2")
        .bind(chat_id)
        .bind(keyword.to_lowercase())
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_all_filters(pool: &Pool, chat_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM filters WHERE chat_id = $1")
        .bind(chat_id)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn add_blacklist(
    pool: &Pool,
    chat_id: i64,
    trigger: &str,
    mode: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO blacklist (chat_id, trigger_word, mode) VALUES ($1, $2, $3)
         ON CONFLICT(chat_id, trigger_word) DO UPDATE SET mode = excluded.mode",
    )
    .bind(chat_id)
    .bind(trigger.to_lowercase())
    .bind(mode)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_blacklist(pool: &Pool, chat_id: i64) -> Result<Vec<Blacklist>, sqlx::Error> {
    sqlx::query_as::<_, Blacklist>(
        "SELECT * FROM blacklist WHERE chat_id = $1 ORDER BY trigger_word",
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await
}

pub async fn remove_blacklist(
    pool: &Pool,
    chat_id: i64,
    trigger: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM blacklist WHERE chat_id = $1 AND trigger_word = $2")
        .bind(chat_id)
        .bind(trigger.to_lowercase())
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn set_blacklist_mode(
    pool: &Pool,
    chat_id: i64,
    mode: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE blacklist SET mode = $1 WHERE chat_id = $2")
        .bind(mode)
        .bind(chat_id)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn set_antiflood(pool: &Pool, chat_id: i64, count: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET antiflood_count = $1 WHERE chat_id = $2")
        .bind(count)
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}

pub async fn set_antiflood_mode(pool: &Pool, chat_id: i64, mode: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET antiflood_mode = $1 WHERE chat_id = $2")
        .bind(mode)
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}
pub async fn disable_command(pool: &Pool, chat_id: i64, command: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO disabled_commands (chat_id, command) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(chat_id)
    .bind(command.to_lowercase())
    .execute(pool)
    .await?;
    redis_cache::del(&redis_cache::disabled_cmds_key(chat_id)).await;
    Ok(())
}

pub async fn enable_command(pool: &Pool, chat_id: i64, command: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM disabled_commands WHERE chat_id = $1 AND command = $2")
        .bind(chat_id)
        .bind(command.to_lowercase())
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::disabled_cmds_key(chat_id)).await;
    Ok(result.rows_affected() > 0)
}

pub async fn get_disabled_commands(pool: &Pool, chat_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let key = redis_cache::disabled_cmds_key(chat_id);
    if let Some(cached) = redis_cache::get(&key).await {
        if let Ok(cmds) = serde_json::from_str::<Vec<String>>(&cached) {
            return Ok(cmds);
        }
    }
    let rows: Vec<DisabledCommand> = sqlx::query_as(
        "SELECT * FROM disabled_commands WHERE chat_id = $1",
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await?;
    let cmds: Vec<String> = rows.into_iter().map(|r| r.command).collect();
    if let Ok(json) = serde_json::to_string(&cmds) {
        redis_cache::set(&key, &json, 300).await;
    }
    Ok(cmds)
}

#[allow(dead_code)]
pub async fn is_command_disabled(pool: &Pool, chat_id: i64, command: &str) -> Result<bool, sqlx::Error> {
    let result: Option<DisabledCommand> = sqlx::query_as(
        "SELECT * FROM disabled_commands WHERE chat_id = $1 AND command = $2",
    )
    .bind(chat_id)
    .bind(command.to_lowercase())
    .fetch_optional(pool)
    .await?;
    Ok(result.is_some())
}
pub async fn set_language(pool: &Pool, chat_id: i64, lang: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET language = $1 WHERE chat_id = $2")
        .bind(lang)
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}
pub async fn set_log_channel(pool: &Pool, chat_id: i64, channel_id: Option<i64>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET log_channel = $1 WHERE chat_id = $2")
        .bind(channel_id)
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::chat_key(chat_id)).await;
    Ok(())
}

pub async fn get_log_channel(pool: &Pool, chat_id: i64) -> Result<Option<i64>, sqlx::Error> {
    let chat = get_chat(pool, chat_id).await?;
    Ok(chat.and_then(|c| c.log_channel))
}
pub async fn add_lock(pool: &Pool, chat_id: i64, lock_type: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO locks (chat_id, lock_type) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(chat_id)
    .bind(lock_type.to_lowercase())
    .execute(pool)
    .await?;
    redis_cache::del(&redis_cache::locks_key(chat_id)).await;
    Ok(())
}

pub async fn remove_lock(pool: &Pool, chat_id: i64, lock_type: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM locks WHERE chat_id = $1 AND lock_type = $2")
        .bind(chat_id)
        .bind(lock_type.to_lowercase())
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::locks_key(chat_id)).await;
    Ok(result.rows_affected() > 0)
}

pub async fn get_locks(pool: &Pool, chat_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let key = redis_cache::locks_key(chat_id);
    if let Some(cached) = redis_cache::get(&key).await {
        if let Ok(locks) = serde_json::from_str::<Vec<String>>(&cached) {
            return Ok(locks);
        }
    }
    let rows: Vec<Lock> = sqlx::query_as("SELECT * FROM locks WHERE chat_id = $1")
        .bind(chat_id)
        .fetch_all(pool)
        .await?;
    let locks: Vec<String> = rows.into_iter().map(|r| r.lock_type).collect();
    if let Ok(json) = serde_json::to_string(&locks) {
        redis_cache::set(&key, &json, 300).await;
    }
    Ok(locks)
}

#[allow(dead_code)]
pub async fn is_locked(pool: &Pool, chat_id: i64, lock_type: &str) -> Result<bool, sqlx::Error> {
    let result: Option<Lock> = sqlx::query_as(
        "SELECT * FROM locks WHERE chat_id = $1 AND lock_type = $2",
    )
    .bind(chat_id)
    .bind(lock_type.to_lowercase())
    .fetch_optional(pool)
    .await?;
    Ok(result.is_some())
}

pub async fn remove_all_locks(pool: &Pool, chat_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM locks WHERE chat_id = $1")
        .bind(chat_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::locks_key(chat_id)).await;
    Ok(())
}
pub async fn gban_user(pool: &Pool, user_id: i64, reason: &str, banned_by: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO gbans (user_id, reason, banned_by) VALUES ($1, $2, $3)
         ON CONFLICT(user_id) DO UPDATE SET reason = excluded.reason",
    )
    .bind(user_id)
    .bind(reason)
    .bind(banned_by)
    .execute(pool)
    .await?;
    redis_cache::del(&redis_cache::gban_key(user_id)).await;
    Ok(())
}

pub async fn ungban_user(pool: &Pool, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM gbans WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    redis_cache::del(&redis_cache::gban_key(user_id)).await;
    Ok(result.rows_affected() > 0)
}

pub async fn is_gbanned(pool: &Pool, user_id: i64) -> Result<bool, sqlx::Error> {
    let key = redis_cache::gban_key(user_id);
    if let Some(cached) = redis_cache::get(&key).await {
        return Ok(cached == "1");
    }
    let result: Option<Gban> = sqlx::query_as("SELECT * FROM gbans WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    let is_banned = result.is_some();
    redis_cache::set(&key, if is_banned { "1" } else { "0" }, 300).await;
    Ok(is_banned)
}

pub async fn get_gban(pool: &Pool, user_id: i64) -> Result<Option<Gban>, sqlx::Error> {
    sqlx::query_as::<_, Gban>("SELECT * FROM gbans WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_all_gbans(pool: &Pool) -> Result<Vec<Gban>, sqlx::Error> {
    sqlx::query_as::<_, Gban>("SELECT * FROM gbans ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}
pub async fn connect_chat(pool: &Pool, user_id: i64, chat_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO connections (user_id, chat_id) VALUES ($1, $2)
         ON CONFLICT(user_id) DO UPDATE SET chat_id = excluded.chat_id",
    )
    .bind(user_id)
    .bind(chat_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn disconnect_chat(pool: &Pool, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM connections WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_connection(pool: &Pool, user_id: i64) -> Result<Option<i64>, sqlx::Error> {
    let row: Option<Connection> = sqlx::query_as(
        "SELECT * FROM connections WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.chat_id))
}
pub async fn set_report_setting(pool: &Pool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO report_settings (chat_id, enabled) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET enabled = excluded.enabled",
    )
    .bind(chat_id)
    .bind(enabled)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_report_setting(pool: &Pool, chat_id: i64) -> Result<bool, sqlx::Error> {
    let row: Option<ReportSetting> = sqlx::query_as(
        "SELECT * FROM report_settings WHERE chat_id = $1",
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.enabled).unwrap_or(true))
}
pub async fn get_all_chats(pool: &Pool) -> Result<Vec<Chat>, sqlx::Error> {
    sqlx::query_as::<_, Chat>("SELECT * FROM chats")
        .fetch_all(pool)
        .await
}
pub async fn set_afk(pool: &Pool, user_id: i64, reason: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO afk_users (user_id, reason, is_afk) VALUES ($1, $2, TRUE)
         ON CONFLICT(user_id) DO UPDATE SET reason = excluded.reason, is_afk = TRUE",
    )
    .bind(user_id)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_afk(pool: &Pool, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM afk_users WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[allow(dead_code)]
pub async fn is_afk(pool: &Pool, user_id: i64) -> Result<bool, sqlx::Error> {
    let row: Option<AfkUser> = sqlx::query_as("SELECT * FROM afk_users WHERE user_id = $1 AND is_afk = TRUE")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

pub async fn get_afk(pool: &Pool, user_id: i64) -> Result<Option<AfkUser>, sqlx::Error> {
    sqlx::query_as::<_, AfkUser>("SELECT * FROM afk_users WHERE user_id = $1 AND is_afk = TRUE")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}
pub async fn add_blacklist_sticker(pool: &Pool, chat_id: i64, sticker_set: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO blacklist_stickers (chat_id, sticker_set) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(chat_id)
    .bind(sticker_set.to_lowercase())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_blacklist_sticker(pool: &Pool, chat_id: i64, sticker_set: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM blacklist_stickers WHERE chat_id = $1 AND sticker_set = $2")
        .bind(chat_id)
        .bind(sticker_set.to_lowercase())
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_blacklist_stickers(pool: &Pool, chat_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<BlacklistSticker> = sqlx::query_as(
        "SELECT * FROM blacklist_stickers WHERE chat_id = $1",
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.sticker_set).collect())
}

pub async fn set_blsticker_mode(pool: &Pool, chat_id: i64, mode: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO blsticker_settings (chat_id, mode) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET mode = excluded.mode",
    )
    .bind(chat_id)
    .bind(mode)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_blsticker_mode(pool: &Pool, chat_id: i64) -> Result<i32, sqlx::Error> {
    let row: Option<BlstickerSetting> = sqlx::query_as(
        "SELECT * FROM blsticker_settings WHERE chat_id = $1",
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.mode).unwrap_or(1))
}
pub async fn track_user_chat(pool: &Pool, user_id: i64, chat_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_chats (user_id, chat_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(chat_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_user_count(pool: &Pool) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(count.0)
}

pub async fn get_chat_count(pool: &Pool) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chats")
        .fetch_one(pool)
        .await?;
    Ok(count.0)
}

#[allow(dead_code)]
pub async fn get_user_chats_count(pool: &Pool, user_id: i64) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_chats WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(count.0)
}
pub async fn get_captcha_settings(pool: &Pool, chat_id: i64) -> Result<CaptchaSettings, sqlx::Error> {
    let row: Option<CaptchaSettings> = sqlx::query_as(
        "SELECT * FROM captcha_settings WHERE chat_id = $1",
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.unwrap_or(CaptchaSettings {
        chat_id,
        enabled: false,
        captcha_mode: "math".to_string(),
        timeout_min: 5,
        failure_action: "kick".to_string(),
        max_attempts: 3,
    }))
}

pub async fn set_captcha_enabled(pool: &Pool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO captcha_settings (chat_id, enabled) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET enabled = excluded.enabled",
    )
    .bind(chat_id)
    .bind(enabled)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_captcha_mode(pool: &Pool, chat_id: i64, mode: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO captcha_settings (chat_id, captcha_mode) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET captcha_mode = excluded.captcha_mode",
    )
    .bind(chat_id)
    .bind(mode)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_captcha_timeout(pool: &Pool, chat_id: i64, timeout: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO captcha_settings (chat_id, timeout_min) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET timeout_min = excluded.timeout_min",
    )
    .bind(chat_id)
    .bind(timeout)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_captcha_action(pool: &Pool, chat_id: i64, action: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO captcha_settings (chat_id, failure_action) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET failure_action = excluded.failure_action",
    )
    .bind(chat_id)
    .bind(action)
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn set_captcha_max_attempts(pool: &Pool, chat_id: i64, max: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO captcha_settings (chat_id, max_attempts) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET max_attempts = excluded.max_attempts",
    )
    .bind(chat_id)
    .bind(max)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn create_captcha_attempt(
    pool: &Pool,
    user_id: i64,
    chat_id: i64,
    answer: &str,
    message_id: i64,
    expires_at: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO captcha_attempts (user_id, chat_id, answer, attempts, message_id, expires_at) VALUES ($1, $2, $3, 0, $4, $5)
         ON CONFLICT(user_id, chat_id) DO UPDATE SET answer = excluded.answer, attempts = 0, message_id = excluded.message_id, expires_at = excluded.expires_at",
    )
    .bind(user_id)
    .bind(chat_id)
    .bind(answer)
    .bind(message_id)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_captcha_attempt(pool: &Pool, user_id: i64, chat_id: i64) -> Result<Option<CaptchaAttempt>, sqlx::Error> {
    sqlx::query_as::<_, CaptchaAttempt>(
        "SELECT * FROM captcha_attempts WHERE user_id = $1 AND chat_id = $2",
    )
    .bind(user_id)
    .bind(chat_id)
    .fetch_optional(pool)
    .await
}

pub async fn increment_captcha_attempts(pool: &Pool, user_id: i64, chat_id: i64) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        "UPDATE captcha_attempts SET attempts = attempts + 1 WHERE user_id = $1 AND chat_id = $2 RETURNING attempts",
    )
    .bind(user_id)
    .bind(chat_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.unwrap_or(0))
}

pub async fn delete_captcha_attempt(pool: &Pool, user_id: i64, chat_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM captcha_attempts WHERE user_id = $1 AND chat_id = $2")
        .bind(user_id)
        .bind(chat_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_all_captcha_attempts(pool: &Pool) -> Result<Vec<CaptchaAttempt>, sqlx::Error> {
    sqlx::query_as::<_, CaptchaAttempt>("SELECT * FROM captcha_attempts")
        .fetch_all(pool)
        .await
}

pub async fn delete_all_captcha_attempts(pool: &Pool, chat_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM captcha_attempts WHERE chat_id = $1")
        .bind(chat_id)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn add_dev_team(pool: &Pool, user_id: i64, role: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO dev_team (user_id, role) VALUES ($1, $2)
         ON CONFLICT(user_id) DO UPDATE SET role = excluded.role",
    )
    .bind(user_id)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_dev_team(pool: &Pool, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM dev_team WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_dev_team(pool: &Pool) -> Result<Vec<DevTeamMember>, sqlx::Error> {
    sqlx::query_as::<_, DevTeamMember>("SELECT * FROM dev_team ORDER BY role, user_id")
        .fetch_all(pool)
        .await
}

pub async fn get_dev_team_member(pool: &Pool, user_id: i64) -> Result<Option<DevTeamMember>, sqlx::Error> {
    sqlx::query_as::<_, DevTeamMember>("SELECT * FROM dev_team WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

#[allow(dead_code)]
pub async fn is_dev_team(pool: &Pool, user_id: i64) -> Result<bool, sqlx::Error> {
    let row: Option<DevTeamMember> = sqlx::query_as(
        "SELECT * FROM dev_team WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}
pub async fn create_federation(pool: &Pool, fed_id: &str, fed_name: &str, owner_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO federations (fed_id, fed_name, owner_id) VALUES ($1, $2, $3)",
    )
    .bind(fed_id)
    .bind(fed_name)
    .bind(owner_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_federation(pool: &Pool, fed_id: &str) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM fed_bans WHERE fed_id = $1").bind(fed_id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM fed_admins WHERE fed_id = $1").bind(fed_id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM fed_chats WHERE fed_id = $1").bind(fed_id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM federations WHERE fed_id = $1").bind(fed_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_federation(pool: &Pool, fed_id: &str) -> Result<Option<Federation>, sqlx::Error> {
    sqlx::query_as::<_, Federation>("SELECT * FROM federations WHERE fed_id = $1")
        .bind(fed_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_fed_by_owner(pool: &Pool, owner_id: i64) -> Result<Option<Federation>, sqlx::Error> {
    sqlx::query_as::<_, Federation>("SELECT * FROM federations WHERE owner_id = $1")
        .bind(owner_id)
        .fetch_optional(pool)
        .await
}

pub async fn join_federation(pool: &Pool, chat_id: i64, fed_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO fed_chats (chat_id, fed_id) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET fed_id = excluded.fed_id",
    )
    .bind(chat_id)
    .bind(fed_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn leave_federation(pool: &Pool, chat_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM fed_chats WHERE chat_id = $1")
        .bind(chat_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_fed_chat(pool: &Pool, chat_id: i64) -> Result<Option<FedChat>, sqlx::Error> {
    sqlx::query_as::<_, FedChat>("SELECT * FROM fed_chats WHERE chat_id = $1")
        .bind(chat_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_fed_chats(pool: &Pool, fed_id: &str) -> Result<Vec<FedChat>, sqlx::Error> {
    sqlx::query_as::<_, FedChat>("SELECT * FROM fed_chats WHERE fed_id = $1")
        .bind(fed_id)
        .fetch_all(pool)
        .await
}

pub async fn add_fed_admin(pool: &Pool, fed_id: &str, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO fed_admins (fed_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(fed_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_fed_admin(pool: &Pool, fed_id: &str, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM fed_admins WHERE fed_id = $1 AND user_id = $2")
        .bind(fed_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_fed_admins(pool: &Pool, fed_id: &str) -> Result<Vec<FedAdmin>, sqlx::Error> {
    sqlx::query_as::<_, FedAdmin>("SELECT * FROM fed_admins WHERE fed_id = $1")
        .bind(fed_id)
        .fetch_all(pool)
        .await
}

pub async fn is_fed_admin(pool: &Pool, fed_id: &str, user_id: i64) -> Result<bool, sqlx::Error> {
    let row: Option<FedAdmin> = sqlx::query_as(
        "SELECT * FROM fed_admins WHERE fed_id = $1 AND user_id = $2",
    )
    .bind(fed_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

pub async fn fban_user(pool: &Pool, fed_id: &str, user_id: i64, user_name: &str, reason: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO fed_bans (fed_id, user_id, user_name, reason, banned_at) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT(fed_id, user_id) DO UPDATE SET reason = excluded.reason, user_name = excluded.user_name, banned_at = excluded.banned_at",
    )
    .bind(fed_id)
    .bind(user_id)
    .bind(user_name)
    .bind(reason)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unfban_user(pool: &Pool, fed_id: &str, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM fed_bans WHERE fed_id = $1 AND user_id = $2")
        .bind(fed_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_fban(pool: &Pool, fed_id: &str, user_id: i64) -> Result<Option<FedBan>, sqlx::Error> {
    sqlx::query_as::<_, FedBan>(
        "SELECT * FROM fed_bans WHERE fed_id = $1 AND user_id = $2",
    )
    .bind(fed_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

#[allow(dead_code)]
pub async fn is_fbanned(pool: &Pool, fed_id: &str, user_id: i64) -> Result<bool, sqlx::Error> {
    Ok(get_fban(pool, fed_id, user_id).await?.is_some())
}

pub async fn get_fed_bans(pool: &Pool, fed_id: &str) -> Result<Vec<FedBan>, sqlx::Error> {
    sqlx::query_as::<_, FedBan>("SELECT * FROM fed_bans WHERE fed_id = $1 ORDER BY banned_at DESC")
        .bind(fed_id)
        .fetch_all(pool)
        .await
}

#[allow(dead_code)]
pub async fn get_user_fban_all(pool: &Pool, user_id: i64) -> Result<Vec<FedBan>, sqlx::Error> {
    sqlx::query_as::<_, FedBan>("SELECT * FROM fed_bans WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn set_fed_rules(pool: &Pool, fed_id: &str, rules: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE federations SET fed_rules = $1 WHERE fed_id = $2")
        .bind(rules)
        .bind(fed_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn set_fed_log(pool: &Pool, fed_id: &str, log_channel: Option<i64>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE federations SET fed_log = $1 WHERE fed_id = $2")
        .bind(log_channel)
        .bind(fed_id)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn get_user_info(pool: &Pool, user_id: i64) -> Result<Option<UserInfoRow>, sqlx::Error> {
    sqlx::query_as::<_, UserInfoRow>("SELECT * FROM user_info WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_user_bio(pool: &Pool, user_id: i64) -> Result<String, sqlx::Error> {
    let row = get_user_info(pool, user_id).await?;
    Ok(row.map(|r| r.bio).unwrap_or_default())
}

pub async fn get_user_me(pool: &Pool, user_id: i64) -> Result<String, sqlx::Error> {
    let row = get_user_info(pool, user_id).await?;
    Ok(row.map(|r| r.me_info).unwrap_or_default())
}

pub async fn set_user_bio(pool: &Pool, user_id: i64, bio: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_info (user_id, bio) VALUES ($1, $2)
         ON CONFLICT(user_id) DO UPDATE SET bio = excluded.bio",
    )
    .bind(user_id)
    .bind(bio)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_user_me(pool: &Pool, user_id: i64, me_info: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_info (user_id, me_info) VALUES ($1, $2)
         ON CONFLICT(user_id) DO UPDATE SET me_info = excluded.me_info",
    )
    .bind(user_id)
    .bind(me_info)
    .execute(pool)
    .await?;
    Ok(())
}
pub async fn get_cleaner_settings(pool: &Pool, chat_id: i64) -> Result<CleanerSettings, sqlx::Error> {
    let row: Option<CleanerSettings> = sqlx::query_as(
        "SELECT * FROM cleaner_settings WHERE chat_id = $1",
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.unwrap_or(CleanerSettings {
        chat_id,
        clean_service: false,
        clean_bluetext: false,
    }))
}

pub async fn set_clean_service(pool: &Pool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO cleaner_settings (chat_id, clean_service) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET clean_service = excluded.clean_service",
    )
    .bind(chat_id)
    .bind(enabled)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_clean_bluetext(pool: &Pool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO cleaner_settings (chat_id, clean_bluetext) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET clean_bluetext = excluded.clean_bluetext",
    )
    .bind(chat_id)
    .bind(enabled)
    .execute(pool)
    .await?;
    Ok(())
}
pub async fn add_reaction(pool: &Pool, chat_id: i64, keyword: &str, emoji: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO reactions (chat_id, keyword, emoji) VALUES ($1, $2, $3)
         ON CONFLICT(chat_id, keyword) DO UPDATE SET emoji = excluded.emoji",
    )
    .bind(chat_id)
    .bind(keyword)
    .bind(emoji)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_reaction(pool: &Pool, chat_id: i64, keyword: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM reactions WHERE chat_id = $1 AND keyword = $2")
        .bind(chat_id)
        .bind(keyword)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_reactions(pool: &Pool, chat_id: i64) -> Result<Vec<Reaction>, sqlx::Error> {
    sqlx::query_as::<_, Reaction>("SELECT * FROM reactions WHERE chat_id = $1")
        .bind(chat_id)
        .fetch_all(pool)
        .await
}

pub async fn reset_reactions(pool: &Pool, chat_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM reactions WHERE chat_id = $1")
        .bind(chat_id)
        .execute(pool)
        .await?;
    Ok(())
}
