use rand::Rng;
use teloxide::prelude::*;
use teloxide::types::{ChatMemberUpdated, ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

use crate::config::AppConfig;
use crate::db;
use crate::utils::{formatting, formatting::uid_to_i64, permissions};

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

    // Generate 3 wrong answers + correct
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

    // Shuffle
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

pub async fn captcha_cmd(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to use this command.")
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
            "🔐 <b>Captcha Settings</b>\n\n\
            • <b>Status:</b> {}\n\
            • <b>Mode:</b> {}\n\
            • <b>Timeout:</b> {} min\n\
            • <b>Failure action:</b> {}\n\
            • <b>Max attempts:</b> {}\n\n\
            Usage: /captcha &lt;on/off&gt;",
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
            bot.send_message(chat_id, "✅ Captcha verification enabled! New members must solve a captcha.")
                .await?;
        }
        "off" | "disable" | "no" => {
            db::queries::set_captcha_enabled(&pool, chat_id.0, false).await.ok();
            db::queries::delete_all_captcha_attempts(&pool, chat_id.0).await.ok();
            bot.send_message(chat_id, "❌ Captcha verification disabled.")
                .await?;
        }
        _ => {
            bot.send_message(chat_id, "❌ Usage: /captcha <on/off>")
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

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to use this command.")
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
        bot.send_message(chat_id, "❌ Usage: /captchamode <math/text>\n\n• <b>math</b> - Solve a math problem\n• <b>text</b> - Type the shown text")
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let mode = args[0].to_lowercase();
    if mode != "math" && mode != "text" {
        bot.send_message(chat_id, "❌ Invalid mode. Use: math or text")
            .await?;
        return Ok(());
    }

    db::queries::set_captcha_mode(&pool, chat_id.0, &mode).await.ok();
    bot.send_message(chat_id, format!("✅ Captcha mode set to <b>{}</b>.", mode))
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

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to use this command.")
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
        bot.send_message(chat_id, "❌ Usage: /captchatime <1-10>")
            .await?;
        return Ok(());
    }

    let timeout: i64 = match args[0].parse() {
        Ok(v) if (1..=10).contains(&v) => v,
        _ => {
            bot.send_message(chat_id, "❌ Timeout must be between 1 and 10 minutes.")
                .await?;
            return Ok(());
        }
    };

    db::queries::set_captcha_timeout(&pool, chat_id.0, timeout).await.ok();
    bot.send_message(chat_id, format!("✅ Captcha timeout set to <b>{}</b> minutes.", timeout))
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

    if !permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to use this command.")
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
        bot.send_message(chat_id, "❌ Usage: /captchaaction <kick/ban/mute>")
            .await?;
        return Ok(());
    }

    let action = args[0].to_lowercase();
    if !["kick", "ban", "mute"].contains(&action.as_str()) {
        bot.send_message(chat_id, "❌ Invalid action. Use: kick, ban, or mute")
            .await?;
        return Ok(());
    }

    db::queries::set_captcha_action(&pool, chat_id.0, &action).await.ok();
    bot.send_message(chat_id, format!("✅ Captcha failure action set to <b>{}</b>.", action))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

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

    // Mute user until captcha solved
    let perms = ChatPermissions::empty();
    bot.restrict_chat_member(chat_id, user.id, perms).await.ok();

    let (question_text, answer, options) = if settings.captcha_mode == "text" {
        let (ans, opts) = generate_text_captcha();
        let spaced: String = ans.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
        (
            format!("Select the code: <code>{}</code>", spaced),
            ans.clone(),
            opts,
        )
    } else {
        let (q, ans, opts) = generate_math_captcha();
        (
            format!("Solve: <b>{}</b> = ?", q),
            ans,
            opts,
        )
    };

    let user_mention = formatting::mention_html(user);
    let text = format!(
        "🔐 <b>Captcha Verification</b>\n\n\
        Welcome {}! Please solve this to verify you're human.\n\n\
        {}\n\n\
        ⏱ You have <b>{}</b> minute(s). Max <b>{}</b> attempts.",
        user_mention,
        question_text,
        settings.timeout_min,
        settings.max_attempts,
    );

    let keyboard = build_captcha_keyboard(&options, uid_to_i64(user.id));

    let sent = bot
        .send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    // Store attempt in DB
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

    // Spawn timeout task
    let bot_clone = bot.clone();
    let pool_clone = pool.clone();
    let user_id = user.id;
    let failure_action = settings.failure_action.clone();
    let timeout_min = settings.timeout_min;
    let msg_id = sent.id;

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(timeout_min as u64 * 60)).await;

        // Check if attempt still exists (user hasn't solved it)
        if let Ok(Some(_attempt)) =
            db::queries::get_captcha_attempt(&pool_clone, uid_to_i64(user_id), chat_id.0).await
        {
            // Apply failure action
            match failure_action.as_str() {
                "ban" => {
                    bot_clone.ban_chat_member(chat_id, user_id).await.ok();
                }
                "mute" => {
                    // Already muted, just inform
                }
                _ => {
                    // kick
                    bot_clone.ban_chat_member(chat_id, user_id).await.ok();
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    bot_clone.unban_chat_member(chat_id, user_id).await.ok();
                }
            }

            bot_clone
                .send_message(
                    chat_id,
                    format!(
                        "⏱ User {} failed to complete captcha in time. Action: <b>{}</b>",
                        user_id.0, failure_action
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
    });

    Ok(())
}

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

    // Only the target user can answer
    if uid_to_i64(q.from.id) != target_user_id {
        bot.answer_callback_query(q.id.clone())
            .text("❌ This captcha is not for you!")
            .await?;
        return Ok(());
    }

    let chat_id = msg.chat().id;
    let attempt = match db::queries::get_captcha_attempt(&pool, target_user_id, chat_id.0).await {
        Ok(Some(a)) => a,
        _ => {
            bot.answer_callback_query(q.id.clone())
                .text("❌ No pending captcha found.")
                .await?;
            return Ok(());
        }
    };

    if chosen_answer == attempt.answer {
        // Correct! Unmute user
        let all_perms = ChatPermissions::all();
        bot.restrict_chat_member(chat_id, UserId(target_user_id as u64), all_perms)
            .await
            .ok();

        bot.answer_callback_query(q.id.clone())
            .text("✅ Correct! Welcome!")
            .await?;

        bot.edit_message_text(
            chat_id,
            msg.id(),
            format!("✅ {} has been verified!", formatting::mention_html(&q.from)),
        )
        .parse_mode(ParseMode::Html)
        .await
        .ok();

        db::queries::delete_captcha_attempt(&pool, target_user_id, chat_id.0)
            .await
            .ok();
    } else {
        // Wrong answer
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
            // Max attempts reached
            match settings.failure_action.as_str() {
                "ban" => {
                    bot.ban_chat_member(chat_id, UserId(target_user_id as u64))
                        .await
                        .ok();
                }
                "mute" => {
                    // Already muted
                }
                _ => {
                    bot.ban_chat_member(chat_id, UserId(target_user_id as u64))
                        .await
                        .ok();
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    bot.unban_chat_member(chat_id, UserId(target_user_id as u64))
                        .await
                        .ok();
                }
            }

            bot.answer_callback_query(q.id.clone())
                .text("❌ Too many wrong attempts!")
                .await?;

            bot.edit_message_text(
                chat_id,
                msg.id(),
                format!(
                    "❌ User failed captcha after {} attempts. Action: <b>{}</b>",
                    settings.max_attempts, settings.failure_action
                ),
            )
            .parse_mode(ParseMode::Html)
            .await
            .ok();

            db::queries::delete_captcha_attempt(&pool, target_user_id, chat_id.0)
                .await
                .ok();
        } else {
            let remaining = settings.max_attempts - current;
            bot.answer_callback_query(q.id.clone())
                .text(format!(
                    "❌ Wrong! {} attempt(s) remaining.",
                    remaining
                ))
                .await?;
        }
    }

    Ok(())
}
