use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::keyboards::inline;

const HELP_ADMIN: &str = "\
<b>👮 Admin Module</b>\n\n\
<b>Commands:</b>\n\
/promote &lt;user&gt; - Promote a user to admin\n\
/demote &lt;user&gt; - Demote an admin\n\
/adminlist - List all admins in the group\n\
/title &lt;title&gt; - Set admin title (reply to user)\n\n\
<i>Note: Both you and the bot need appropriate permissions.</i>";

const HELP_BANS: &str = "\
<b>🚫 Bans Module</b>\n\n\
<b>Commands:</b>\n\
/ban &lt;user&gt; [reason] - Ban a user\n\
/unban &lt;user&gt; - Unban a user\n\
/kick &lt;user&gt; - Kick a user (they can rejoin)\n\
/tban &lt;user&gt; &lt;time&gt; - Temporarily ban (e.g. 1h, 30m, 1d)\n\
/dkick &lt;reply&gt; - Delete message and kick user\n\
/dban &lt;reply&gt; - Delete message and ban user\n\n\
<i>Reply to a message or provide a user ID/username.</i>";

const HELP_MUTES: &str = "\
<b>🔇 Mutes Module</b>\n\n\
<b>Commands:</b>\n\
/mute &lt;user&gt; [reason] - Mute a user\n\
/unmute &lt;user&gt; - Unmute a user\n\
/tmute &lt;user&gt; &lt;time&gt; - Temporarily mute (e.g. 1h, 30m)\n\n\
<i>Muted users cannot send messages in the group.</i>";

const HELP_WARNS: &str = "\
<b>⚠️ Warns Module</b>\n\n\
<b>Commands:</b>\n\
/warn &lt;user&gt; [reason] - Warn a user\n\
/warns &lt;user&gt; - Check user's warnings\n\
/resetwarns &lt;user&gt; - Reset all warns for a user\n\
/setwarnlimit &lt;number&gt; - Set max warns before action\n\
/setwarnmode &lt;ban/kick/mute&gt; - Set action on max warns\n\n\
<i>Users exceeding the warn limit will be actioned.</i>";

const HELP_NOTES: &str = "\
<b>📝 Notes Module</b>\n\n\
<b>Commands:</b>\n\
/save &lt;name&gt; &lt;content&gt; - Save a note\n\
/get &lt;name&gt; - Retrieve a note\n\
/notes - List all notes\n\
/clear &lt;name&gt; - Delete a note\n\
#notename - Quick get note\n\n\
<i>Notes can contain text with formatting.</i>";

const HELP_FILTERS: &str = "\
<b>🔍 Filters Module</b>\n\n\
<b>Commands:</b>\n\
/filter &lt;keyword&gt; &lt;reply&gt; - Add a filter\n\
/filters - List all filters\n\
/stop &lt;keyword&gt; - Remove a filter\n\
/stopall - Remove all filters\n\n\
<i>Bot will auto-reply when keyword is detected.</i>";

const HELP_WELCOME: &str = "\
<b>👋 Welcome Module</b>\n\n\
<b>Commands:</b>\n\
/welcome &lt;on/off&gt; - Toggle welcome messages\n\
/setwelcome &lt;text&gt; - Set welcome message\n\
/goodbye &lt;on/off&gt; - Toggle goodbye messages\n\
/setgoodbye &lt;text&gt; - Set goodbye message\n\n\
<b>Placeholders:</b>\n\
{first_name}, {last_name}, {full_name}, {username}, {mention}, {id}, {chat_name}";

const HELP_RULES: &str = "\
<b>📏 Rules Module</b>\n\n\
<b>Commands:</b>\n\
/rules - View group rules\n\
/setrules &lt;text&gt; - Set group rules\n\
/clearrules - Clear all rules\n\n\
<i>Set rules to inform members about group guidelines.</i>";

const HELP_BLACKLIST: &str = "\
<b>🚫 Blacklist Module</b>\n\n\
<b>Commands:</b>\n\
/blacklist - View blacklisted words\n\
/addblacklist &lt;word&gt; - Add word to blacklist\n\
/rmblacklist &lt;word&gt; - Remove word from blacklist\n\
/blacklistmode &lt;delete/warn/mute/kick/ban&gt; - Set action\n\n\
<i>Messages containing blacklisted words will be actioned.</i>";

