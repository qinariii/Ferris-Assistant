use teloxide::dispatching::{UpdateFilterExt, UpdateHandler};
use teloxide::dptree;
use teloxide::prelude::*;
use teloxide::types::{ChatMemberUpdated, Message, Update};

use crate::config::AppConfig;
use crate::db;
use crate::utils::{formatting, i18n, permissions, LogErrExt};
use crate::modules::{
    admin, afk, antiflood, antispam, backups, bans, blacklist, blstickers, captcha, chatpermissions,
    cleaner, connections, devs, disable, feds, filters, gbans, help, locks, log_channel, mutes, notes,
    pins, purges, reactions, reports, rules, sed, start, userinfo, users, warns, welcome,
};

pub fn build_handler() -> UpdateHandler<teloxide::RequestError> {
    let command_handler = Update::filter_message()
        .filter_command::<Command>()
        // Intercept disabled commands before dispatching
        .branch(
            dptree::filter_async(|msg: Message, pool: db::Pool| async move {
                if msg.chat.is_private() {
                    return false;
                }
                let text = msg.text().unwrap_or("");
                let cmd = text
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('/')
                    .split('@')
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
                disable::is_disabled(&pool, msg.chat.id.0, &cmd).await
            })
            .endpoint(|bot: Bot, msg: Message| async move {
                bot.send_message(msg.chat.id, "❌ This command is disabled in this chat.")
                    .await?;
                respond(())
            }),
        )
        .branch(dptree::case![Command::Start].endpoint(start::start))
        .branch(dptree::case![Command::Help].endpoint(help::help_command))
        // Admin
        .branch(dptree::case![Command::Promote].endpoint(admin::promote))
        .branch(dptree::case![Command::Demote].endpoint(admin::demote))
        .branch(dptree::case![Command::Adminlist].endpoint(admin::adminlist))
        .branch(dptree::case![Command::Title].endpoint(admin::set_title))
        // Bans
        .branch(dptree::case![Command::Ban].endpoint(bans::ban))
        .branch(dptree::case![Command::Unban].endpoint(bans::unban))
        .branch(dptree::case![Command::Kick].endpoint(bans::kick))
        .branch(dptree::case![Command::Dban].endpoint(bans::dban))
        .branch(dptree::case![Command::Dkick].endpoint(bans::dkick))
        .branch(dptree::case![Command::Tban].endpoint(bans::tban))
        // Mutes
        .branch(dptree::case![Command::Mute].endpoint(mutes::mute))
        .branch(dptree::case![Command::Unmute].endpoint(mutes::unmute))
        .branch(dptree::case![Command::Tmute].endpoint(mutes::tmute))
        // Warns
        .branch(dptree::case![Command::Warn].endpoint(warns::warn))
        .branch(dptree::case![Command::Warns].endpoint(warns::warns))
        .branch(dptree::case![Command::Resetwarns].endpoint(warns::resetwarns))
        .branch(dptree::case![Command::Setwarnlimit].endpoint(warns::setwarnlimit))
        .branch(dptree::case![Command::Setwarnmode].endpoint(warns::setwarnmode))
        // Notes
        .branch(dptree::case![Command::Save].endpoint(notes::save))
        .branch(dptree::case![Command::Get].endpoint(notes::get))
        .branch(dptree::case![Command::Notes].endpoint(notes::notes_list))
        .branch(dptree::case![Command::Clear].endpoint(notes::clear))
        // Filters
        .branch(dptree::case![Command::Filter].endpoint(filters::add_filter))
        .branch(dptree::case![Command::Filters].endpoint(filters::list_filters))
        .branch(dptree::case![Command::Stop].endpoint(filters::stop_filter))
        .branch(dptree::case![Command::Stopall].endpoint(filters::stopall))
        // Welcome
        .branch(dptree::case![Command::Welcome].endpoint(welcome::welcome_toggle))
        .branch(dptree::case![Command::Setwelcome].endpoint(welcome::set_welcome))
        .branch(dptree::case![Command::Goodbye].endpoint(welcome::goodbye_toggle))
        .branch(dptree::case![Command::Setgoodbye].endpoint(welcome::set_goodbye))
        // Rules
        .branch(dptree::case![Command::Rules].endpoint(rules::rules))
        .branch(dptree::case![Command::Setrules].endpoint(rules::setrules))
        .branch(dptree::case![Command::Clearrules].endpoint(rules::clearrules))
        // Blacklist
        .branch(dptree::case![Command::Blacklist].endpoint(blacklist::blacklist))
        .branch(dptree::case![Command::Addblacklist].endpoint(blacklist::add_blacklist))
        .branch(dptree::case![Command::Rmblacklist].endpoint(blacklist::rm_blacklist))
        .branch(dptree::case![Command::Blacklistmode].endpoint(blacklist::blacklist_mode))
        // Purges
        .branch(dptree::case![Command::Purge].endpoint(purges::purge))
        .branch(dptree::case![Command::Del].endpoint(purges::del))
        // Pins
        .branch(dptree::case![Command::Pin].endpoint(pins::pin))
        .branch(dptree::case![Command::Unpin].endpoint(pins::unpin))
        .branch(dptree::case![Command::Unpinall].endpoint(pins::unpinall))
        // Antiflood
        .branch(dptree::case![Command::Setflood].endpoint(antiflood::set_flood))
        .branch(dptree::case![Command::Flood].endpoint(antiflood::flood_info))
        .branch(dptree::case![Command::Setfloodmode].endpoint(antiflood::set_flood_mode))
        // Disable
        .branch(dptree::case![Command::Disable].endpoint(disable::disable))
        .branch(dptree::case![Command::Enable].endpoint(disable::enable))
        .branch(dptree::case![Command::Disabled].endpoint(disable::disabled_list))
        // Locks
        .branch(dptree::case![Command::Lock].endpoint(locks::lock))
        .branch(dptree::case![Command::Unlock].endpoint(locks::unlock))
        .branch(dptree::case![Command::Locks].endpoint(locks::locks_list))
        .branch(dptree::case![Command::Locktypes].endpoint(locks::locktypes))
        // Log Channel
        .branch(dptree::case![Command::Logchannel].endpoint(log_channel::log_channel))
        .branch(dptree::case![Command::Setlogchannel].endpoint(log_channel::set_log_channel))
        .branch(dptree::case![Command::Unsetlogchannel].endpoint(log_channel::unset_log_channel))
        // Reports
        .branch(dptree::case![Command::Report].endpoint(reports::report))
        .branch(dptree::case![Command::Reports].endpoint(reports::report_setting))
        // Global Bans
        .branch(dptree::case![Command::Gban].endpoint(gbans::gban))
        .branch(dptree::case![Command::Ungban].endpoint(gbans::ungban))
        .branch(dptree::case![Command::Gbanlist].endpoint(gbans::gbanlist))
        // Backups
        .branch(dptree::case![Command::Export].endpoint(backups::export))
        .branch(dptree::case![Command::Import].endpoint(backups::import))
        // Connections
        .branch(dptree::case![Command::Connect].endpoint(connections::connect))
        .branch(dptree::case![Command::Disconnect].endpoint(connections::disconnect))
        .branch(dptree::case![Command::Connection].endpoint(connections::connection_status))
        // AFK
        .branch(dptree::case![Command::Afk].endpoint(afk::afk))
        // Blacklist Stickers
        .branch(dptree::case![Command::Blsticker].endpoint(blstickers::blsticker_list))
        .branch(dptree::case![Command::Addblsticker].endpoint(blstickers::add_blsticker))
        .branch(dptree::case![Command::Rmblsticker].endpoint(blstickers::rm_blsticker))
        .branch(dptree::case![Command::Blstickermode].endpoint(blstickers::blsticker_mode))
        // Chat Permissions
        .branch(dptree::case![Command::Setpermissions].endpoint(chatpermissions::set_permissions))
        .branch(dptree::case![Command::Permissions].endpoint(chatpermissions::view_permissions))
        // Users / Stats
        .branch(dptree::case![Command::Stats].endpoint(users::stats))
        .branch(dptree::case![Command::Chatlist].endpoint(users::chatlist))
        // Misc
        .branch(dptree::case![Command::Id].endpoint(misc_id))
        .branch(dptree::case![Command::Info].endpoint(misc_info))
        .branch(dptree::case![Command::Settings].endpoint(misc_settings))
        .branch(dptree::case![Command::Setlang].endpoint(misc_setlang))
        // Captcha
        .branch(dptree::case![Command::Captcha].endpoint(captcha::captcha_cmd))
        .branch(dptree::case![Command::Captchamode].endpoint(captcha::captchamode_cmd))
        .branch(dptree::case![Command::Captchatime].endpoint(captcha::captchatime_cmd))
        .branch(dptree::case![Command::Captchaaction].endpoint(captcha::captchaaction_cmd))
        // Devs
        .branch(dptree::case![Command::Addsudo].endpoint(devs::addsudo))
        .branch(dptree::case![Command::Remsudo].endpoint(devs::remsudo))
        .branch(dptree::case![Command::Adddev].endpoint(devs::adddev))
        .branch(dptree::case![Command::Remdev].endpoint(devs::remdev))
        .branch(dptree::case![Command::Teamusers].endpoint(devs::teamusers))
        .branch(dptree::case![Command::Chatinfo].endpoint(devs::chatinfo))
        .branch(dptree::case![Command::Leavechat].endpoint(devs::leavechat))
        .branch(dptree::case![Command::Botstats].endpoint(devs::botstats))
        .branch(dptree::case![Command::Broadcast].endpoint(devs::broadcast))
        // Federations
        .branch(dptree::case![Command::Newfed].endpoint(feds::newfed))
        .branch(dptree::case![Command::Delfed].endpoint(feds::delfed))
        .branch(dptree::case![Command::Joinfed].endpoint(feds::joinfed))
        .branch(dptree::case![Command::Leavefed].endpoint(feds::leavefed))
        .branch(dptree::case![Command::Fedinfo].endpoint(feds::fedinfo))
        .branch(dptree::case![Command::Fedpromote].endpoint(feds::fedpromote))
        .branch(dptree::case![Command::Feddemote].endpoint(feds::feddemote))
        .branch(dptree::case![Command::Fban].endpoint(feds::fban))
        .branch(dptree::case![Command::Unfban].endpoint(feds::unfban))
        .branch(dptree::case![Command::Fbanlist].endpoint(feds::fbanlist))
        .branch(dptree::case![Command::Fedchat].endpoint(feds::fedchat))
        .branch(dptree::case![Command::Fedrules].endpoint(feds::fedrules))
        // Userinfo
        .branch(dptree::case![Command::Setme].endpoint(userinfo::setme))
        .branch(dptree::case![Command::Me].endpoint(userinfo::me))
        .branch(dptree::case![Command::Setbio].endpoint(userinfo::setbio))
        .branch(dptree::case![Command::Bio].endpoint(userinfo::bio))
        // Cleaner
        .branch(dptree::case![Command::Cleanservice].endpoint(cleaner::cleanservice))
        .branch(dptree::case![Command::Cleanbluetext].endpoint(cleaner::cleanbluetext))
        // Reactions
        .branch(dptree::case![Command::Addreaction].endpoint(reactions::addreaction))
        .branch(dptree::case![Command::Removereaction].endpoint(reactions::removereaction))
        .branch(dptree::case![Command::Reactions].endpoint(reactions::reactions_list))
        .branch(dptree::case![Command::Resetreactions].endpoint(reactions::resetreactions));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(dptree::endpoint(handle_message));

    let callback_handler = Update::filter_callback_query().branch(dptree::endpoint(handle_callback));

    let chat_member_handler = Update::filter_chat_member()
        .branch(dptree::endpoint(handle_chat_member));

    dptree::entry()
        .branch(message_handler)
        .branch(callback_handler)
        .branch(chat_member_handler)
}

