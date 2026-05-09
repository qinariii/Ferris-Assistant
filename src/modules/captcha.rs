use rand::Rng;
use teloxide::prelude::*;
use teloxide::types::{ChatMemberUpdated, ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

use crate::config::AppConfig;
use crate::db;
use crate::utils::{formatting, formatting::uid_to_i64, i18n, permissions};

// ---------------------------------------------------------------------------
// Helper: Safe kick with retry, without arbitrary sleep
// ---------------------------------------------------------------------------

/// Kick user: ban and then unban using exponential backoff.
/// Replaces the `ban -> sleep(1s) -> unban` pattern, which is prone to race conditions.
async fn safe_kick(bot: &Bot, chat_id: ChatId, user_id: UserId) {
    if bot.ban_chat_member(chat_id, user_id).await.is_err() {
        return;
    }
    for attempt in 0u32..3 {
        match bot.unban_chat_member(chat_id, user_id).await {
            Ok(_) => return,
            Err(_) => {
                let delay = tokio::time::Duration::from_millis(200 * (1u64 << attempt));
                tokio::time::sleep(delay).await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Captcha generators
// ---------------------------------------------------------------------------

fn generate_math_captcha() -> (String, String, Vec<String>) {
    let mut rng = rand::thread_rng();
    let ops = ['+', '-', '*'];
    let op = ops[rng.gen_range(0..ops.len())];

    let (a, b, answer) = match op {
        '+' => {
            let a = rng.gen_range(1..=50);
            let b = rng.gen_range(1..=50);
            (a, b, a + b)
        }
        '-' => {
            let a = rng.gen_range(20..=70);
            let b = rng.gen_range(1..=a);
            (a, b, a - b)
        }
        '*' => {
            let a = rng.gen_range(1..=12);
            let b = rng.gen_range(1..=12);
            (a, b, a * b)
        }
        _ => unreachable!(),
    };

    let op_display = if op == '*' { 'x' } else { op };
    let question = format!("{} {} {}", a, op_display, b);

    let mut options: Vec<String> = vec![answer.to_string()];
    while options.len() < 4 {
        let wrong = answer + rng.gen_range(-10..=10);
        if wrong != answer && wrong > 0 {
            let wrong_str = wrong.to_string();
            if !options.contains(&wrong_str) {
                options.push(wrong_str);
            }
        }
    }

    for i in (1..options.len()).rev() {
        let j = rng.gen_range(0..=i);
        options.swap(i, j);
    }

    (question, answer.to_string(), options)
}

fn generate_text_captcha() -> (String, Vec<String>) {
    let mut rng = rand::thread_rng();
    let chars = b"234567890abcdefghjkmnpqrstuvwxyz";
    let len = 4;

    let answer: String = (0..len)
        .map(|_| chars[rng.gen_range(0..chars.len())] as char)
        .collect();

    let mut options = vec![answer.clone()];
    while options.len() < 4 {
        let decoy: String = (0..len)
            .map(|_| chars[rng.gen_range(0..chars.len())] as char)
            .collect();
        if !options.contains(&decoy) {
            options.push(decoy);
        }
    }

    for i in (1..options.len()).rev() {
        let j = rng.gen_range(0..=i);
        options.swap(i, j);
    }

    (answer, options)
}

fn build_captcha_keyboard(options: &[String], user_id: i64) -> InlineKeyboardMarkup {
    let buttons: Vec<Vec<InlineKeyboardButton>> = options
        .iter()
        .map(|opt| {
            vec![InlineKeyboardButton::callback(
                opt.clone(),
                format!("captcha_{}_{}", user_id, opt),
            )]
        })
        .collect();
    InlineKeyboardMarkup::new(buttons)
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

pub async fn captcha_cmd(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "captcha-need-admin", None))
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

    let settings = db::queries::get_captcha_settings(&pool, chat_id.0).await.unwrap_or_else(|_| {
        db::models::CaptchaSettings {
            chat_id: chat_id.0,
            enabled: false,
            captcha_mode: "math".to_string(),
            timeout_min: 5,
            failure_action: "kick".to_string(),
            max_attempts: 3,
        }
    });

    if args.is_empty() {
        let status = if settings.enabled { "enabled ✅" } else { "disabled ❌" };
        let text = format!(
            "{}\n\n\
            • <b>Status:</b> {}\n\
            • <b>Mode:</b> {}\n\
            • <b>Timeout:</b> {} min\n\
            • <b>Failure action:</b> {}\n\
            • <b>Max attempts:</b> {}\n\n\
            Usage: /captcha &lt;on/off&gt;",
            i18n::t(&lang, "captcha-settings", None),
            status, settings.captcha_mode, settings.timeout_min,
            settings.failure_action, settings.max_attempts
        );
        bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    match args[0].to_lowercase().as_str() {
        "on" | "enable" | "yes" => {
            db::queries::set_captcha_enabled(&pool, chat_id.0, true).await.ok();
            bot.send_message(chat_id, i18n::t(&lang, "captcha-enabled", None))
                .await?;
        }
        "off" | "disable" | "no" => {
            db::queries::set_captcha_enabled(&pool, chat_id.0, false).await.ok();
            db::queries::delete_all_captcha_attempts(&pool, chat_id.0).await.ok();
            bot.send_message(chat_id, i18n::t(&lang, "captcha-disabled", None))
                .await?;
        }
        _ => {
            bot.send_message(chat_id, i18n::t(&lang, "captcha-usage", None))
                .await?;
        }
    }
    Ok(())
}

pub async fn captchamode_cmd(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "captcha-need-admin", None))
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
        bot.send_message(chat_id, i18n::t(&lang, "captcha-mode-usage", None))
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let mode = args[0].to_lowercase();
    if mode != "math" && mode != "text" {
        bot.send_message(chat_id, i18n::t(&lang, "captcha-mode-invalid", None))
            .await?;
        return Ok(());
    }

    db::queries::set_captcha_mode(&pool, chat_id.0, &mode).await.ok();
    bot.send_message(chat_id, i18n::t(&lang, "captcha-mode-set", Some(&[("mode", &mode)])))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn captchatime_cmd(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "captcha-need-admin", None))
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
        bot.send_message(chat_id, i18n::t(&lang, "captcha-timeout-usage", None))
            .await?;
        return Ok(());
    }

    let timeout: i64 = match args[0].parse() {
        Ok(v) if (1..=10).contains(&v) => v,
        _ => {
            bot.send_message(chat_id, i18n::t(&lang, "captcha-timeout-invalid", None))
                .await?;
            return Ok(());
        }
    };

    let timeout_str = timeout.to_string();
    db::queries::set_captcha_timeout(&pool, chat_id.0, timeout).await.ok();
    bot.send_message(chat_id, i18n::t(&lang, "captcha-timeout-set", Some(&[("timeout", &timeout_str)])))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn captchaaction_cmd(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, i18n::t(&lang, "captcha-need-admin", None))
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
        bot.send_message(chat_id, i18n::t(&lang, "captcha-action-usage", None))
            .await?;
        return Ok(());
    }

    let action = args[0].to_lowercase();
    if !["kick", "ban", "mute"].contains(&action.as_str()) {
        bot.send_message(chat_id, i18n::t(&lang, "captcha-action-invalid", None))
            .await?;
        return Ok(());
    }

    db::queries::set_captcha_action(&pool, chat_id.0, &action).await.ok();
    bot.send_message(chat_id, i18n::t(&lang, "captcha-action-set", Some(&[("action", &action)])))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Join handler — Send a CAPTCHA to new members
// ---------------------------------------------------------------------------

/// Send captcha to a new member. Called from the chat_member handler.
pub async fn send_captcha_on_join(
    bot: Bot,
    update: &ChatMemberUpdated,
    pool: db::Pool,
) -> ResponseResult<()> {
    let chat_id = update.chat.id;
    let user = &update.new_chat_member.user;

    if user.is_bot {
        return Ok(());
    }

    let settings = db::queries::get_captcha_settings(&pool, chat_id.0)
        .await
        .unwrap_or_else(|_| db::models::CaptchaSettings {
            chat_id: chat_id.0,
            enabled: false,
            captcha_mode: "math".to_string(),
            timeout_min: 5,
            failure_action: "kick".to_string(),
            max_attempts: 3,
        });

    if !settings.enabled {
        return Ok(());
    }

    // Mute the user until the CAPTCHA is completed
    let perms = ChatPermissions::empty();
    bot.restrict_chat_member(chat_id, user.id, perms).await.ok();

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    let (question_text, answer, options) = if settings.captcha_mode == "text" {
        let (ans, opts) = generate_text_captcha();
        let spaced: String = ans.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
        (
            i18n::t(&lang, "captcha-solve-text", Some(&[("code", &spaced)])),
            ans.clone(),
            opts,
        )
    } else {
        let (q, ans, opts) = generate_math_captcha();
        (
            i18n::t(&lang, "captcha-solve-math", Some(&[("question", &q)])),
            ans,
            opts,
        )
    };

    let user_mention = formatting::mention_html(user);
    let timeout_str = settings.timeout_min.to_string();
    let attempts_str = settings.max_attempts.to_string();
    let time_info = i18n::t(&lang, "captcha-time-info", Some(&[("timeout", &timeout_str), ("attempts", &attempts_str)]));
    let text = format!(
        "{}\n\nWelcome {}!\n\n{}\n\n{}",
        i18n::t(&lang, "captcha-welcome", None),
        user_mention,
        question_text,
        time_info,
    );

    let keyboard = build_captcha_keyboard(&options, uid_to_i64(user.id));

    let sent = bot
        .send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    // Save the attempt to the database
    let expires = chrono::Utc::now()
        + chrono::Duration::minutes(settings.timeout_min);
    db::queries::create_captcha_attempt(
        &pool,
        uid_to_i64(user.id),
        chat_id.0,
        &answer,
        sent.id.0 as i64,
        &expires.to_rfc3339(),
    )
    .await
    .ok();

    // FIX: Use a one-shot channel so that the task timeout can be canceled
    // when the user successfully solves the CAPTCHA before the timeout.
    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();

    // Store `cancel_tx` in the database so it can be triggered from `captcha_callback`.
    // Since `oneshot::Sender` cannot be stored in SQLite, we use
    // an in-memory cache (DashMap/Mutex) stored as a static global.
    CAPTCHA_CANCEL_MAP.insert((uid_to_i64(user.id), chat_id.0), cancel_tx);

    let bot_clone = bot.clone();
    let pool_clone = pool.clone();
    let user_id = user.id;
    let failure_action = settings.failure_action.clone();
    let timeout_min = settings.timeout_min;
    let msg_id = sent.id;

    tokio::spawn(async move {
        let sleep = tokio::time::sleep(tokio::time::Duration::from_secs(timeout_min as u64 * 60));
        tokio::pin!(sleep);

        tokio::select! {
            // FIX: The task is canceled if the user answers correctly
            _ = cancel_rx => {
                log::debug!("captcha timeout task cancelled for user {}", user_id.0);
                return;
            }
            _ = &mut sleep => {
                // Check if the attempt is still active (the user hasn't responded yet)
                if let Ok(Some(_attempt)) =
                    db::queries::get_captcha_attempt(&pool_clone, uid_to_i64(user_id), chat_id.0).await
                {
                    // FIX: Use `safe_kick` instead of `ban` -> `sleep(1)` -> `unban`
                    match failure_action.as_str() {
                        "ban" => {
                            bot_clone.ban_chat_member(chat_id, user_id).await.ok();
                        }
                        "mute" => {
                            // The user has been muted from the start; no further action is needed
                        }
                        _ => {
                            safe_kick(&bot_clone, chat_id, user_id).await;
                        }
                    }

                    bot_clone
                        .send_message(
                            chat_id,
                            format!(
                                "⏱ User {} failed to complete captcha in time. Action: <b>{}</b>",
                                user_id.0, &failure_action
                            ),
                        )
                        .parse_mode(ParseMode::Html)
                        .await
                        .ok();

                    // Cleanup
                    db::queries::delete_captcha_attempt(&pool_clone, uid_to_i64(user_id), chat_id.0)
                        .await
                        .ok();
                    bot_clone.delete_message(chat_id, msg_id).await.ok();
                }

                // Delete the entry from the cancellation list
                CAPTCHA_CANCEL_MAP.remove(&(uid_to_i64(user_id), chat_id.0));
            }
        }
    });

    Ok(())
}

// ---------------------------------------------------------------------------
// In-memory map to store cancel senders by (user_id, chat_id)
// ---------------------------------------------------------------------------

use once_cell::sync::Lazy;
use std::collections::HashMap;
use parking_lot::Mutex;

type CancelKey = (i64, i64);
type CancelSender = tokio::sync::oneshot::Sender<()>;

static CAPTCHA_CANCEL_MAP: Lazy<CaptchaCancelMap> = Lazy::new(CaptchaCancelMap::new);

struct CaptchaCancelMap {
    inner: Mutex<HashMap<CancelKey, CancelSender>>,
}

impl CaptchaCancelMap {
    fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    fn insert(&self, key: CancelKey, tx: CancelSender) {
        self.inner.lock().insert(key, tx);
    }

    /// Retrieve and remove the sender — calling `send()` will cancel the task timeout.
    fn take_and_cancel(&self, key: &CancelKey) {
        if let Some(tx) = self.inner.lock().remove(key) {
            let _ = tx.send(());
        }
    }

    fn remove(&self, key: &CancelKey) {
        self.inner.lock().remove(key);
    }
}

// ---------------------------------------------------------------------------
// Callback handler
// ---------------------------------------------------------------------------

pub async fn captcha_callback(
    bot: Bot,
    q: CallbackQuery,
    pool: db::Pool,
) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let data = q.data.as_deref().unwrap_or("");
    // Format: captcha_{user_id}_{answer}
    let parts: Vec<&str> = data.splitn(3, '_').collect();
    if parts.len() < 3 {
        return Ok(());
    }

    let target_user_id: i64 = match parts[1].parse() {
        Ok(id) => id,
        Err(_) => return Ok(()),
    };
    let chosen_answer = parts[2];

    // Only the user in question may respond
    if uid_to_i64(q.from.id) != target_user_id {
        bot.answer_callback_query(q.id.clone())
            .text("This captcha is not for you!")
            .await?;
        return Ok(());
    }

    let chat_id = msg.chat().id;
    let attempt = match db::queries::get_captcha_attempt(&pool, target_user_id, chat_id.0).await {
        Ok(Some(a)) => a,
        _ => {
            bot.answer_callback_query(q.id.clone())
                .text("No pending captcha found.")
                .await?;
            return Ok(());
        }
    };

    let lang = i18n::get_chat_lang(&pool, chat_id.0).await;

    if chosen_answer == attempt.answer {
        // Correct! Unmute the user by restoring the chat's default permissions.
        let default_perms = match bot.get_chat(chat_id).await {
            Ok(chat) => chat
                .permissions()
                .unwrap_or_else(ChatPermissions::all),
            Err(e) => {
                log::warn!(
                    "captcha: failed to get chat {} permissions, falling back to all(): {}",
                    chat_id, e
                );
                ChatPermissions::all()
            }
        };
        bot.restrict_chat_member(chat_id, UserId(target_user_id as u64), default_perms)
            .await
            .ok();

        // FIX: Batalkan timeout task agar tidak kick user yang sudah lulus
        CAPTCHA_CANCEL_MAP.take_and_cancel(&(target_user_id, chat_id.0));

        bot.answer_callback_query(q.id.clone())
            .text(i18n::t(&lang, "captcha-correct", None))
            .await?;

        let verified_name = formatting::mention_html(&q.from);
        bot.edit_message_text(
            chat_id,
            msg.id(),
            i18n::t(&lang, "captcha-verified", Some(&[("name", &verified_name)])),
        )
        .parse_mode(ParseMode::Html)
        .await
        .ok();

        db::queries::delete_captcha_attempt(&pool, target_user_id, chat_id.0)
            .await
            .ok();

        // Send welcome message after captcha is solved
        let chat_name = match bot.get_chat(chat_id).await {
            Ok(c) => c.title().unwrap_or("this chat").to_string(),
            Err(_) => "this chat".to_string(),
        };
        let chat_data = db::queries::get_chat(&pool, chat_id.0).await.ok().flatten();
        if let Some(chat) = chat_data {
            if chat.welcome_enabled {
                let welcome_text = formatting::format_greeting(&chat.welcome_text, &q.from, &chat_name);
                bot.send_message(chat_id, welcome_text)
                    .parse_mode(ParseMode::Html)
                    .await
                    .ok();
            }
        }
    } else {
        // Jawaban salah
        let settings = db::queries::get_captcha_settings(&pool, chat_id.0)
            .await
            .unwrap_or_else(|_| db::models::CaptchaSettings {
                chat_id: chat_id.0,
                enabled: false,
                captcha_mode: "math".to_string(),
                timeout_min: 5,
                failure_action: "kick".to_string(),
                max_attempts: 3,
            });

        let current = db::queries::increment_captcha_attempts(&pool, target_user_id, chat_id.0)
            .await
            .unwrap_or(0);

        if current >= settings.max_attempts {
            // Trial limit reached
            // FIX: Cancel the task timeout before executing the action
            CAPTCHA_CANCEL_MAP.take_and_cancel(&(target_user_id, chat_id.0));

            // FIX: Use `safe_kick` instead of `ban` -> `sleep(1)` -> `unban`
            match settings.failure_action.as_str() {
                "ban" => {
                    bot.ban_chat_member(chat_id, UserId(target_user_id as u64))
                        .await
                        .ok();
                }
                "mute" => {
                    // user has been muted
                }
                _ => {
                    safe_kick(&bot, chat_id, UserId(target_user_id as u64)).await;
                }
            }

            bot.answer_callback_query(q.id.clone())
                .text(i18n::t(&lang, "captcha-wrong-max", None))
                .await?;

            let max_str = settings.max_attempts.to_string();
            bot.edit_message_text(
                chat_id,
                msg.id(),
                i18n::t(&lang, "captcha-failed-attempts", Some(&[("max", &max_str), ("action", &settings.failure_action)])),
            )
            .parse_mode(ParseMode::Html)
            .await
            .ok();

            db::queries::delete_captcha_attempt(&pool, target_user_id, chat_id.0)
                .await
                .ok();
        } else {
            let remaining = settings.max_attempts - current;
            let remaining_str = remaining.to_string();
            bot.answer_callback_query(q.id.clone())
                .text(i18n::t(&lang, "captcha-wrong-answer", Some(&[("remaining", &remaining_str)])))
                .await?;
        }
    }

    Ok(())
}