const HELP_PURGES: &str = "\
<b>🧹 Purges Module</b>\n\n\
<b>Commands:</b>\n\
/purge - Delete messages from replied message to current\n\
/del - Delete the replied message\n\n\
<i>Bot needs delete message permission.</i>";

const HELP_PINS: &str = "\
<b>📌 Pins Module</b>\n\n\
<b>Commands:</b>\n\
/pin - Pin the replied message\n\
/unpin - Unpin the current pinned message\n\
/unpinall - Unpin all pinned messages\n\n\
<i>Bot needs pin message permission.</i>";

const HELP_ANTIFLOOD: &str = "\
<b>🌊 Antiflood Module</b>\n\n\
<b>Commands:</b>\n\
/setflood &lt;number/off&gt; - Set flood limit\n\
/flood - View current flood settings\n\
/setfloodmode &lt;ban/kick/mute&gt; - Set flood action\n\n\
<i>Users sending too many messages will be actioned.</i>";

const HELP_DISABLE: &str = "\
<b>🔒 Disable Module</b>\n\n\
<b>Commands:</b>\n\
/disable &lt;command&gt; - Disable a command\n\
/enable &lt;command&gt; - Enable a command\n\
/disabled - List disabled commands\n\n\
<i>Disabled commands won't work until re-enabled.</i>";

const HELP_LOCKS: &str = "\
<b>🔐 Locks Module</b>\n\n\
<b>Commands:</b>\n\
/lock &lt;type&gt; - Lock a message type\n\
/unlock &lt;type&gt; - Unlock a message type\n\
/locks - View current locks\n\
/locktypes - List available lock types\n\n\
<b>Lock types:</b> sticker, audio, voice, document, video, photo, gif, url, forward, game, location, media, all\n\n\
<i>Locked message types will be auto-deleted.</i>";

const HELP_LOGCHANNEL: &str = "\
<b>📋 Log Channel Module</b>\n\n\
<b>Commands:</b>\n\
/logchannel - View current log channel\n\
/setlogchannel &lt;channel_id&gt; - Set log channel\n\
/unsetlogchannel - Remove log channel\n\n\
<i>Admin actions will be logged to the specified channel.</i>";

const HELP_REPORTS: &str = "\
<b>📢 Reports Module</b>\n\n\
<b>Commands:</b>\n\
/report - Report a message (reply)\n\
/reports &lt;on/off&gt; - Toggle reporting\n\
@admin - Tag to report a message\n\n\
<i>Reports notify all admins about problematic messages.</i>";

const HELP_GBANS: &str = "\
<b>🌐 Global Bans Module</b>\n\n\
<b>Commands:</b>\n\
/gban &lt;user&gt; [reason] - Globally ban a user\n\
/ungban &lt;user&gt; - Remove global ban\n\
/gbanlist - List all global bans\n\n\
<i>Only bot owner/sudo can use these commands.\nGbanned users are auto-banned in all groups.</i>";

const HELP_BACKUPS: &str = "\
<b>💾 Backups Module</b>\n\n\
<b>Commands:</b>\n\
/export - Export chat settings as JSON\n\
/import - Import settings from a backup file (reply)\n\n\
<i>Only group owners can export/import settings.\nBackups include: notes, filters, blacklist, rules, settings, locks.</i>";

const HELP_CONNECTIONS: &str = "\
<b>🔗 Connections Module</b>\n\n\
<b>Commands:</b>\n\
/connect - Connect to current group\n\
/connect &lt;chat_id&gt; - Connect to a group (from PM)\n\
/disconnect - Disconnect from group\n\
/connection - Check connection status\n\n\
<i>Connect to manage a group from bot's PM.</i>";

const HELP_AFK: &str = "\
<b>💤 AFK Module</b>\n\n\
<b>Commands:</b>\n\
/afk [reason] - Set yourself as AFK\n\n\
<b>Auto-triggers:</b>\n\
• Sending \"brb\" sets you AFK\n\
• Sending any message removes AFK\n\
• Replying/mentioning an AFK user shows their status\n\n\
<i>Let others know you're away from keyboard.</i>";

const HELP_BLSTICKERS: &str = "\
<b>🎨 Sticker Blacklist Module</b>\n\n\
<b>Commands:</b>\n\
/blsticker - View blacklisted sticker sets\n\
/addblsticker &lt;set_name&gt; - Add sticker set (or reply to sticker)\n\
/rmblsticker &lt;set_name&gt; - Remove sticker set\n\
/blstickermode &lt;off/del/warn/mute/kick/ban&gt; - Set action\n\n\
<i>Block specific sticker packs from being used.</i>";