#[derive(teloxide::utils::command::BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Show help")]
    Help,
    // Admin
    #[command(description = "Promote a user")]
    Promote,
    #[command(description = "Demote a user")]
    Demote,
    #[command(description = "List admins")]
    Adminlist,
    #[command(description = "Set admin title")]
    Title,
    // Bans
    #[command(description = "Ban a user")]
    Ban,
    #[command(description = "Unban a user")]
    Unban,
    #[command(description = "Kick a user")]
    Kick,
    #[command(description = "Delete msg & ban")]
    Dban,
    #[command(description = "Delete msg & kick")]
    Dkick,
    #[command(description = "Temp ban a user")]
    Tban,
    // Mutes
    #[command(description = "Mute a user")]
    Mute,
    #[command(description = "Unmute a user")]
    Unmute,
    #[command(description = "Temp mute a user")]
    Tmute,
    // Warns
    #[command(description = "Warn a user")]
    Warn,
    #[command(description = "Check warns")]
    Warns,
    #[command(description = "Reset warns")]
    Resetwarns,
    #[command(description = "Set warn limit")]
    Setwarnlimit,
    #[command(description = "Set warn mode")]
    Setwarnmode,
    // Notes
    #[command(description = "Save a note")]
    Save,
    #[command(description = "Get a note")]
    Get,
    #[command(description = "List notes")]
    Notes,
    #[command(description = "Clear a note")]
    Clear,
    // Filters
    #[command(description = "Add a filter")]
    Filter,
    #[command(description = "List filters")]
    Filters,
    #[command(description = "Remove a filter")]
    Stop,
    #[command(description = "Remove all filters")]
    Stopall,
    // Welcome
    #[command(description = "Toggle welcome")]
    Welcome,
    #[command(description = "Set welcome text")]
    Setwelcome,
    #[command(description = "Toggle goodbye")]
    Goodbye,
    #[command(description = "Set goodbye text")]
    Setgoodbye,
    // Rules
    #[command(description = "View rules")]
    Rules,
    #[command(description = "Set rules")]
    Setrules,
    #[command(description = "Clear rules")]
    Clearrules,
    // Blacklist
    #[command(description = "View blacklist")]
    Blacklist,
    #[command(description = "Add to blacklist")]
    Addblacklist,
    #[command(description = "Remove from blacklist")]
    Rmblacklist,
    #[command(description = "Set blacklist mode")]
    Blacklistmode,
    // Purges
    #[command(description = "Purge messages")]
    Purge,
    #[command(description = "Delete a message")]
    Del,
    // Pins
    #[command(description = "Pin a message")]
    Pin,
    #[command(description = "Unpin a message")]
    Unpin,
    #[command(description = "Unpin all")]
    Unpinall,
    // Antiflood
    #[command(description = "Set flood limit")]
    Setflood,
    #[command(description = "View flood settings")]
    Flood,
    #[command(description = "Set flood mode")]
    Setfloodmode,
    // Disable
    #[command(description = "Disable a command")]
    Disable,
    #[command(description = "Enable a command")]
    Enable,
    #[command(description = "List disabled")]
    Disabled,
    // Locks
    #[command(description = "Lock a message type")]
    Lock,
    #[command(description = "Unlock a message type")]
    Unlock,
    #[command(description = "View locks")]
    Locks,
    #[command(description = "List lock types")]
    Locktypes,
    // Log Channel
    #[command(description = "View log channel")]
    Logchannel,
    #[command(description = "Set log channel")]
    Setlogchannel,
    #[command(description = "Unset log channel")]
    Unsetlogchannel,
    // Reports
    #[command(description = "Report a message")]
    Report,
    #[command(description = "Toggle reporting")]
    Reports,
    // Global Bans
    #[command(description = "Global ban a user")]
    Gban,
    #[command(description = "Remove global ban")]
    Ungban,
    #[command(description = "List global bans")]
    Gbanlist,
    // Backups
    #[command(description = "Export settings")]
    Export,
    #[command(description = "Import settings")]
    Import,
    // Connections
    #[command(description = "Connect to a group")]
    Connect,
    #[command(description = "Disconnect")]
    Disconnect,
    #[command(description = "Connection status")]
    Connection,
    // AFK
    #[command(description = "Set AFK status")]
    Afk,
    // Blacklist Stickers
    #[command(description = "View blacklisted stickers")]
    Blsticker,
    #[command(description = "Add sticker to blacklist")]
    Addblsticker,
    #[command(description = "Remove sticker from blacklist")]
    Rmblsticker,
    #[command(description = "Set sticker blacklist mode")]
    Blstickermode,
    // Chat Permissions
    #[command(description = "Set chat permissions")]
    Setpermissions,
    #[command(description = "View chat permissions")]
    Permissions,
    // Users / Stats
    #[command(description = "Bot statistics")]
    Stats,
    #[command(description = "List known chats")]
    Chatlist,
    // Misc
    #[command(description = "Get ID")]
    Id,
    #[command(description = "User info")]
    Info,
    #[command(description = "Bot settings")]
    Settings,
    #[command(description = "Set language")]
    Setlang,
    // Captcha
    #[command(description = "Toggle captcha")]
    Captcha,
    #[command(description = "Set captcha mode")]
    Captchamode,
    #[command(description = "Set captcha timeout")]
    Captchatime,
    #[command(description = "Set captcha failure action")]
    Captchaaction,
    // Devs
    #[command(description = "Add sudo user")]
    Addsudo,
    #[command(description = "Remove sudo user")]
    Remsudo,
    #[command(description = "Add dev user")]
    Adddev,
    #[command(description = "Remove dev user")]
    Remdev,
    #[command(description = "List team members")]
    Teamusers,
    #[command(description = "Get chat info")]
    Chatinfo,
    #[command(description = "Leave a chat")]
    Leavechat,
    #[command(description = "Bot statistics")]
    Botstats,
    #[command(description = "Broadcast message")]
    Broadcast,
    // Federations
    #[command(description = "Create a federation")]
    Newfed,
    #[command(description = "Delete a federation")]
    Delfed,
    #[command(description = "Join a federation")]
    Joinfed,
    #[command(description = "Leave a federation")]
    Leavefed,
    #[command(description = "Federation info")]
    Fedinfo,
    #[command(description = "Promote fed admin")]
    Fedpromote,
    #[command(description = "Demote fed admin")]
    Feddemote,
    #[command(description = "Federation ban")]
    Fban,
    #[command(description = "Remove federation ban")]
    Unfban,
    #[command(description = "Federation ban list")]
    Fbanlist,
    #[command(description = "Check chat's federation")]
    Fedchat,
    #[command(description = "View/set fed rules")]
    Fedrules,
    // Userinfo
    #[command(description = "Set your info")]
    Setme,
    #[command(description = "Get user info")]
    Me,
    #[command(description = "Set user bio")]
    Setbio,
    #[command(description = "Get user bio")]
    Bio,
    // Cleaner
    #[command(description = "Toggle service msg cleaning")]
    Cleanservice,
    #[command(description = "Toggle blue text cleaning")]
    Cleanbluetext,
    // Reactions
    #[command(description = "Add a reaction")]
    Addreaction,
    #[command(description = "Remove a reaction")]
    Removereaction,
    #[command(description = "List reactions")]
    Reactions,
    #[command(description = "Clear all reactions")]
    Resetreactions,
}

