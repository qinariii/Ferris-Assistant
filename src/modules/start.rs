use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::config::AppConfig;
use crate::keyboards::inline;

fn start_text(user_name: &str, bot_name: &str) -> String {
    format!(
        "👋 <b>Hey {}!</b>\n\n\
        I'm <b>{}</b>, a powerful group management bot built with Rust 🦀\n\n\
        I can help you manage your group with features like:\n\
        • Admin management\n\
        • Bans, Mutes & Warns\n\
        • Welcome/Goodbye messages\n\
        • Notes & Filters\n\
        • Antiflood & Blacklist\n\
        • And much more!\n\n\
        Click <b>Help</b> to see all available commands.",
        user_name, bot_name
    )
}

pub async fn start(bot: Bot, msg: Message, cfg: AppConfig) -> ResponseResult<()> {
    let user = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let text = start_text(&user.first_name, &cfg.bot_name);

    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(inline::start_keyboard(&cfg.bot_username))
        .await?;
    Ok(())
}

pub async fn start_back_callback(bot: Bot, q: CallbackQuery, cfg: AppConfig) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let text = start_text(&q.from.first_name, &cfg.bot_name);

    bot.edit_message_text(msg.chat().id, msg.id(), text)
        .parse_mode(ParseMode::Html)
        .reply_markup(inline::start_keyboard(&cfg.bot_username))
        .await?;
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}