const HELP_CHATPERMS: &str = "\
<b>🛡️ Chat Permissions Module</b>\n\n\
<b>Commands:</b>\n\
/permissions - View current chat permissions\n\
/setpermissions key=on/off - Set permissions\n\n\
<b>Keys:</b> messages, media, photos, videos, audios, documents, stickers, polls, preview, info, invite, pin, topics\n\n\
<i>Example: /setpermissions stickers=off polls=off</i>";

const HELP_USERS: &str = "\
<b>👥 Users Module</b>\n\n\
<b>Commands:</b>\n\
/stats - View bot statistics (sudo only)\n\
/chatlist - List known chats (sudo only)\n\n\
<i>Users and chats are automatically tracked.\nOnly bot owner/sudo can view this data.</i>";

const HELP_MISC: &str = "\
<b>📊 Misc</b>\n\n\
<b>Commands:</b>\n\
/id - Get chat/user ID\n\
/info &lt;user&gt; - Get user info\n\
/setlang &lt;en/id&gt; - Set bot language\n\
/settings - Open settings panel\n\n\
<i>General utility commands.</i>";

const HELP_CAPTCHA: &str = "\
<b>🔐 Captcha Module</b>\n\n\
<b>Commands:</b>\n\
/captcha &lt;on/off&gt; - Toggle captcha verification\n\
/captchamode &lt;math/text&gt; - Set captcha type\n\
/captchatime &lt;1-10&gt; - Set timeout in minutes\n\
/captchaaction &lt;kick/ban/mute&gt; - Set failure action\n\n\
<i>New members must solve a captcha to chat.\nBot needs restrict members permission.</i>";

const HELP_DEVS: &str = "\
<b>🛠 Devs Module</b>\n\n\
<b>Owner Only:</b>\n\
/addsudo &lt;user&gt; - Add sudo user\n\
/remsudo &lt;user&gt; - Remove sudo user\n\
/adddev &lt;user&gt; - Add dev user\n\
/remdev &lt;user&gt; - Remove dev user\n\
/broadcast &lt;text&gt; - Broadcast to all chats\n\n\
<b>Dev/Owner:</b>\n\
/teamusers - List team members\n\
/chatinfo [chat_id] - Get chat info\n\
/leavechat &lt;chat_id&gt; - Leave a chat\n\
/botstats - View bot statistics\n\n\
<i>Bot management commands for the dev team.</i>";

const HELP_SED: &str = "\
<b>✏️ Sed/Regex Module</b>\n\n\
<b>Usage:</b>\n\
Reply to a message with:\n\
<code>s/pattern/replacement/flags</code>\n\n\
<b>Flags:</b>\n\
• <b>i</b> - Case insensitive\n\
• <b>g</b> - Replace all occurrences\n\n\
<b>Delimiters:</b> / : | _\n\n\
<i>Example: s/hello/bye/gi</i>";

const HELP_USERINFO: &str = "\
<b>📋 Bios &amp; Abouts Module</b>\n\n\
<b>Commands:</b>\n\
/setme &lt;text&gt; - Set your own info\n\
/me [user] - View user info\n\
/setbio &lt;text&gt; - Set another user's bio (reply)\n\
/bio [user] - View user bio\n\n\
<i>Bio can only be set by others. Info is set by yourself.</i>";

const HELP_CLEANER: &str = "\
<b>🧹 Cleaner Module</b>\n\n\
<b>Commands:</b>\n\
/cleanservice &lt;on/off&gt; - Auto-delete service messages (join, leave, pin, etc.)\n\
/cleanbluetext &lt;on/off&gt; - Auto-delete unrecognized bot commands\n\n\
<i>Keeps your chat clean from clutter.</i>";

const HELP_REACTIONS: &str = "\
<b>⚡ Reactions Module</b>\n\n\
<b>Commands:</b>\n\
/addreaction &lt;keyword&gt; &lt;emoji&gt; - React when keyword is mentioned\n\
/removereaction &lt;keyword&gt; - Remove a reaction\n\
/reactions - List all reactions\n\
/resetreactions - Clear all reactions\n\n\
<i>Bot reacts with emoji when keyword is found in messages.</i>";