async fn handle_message(
    bot: Bot,
    msg: Message,
    cfg: AppConfig,
    pool: db::Pool,
    flood_tracker: antiflood::FloodTracker,
) -> ResponseResult<()> {
    let is_private = msg.chat.is_private();

    // rate-limit check — bail before any DB work
    if antispam::check_antispam(bot.clone(), msg.clone(), cfg.clone())
        .await
        .unwrap_or(false)
    {
        return Ok(());
    }

    if let Err(e) = users::log_users(bot.clone(), msg.clone(), pool.clone()).await {
        log::error!("[users::log_users] {}", e);
    }

    if let Some(text) = msg.text() {
        if text.starts_with('#') {
            notes::hash_get(bot.clone(), msg.clone(), pool.clone())
                .await
                .ok();
        }
        if text.contains("@admin") && !is_private {
            reports::check_admin_tag(bot.clone(), msg.clone(), pool.clone())
                .await
                .ok();
        }
    }

    if let Err(e) = sed::check_sed(bot.clone(), msg.clone()).await {
        log::error!("[sed::check_sed] {}", e);
    }

    // afk runs in both private and group
    if let Err(e) = afk::check_no_longer_afk(bot.clone(), msg.clone(), pool.clone()).await {
        log::error!("[afk::check_no_longer_afk] {}", e);
    }
    if let Err(e) = afk::check_afk_reply(bot.clone(), msg.clone(), pool.clone()).await {
        log::error!("[afk::check_afk_reply] {}", e);
    }

    // group-only checks from here on
    if is_private {
        return Ok(());
    }

    if let Err(e) = antiflood::check_flood(
        bot.clone(),
        msg.clone(),
        cfg.clone(),
        pool.clone(),
        flood_tracker,
    )
    .await
    {
        log::error!("[antiflood::check_flood] {}", e);
    }

    if let Err(e) = gbans::check_gban(bot.clone(), msg.clone(), pool.clone()).await {
        log::error!("[gbans::check_gban] {}", e);
    }
    if let Err(e) = feds::check_fban(bot.clone(), msg.clone(), pool.clone()).await {
        log::error!("[feds::check_fban] {}", e);
    }

    match locks::check_locks(bot.clone(), msg.clone(), pool.clone()).await {
        Ok(true) => return Ok(()),
        Err(e) => log::error!("[locks::check_locks] {}", e),
        _ => {}
    }

    match blstickers::check_blacklist_sticker(bot.clone(), msg.clone(), pool.clone()).await {
        Ok(true) => return Ok(()),
        Err(e) => log::error!("[blstickers::check_blacklist_sticker] {}", e),
        _ => {}
    }

    match blacklist::check_blacklist(bot.clone(), msg.clone(), cfg.clone(), pool.clone()).await {
        Ok(true) => return Ok(()),
        Err(e) => log::error!("[blacklist::check_blacklist] {}", e),
        _ => {}
    }

    if let Err(e) = filters::check_filters(bot.clone(), msg.clone(), pool.clone()).await {
        log::error!("[filters::check_filters] {}", e);
    }

    if let Err(e) = reactions::check_reactions(bot.clone(), msg.clone(), pool.clone()).await {
        log::error!("[reactions::check_reactions] {}", e);
    }

    if let Err(e) =
        cleaner::check_service_message(bot.clone(), msg.clone(), cfg.clone(), pool.clone()).await
    {
        log::error!("[cleaner::check_service_message] {}", e);
    }

    Ok(())
}