const HELP_FEDS: &str = "\
<b>🏛 Federation Module</b>\n\n\
<b>Commands:</b>\n\
/newfed &lt;name&gt; - Create a federation\n\
/delfed &lt;fed_id&gt; - Delete your federation\n\
/joinfed &lt;fed_id&gt; - Join a federation\n\
/leavefed - Leave current federation\n\
/fedinfo [fed_id] - Federation info\n\
/fedchat - Check chat's federation\n\
/fedpromote &lt;user&gt; - Promote fed admin\n\
/feddemote &lt;user&gt; - Demote fed admin\n\
/fban &lt;user&gt; [reason] - Federation ban\n\
/unfban &lt;user&gt; - Remove federation ban\n\
/fbanlist [fed_id] - List federation bans\n\
/fedrules [text] - View/set fed rules\n\n\
<i>Federated group management across chats.</i>";

pub async fn help_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    let text = "<b>📖 Ferris Bot Help</b>\n\n\
        Select a module below to see available commands.\n\
        Use buttons to navigate between modules.";

    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(inline::help_main_keyboard())
        .await?;
    Ok(())
}

pub async fn help_main_callback(bot: Bot, q: CallbackQuery) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let text = "<b>📖 Ferris Bot Help</b>\n\n\
        Select a module below to see available commands.\n\
        Use buttons to navigate between modules.";

    bot.edit_message_text(msg.chat().id, msg.id(), text)
        .parse_mode(ParseMode::Html)
        .reply_markup(inline::help_main_keyboard())
        .await?;
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}

pub async fn help_module_callback(bot: Bot, q: CallbackQuery) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let data = q.data.as_deref().unwrap_or("");
    let text = match data {
        "help_admin" => HELP_ADMIN,
        "help_bans" => HELP_BANS,
        "help_mutes" => HELP_MUTES,
        "help_warns" => HELP_WARNS,
        "help_notes" => HELP_NOTES,
        "help_filters" => HELP_FILTERS,
        "help_welcome" => HELP_WELCOME,
        "help_rules" => HELP_RULES,
        "help_blacklist" => HELP_BLACKLIST,
        "help_purges" => HELP_PURGES,
        "help_pins" => HELP_PINS,
        "help_antiflood" => HELP_ANTIFLOOD,
        "help_disable" => HELP_DISABLE,
        "help_locks" => HELP_LOCKS,
        "help_logchannel" => HELP_LOGCHANNEL,
        "help_reports" => HELP_REPORTS,
        "help_gbans" => HELP_GBANS,
        "help_backups" => HELP_BACKUPS,
        "help_connections" => HELP_CONNECTIONS,
        "help_afk" => HELP_AFK,
        "help_blstickers" => HELP_BLSTICKERS,
        "help_chatperms" => HELP_CHATPERMS,
        "help_users" => HELP_USERS,
        "help_misc" => HELP_MISC,
        "help_captcha" => HELP_CAPTCHA,
        "help_devs" => HELP_DEVS,
        "help_feds" => HELP_FEDS,
        "help_sed" => HELP_SED,
        "help_userinfo" => HELP_USERINFO,
        "help_cleaner" => HELP_CLEANER,
        "help_reactions" => HELP_REACTIONS,
        _ => "Unknown module",
    };

    bot.edit_message_text(msg.chat().id, msg.id(), text)
        .parse_mode(ParseMode::Html)
        .reply_markup(inline::back_to_help_keyboard())
        .await?;
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}

pub async fn about_callback(bot: Bot, q: CallbackQuery) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let text = "<b>ℹ️ About Ferris Bot</b>\n\n\
        🦀 Built with Rust using Teloxide v0.17.0\n\
        📦 SQLite database with sqlx\n\
        ⚡ Async & high performance\n\n\
        <b>Features:</b>\n\
        • Full group management\n\
        • Inline keyboard navigation\n\
        • Multi-language support\n\
        • Modular architecture\n\n\
        Made with ❤️ by Arumi";

    bot.edit_message_text(msg.chat().id, msg.id(), text)
        .parse_mode(ParseMode::Html)
        .reply_markup(inline::back_to_help_keyboard())
        .await?;
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}

pub async fn close_callback(bot: Bot, q: CallbackQuery) -> ResponseResult<()> {
    if let Some(msg) = q.message {
        bot.delete_message(msg.chat().id, msg.id()).await.ok();
    }
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}