async fn handle_callback(bot: Bot, q: CallbackQuery, cfg: AppConfig, pool: db::Pool) -> ResponseResult<()> {
    let data = q.data.as_deref().unwrap_or("");

    match data {
        // Navigation
        "start_back" => start::start_back_callback(bot, q, cfg).await?,
        "help_main" => help::help_main_callback(bot, q).await?,
        "about" => help::about_callback(bot, q).await?,
        "close" => help::close_callback(bot, q).await?,
        "settings" => {
            let msg = match q.message {
                Some(ref m) => m.clone(),
                None => return Ok(()),
            };
            bot.edit_message_text(msg.chat().id, msg.id(), "⚙️ <b>Settings</b>\n\nChoose a setting to configure:")
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(crate::keyboards::inline::settings_keyboard())
                .await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }

        // Help modules
        d if d.starts_with("help_") => help::help_module_callback(bot, q).await?,

        // Unban
        d if d.starts_with("unban_") => bans::unban_callback(bot, q, cfg, pool).await?,

        // Unmute
        d if d.starts_with("unmute_") => mutes::unmute_callback(bot, q, pool).await?,

        // Remove warn
        d if d.starts_with("rmwarn_") => warns::rmwarn_callback(bot, q, pool).await?,

        // Warn mode
        d if d.starts_with("warnmode_") => warns::warnmode_callback(bot, q, pool).await?,

        // Antiflood
        d if d.starts_with("af_") => antiflood::antiflood_callback(bot, q, pool).await?,

        // Captcha
        d if d.starts_with("captcha_") => captcha::captcha_callback(bot, q, pool).await?,

        // Rules
        d if d.starts_with("rules_") => rules::rules_callback(bot, q, pool).await?,

        // Language
        d if d.starts_with("lang_") => {
            handle_lang_callback(bot, q, pool).await?;
        }

        // Welcome/Goodbye settings panels
        "set_welcome" => {
            let msg = match q.message {
                Some(ref m) => m.clone(),
                None => return Ok(()),
            };
            let chat_data = db::queries::get_chat(&pool, msg.chat().id.0).await.ok().flatten();
            let enabled = chat_data.map(|c| c.welcome_enabled).unwrap_or(true);
            bot.edit_message_text(msg.chat().id, msg.id(), "👋 <b>Welcome Settings</b>")
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(crate::keyboards::inline::welcome_settings_keyboard(enabled))
                .await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "welcome_toggle" => {
            let msg = match q.message {
                Some(ref m) => m.clone(),
                None => return Ok(()),
            };
            let chat_id = msg.chat().id;
            if !permissions::is_user_admin(&bot, chat_id, q.from.id).await {
                bot.answer_callback_query(q.id.clone()).text("❌ Admin only").await?;
                return Ok(());
            }
            let chat_data = db::queries::get_chat(&pool, chat_id.0).await.ok().flatten();
            let currently_enabled = chat_data.map(|c| c.welcome_enabled).unwrap_or(true);
            let new_state = !currently_enabled;
            db::queries::toggle_welcome(&pool, chat_id.0, new_state).await.ok();
            bot.edit_message_text(chat_id, msg.id(), "👋 <b>Welcome Settings</b>")
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(crate::keyboards::inline::welcome_settings_keyboard(new_state))
                .await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "welcome_set" => {
            bot.answer_callback_query(q.id.clone())
                .text("Use /setwelcome <text> to set the welcome message.")
                .await?;
        }
        "welcome_preview" => {
            let msg = match q.message {
                Some(ref m) => m.clone(),
                None => return Ok(()),
            };
            let chat_id = msg.chat().id;
            let chat_data = db::queries::get_chat(&pool, chat_id.0).await.ok().flatten();
            if let Some(chat) = chat_data {
                let chat_name = msg.chat().title().unwrap_or("this chat");
                let preview = formatting::format_greeting(&chat.welcome_text, &q.from, chat_name);
                bot.send_message(chat_id, preview)
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .await?;
            }
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "set_goodbye" => {
            let msg = match q.message {
                Some(ref m) => m.clone(),
                None => return Ok(()),
            };
            let chat_data = db::queries::get_chat(&pool, msg.chat().id.0).await.ok().flatten();
            let enabled = chat_data.map(|c| c.goodbye_enabled).unwrap_or(true);
            bot.edit_message_text(msg.chat().id, msg.id(), "👋 <b>Goodbye Settings</b>")
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(crate::keyboards::inline::goodbye_settings_keyboard(enabled))
                .await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "goodbye_toggle" => {
            let msg = match q.message {
                Some(ref m) => m.clone(),
                None => return Ok(()),
            };
            let chat_id = msg.chat().id;
            if !permissions::is_user_admin(&bot, chat_id, q.from.id).await {
                bot.answer_callback_query(q.id.clone()).text("❌ Admin only").await?;
                return Ok(());
            }
            let chat_data = db::queries::get_chat(&pool, chat_id.0).await.ok().flatten();
            let currently_enabled = chat_data.map(|c| c.goodbye_enabled).unwrap_or(true);
            let new_state = !currently_enabled;
            db::queries::toggle_goodbye(&pool, chat_id.0, new_state).await.ok();
            bot.edit_message_text(chat_id, msg.id(), "👋 <b>Goodbye Settings</b>")
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(crate::keyboards::inline::goodbye_settings_keyboard(new_state))
                .await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "goodbye_set" => {
            bot.answer_callback_query(q.id.clone())
                .text("Use /setgoodbye <text> to set the goodbye message.")
                .await?;
        }
        "goodbye_preview" => {
            let msg = match q.message {
                Some(ref m) => m.clone(),
                None => return Ok(()),
            };
            let chat_id = msg.chat().id;
            let chat_data = db::queries::get_chat(&pool, chat_id.0).await.ok().flatten();
            if let Some(chat) = chat_data {
                let chat_name = msg.chat().title().unwrap_or("this chat");
                let preview = formatting::format_greeting(&chat.goodbye_text, &q.from, chat_name);
                bot.send_message(chat_id, preview)
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .await?;
            }
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "set_blacklist" => {
            let msg = match q.message {
                Some(ref m) => m.clone(),
                None => return Ok(()),
            };
            bot.edit_message_text(msg.chat().id, msg.id(), "🚫 <b>Blacklist Settings</b>\n\nUse commands:\n/blacklist - View blacklisted words\n/addblacklist <word> - Add to blacklist\n/rmblacklist <word> - Remove\n/blacklistmode <mode> - Set action")
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(crate::keyboards::inline::back_to_help_keyboard())
                .await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "set_warnmode" => {
            let msg = match q.message {
                Some(ref m) => m.clone(),
                None => return Ok(()),
            };
            bot.edit_message_text(msg.chat().id, msg.id(), "⚠️ <b>Warn Mode</b>\n\nChoose what happens when a user exceeds the warn limit:")
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(crate::keyboards::inline::warn_mode_keyboard())
                .await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "set_antiflood" => {
            let msg = match q.message {
                Some(ref m) => m.clone(),
                None => return Ok(()),
            };
            bot.edit_message_text(msg.chat().id, msg.id(), "🌊 <b>Antiflood Settings</b>\n\nSet the message limit before action:")
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(crate::keyboards::inline::antiflood_keyboard())
                .await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "set_language" => {
            let msg = match q.message {
                Some(ref m) => m.clone(),
                None => return Ok(()),
            };
            bot.edit_message_text(msg.chat().id, msg.id(), "🌐 <b>Language</b>\n\nSelect bot language:")
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(crate::keyboards::inline::language_keyboard())
                .await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }

        _ => {
            bot.answer_callback_query(q.id.clone()).await?;
        }
    }
    Ok(())
}

async fn handle_chat_member(
    bot: Bot,
    update: ChatMemberUpdated,
    pool: db::Pool,
) -> ResponseResult<()> {
    let new = &update.new_chat_member;
    let old = &update.old_chat_member;

    if !old.is_present() && new.is_present() {
        // New member joined — send captcha if enabled, skip welcome if captcha active
        let captcha_settings = db::queries::get_captcha_settings(&pool, update.chat.id.0)
            .await
            .unwrap_or_else(|_| db::models::CaptchaSettings {
                chat_id: update.chat.id.0,
                enabled: false,
                captcha_mode: "math".to_string(),
                timeout_min: 5,
                failure_action: "kick".to_string(),
                max_attempts: 3,
            });

        if captcha_settings.enabled {
            captcha::send_captcha_on_join(bot.clone(), &update, pool.clone())
                .await
                .log_err("captcha::send_on_join");
        } else {
            welcome::welcome_new_member(bot, update, pool).await?;
        }
    } else if old.is_present() && !new.is_present() {
        // Member left
        welcome::goodbye_member(bot, update, pool).await?;
    }

    Ok(())
}

async fn misc_id(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let mut text = format!("💬 <b>Chat ID:</b> <code>{}</code>", chat_id.0);

    if let Some(from) = msg.from.as_ref() {
        text.push_str(&format!("\n👤 <b>Your ID:</b> <code>{}</code>", from.id.0));
    }

    if let Some(reply) = msg.reply_to_message() {
        if let Some(user) = reply.from.as_ref() {
            text.push_str(&format!(
                "\n↩️ <b>Replied user ID:</b> <code>{}</code>",
                user.id.0
            ));
        }
    }

    bot.send_message(chat_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}

async fn misc_info(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    let user = if let Some(reply) = msg.reply_to_message() {
        reply.from.as_ref().cloned()
    } else {
        msg.from.as_ref().cloned()
    };

    let user = match user {
        Some(u) => u,
        None => return Ok(()),
    };

    let mut text = format!(
        "👤 <b>User Info:</b>\n\n\
        • <b>ID:</b> <code>{}</code>\n\
        • <b>First Name:</b> {}\n",
        user.id.0,
        crate::utils::formatting::html_escape(&user.first_name)
    );

    if let Some(ref last) = user.last_name {
        text.push_str(&format!(
            "• <b>Last Name:</b> {}\n",
            crate::utils::formatting::html_escape(last)
        ));
    }

    if let Some(ref username) = user.username {
        text.push_str(&format!("• <b>Username:</b> @{}\n", username));
    }

    text.push_str(&format!(
        "• <b>User Link:</b> <a href=\"tg://user?id={}\">Link</a>\n",
        user.id.0
    ));

    bot.send_message(chat_id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}

async fn misc_settings(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, "⚙️ <b>Settings</b>\n\nChoose a setting to configure:")
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(crate::keyboards::inline::settings_keyboard())
        .await?;
    Ok(())
}

async fn misc_setlang(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    if !crate::utils::permissions::is_user_admin(&bot, chat_id, from.id).await {
        bot.send_message(chat_id, "❌ You need to be an admin to change the language.")
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
        bot.send_message(chat_id, "🌐 Select a language:")
            .reply_markup(crate::keyboards::inline::language_keyboard())
            .await?;
        return Ok(());
    }

    let lang = args[0].to_lowercase();
    if !["en", "id"].contains(&lang.as_str()) {
        bot.send_message(chat_id, "❌ Supported languages: en, id")
            .await?;
        return Ok(());
    }

    let chat_name = msg.chat.title().unwrap_or("Private");
    db::queries::upsert_chat(&pool, chat_id.0, chat_name).await.log_err("setlang::upsert_chat");
    db::queries::set_language(&pool, chat_id.0, &lang).await.log_err("setlang::set_language");

    let lang_name = match lang.as_str() {
        "en" => "English 🇬🇧",
        "id" => "Indonesia 🇮🇩",
        _ => &lang,
    };

    bot.send_message(chat_id, format!("✅ Language set to <b>{}</b>.", lang_name))
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}

async fn handle_lang_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let data = q.data.as_deref().unwrap_or("");
    let lang = data.strip_prefix("lang_").unwrap_or("");

    if !["en", "id"].contains(&lang) {
        return Ok(());
    }

    let chat_id = msg.chat().id;
    if !crate::utils::permissions::is_user_admin(&bot, chat_id, q.from.id).await {
        bot.answer_callback_query(q.id.clone())
            .text("❌ You need to be an admin.")
            .await?;
        return Ok(());
    }

    db::queries::set_language(&pool, chat_id.0, lang).await.log_err("lang_cb::set_language");

    let lang_name = match lang {
        "en" => "English 🇬🇧",
        "id" => "Indonesia 🇮🇩",
        _ => lang,
    };

    bot.answer_callback_query(q.id.clone())
        .text(format!("✅ Language: {}", lang_name))
        .await?;
    bot.edit_message_text(
        msg.chat().id,
        msg.id(),
        format!("✅ Language set to <b>{}</b>.", lang_name),
    )
    .parse_mode(teloxide::types::ParseMode::Html)
    .await?;
    Ok(())
}
