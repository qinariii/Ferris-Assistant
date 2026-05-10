use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::db;
use crate::keyboards::inline;
use crate::utils::i18n;

fn help_text(lang: &str, key: &str) -> String {
    i18n::t(lang, key, None)
}

// Help texts below are legacy fallbacks — translations are loaded from locales/{lang}/help.ftl via i18n

const _HELP_BANS: &str = "\
<b>🚫 Bans Module</b>\n\n\
Sometimes users can be annoying and you might want to remove them from your chat!\n\n\
<b>User Commands:</b>\n\
- /kickme: Kicks the user who issued the command.\n\n\
<b>Admin Commands:</b>\n\
- /ban <code>&lt;user&gt;</code>: Ban a user. (via handle, or reply)\n\
- /sban <code>&lt;user&gt;</code>: Ban silently — no notification and deletes your command.\n\
- /dban <code>&lt;reply&gt;</code>: Ban a user and delete the replied message.\n\
- /tban <code>&lt;user&gt;</code> <code>&lt;time&gt;</code>: Temp ban for x time. <code>m</code> = minutes, <code>h</code> = hours, <code>d</code> = days.\n\
- /kick <code>&lt;user&gt;</code>: Kick a user (they can rejoin).\n\
- /dkick <code>&lt;reply&gt;</code>: Delete message and kick user.\n\
- /unban <code>&lt;user&gt;</code>: Unban a user.\n\n\
<b>Examples:</b>\n\
<code>/ban @spammer Spamming links</code>\n\
<code>/tban @user 2h Cooldown</code>\n\
<code>/dban</code> (reply to offending message)\n\n\
<i>Reply to a message or provide a user ID/username.</i>";

const _HELP_MUTES: &str = "\
<b>🔇 Mutes Module</b>\n\n\
<b>Admin Commands:</b>\n\
- /mute <code>&lt;user&gt;</code>: Silence a user. Can also be used as a reply.\n\
- /tmute <code>&lt;user&gt;</code> <code>&lt;time&gt;</code>: Mute for x time. <code>m</code> = minutes, <code>h</code> = hours, <code>d</code> = days.\n\
- /unmute <code>&lt;user&gt;</code>: Unmute a user.\n\n\
<b>Examples:</b>\n\
<code>/mute @user</code>\n\
<code>/tmute @user 30m</code>\n\n\
<i>Muted users cannot send any messages in the group.</i>";

const _HELP_WARNS: &str = "\
<b>⚠️ Warns Module</b>\n\n\
Keep your members in check with warnings; stop them getting out of control!\n\n\
<b>Admin Commands:</b>\n\
- /warn <code>&lt;user&gt;</code> <code>[reason]</code>: Warn a user.\n\
- /dwarn <code>&lt;reason&gt;</code>: Warn by reply and delete their message.\n\
- /swarn <code>&lt;reason&gt;</code>: Silently warn, delete your message.\n\
- /warns <code>&lt;user&gt;</code>: See a user's warnings.\n\
- /rmwarn: Remove a user's latest warning.\n\
- /resetwarns <code>&lt;user&gt;</code>: Reset all warnings to 0.\n\
- /setwarnlimit <code>&lt;number&gt;</code>: Set number of warnings before action.\n\
- /setwarnmode <code>&lt;ban/kick/mute&gt;</code>: Set the action on max warns.\n\n\
<b>Examples:</b>\n\
<code>/warn @user For disobeying the rules</code>\n\
<code>/setwarnlimit 5</code>\n\
<code>/setwarnmode mute</code>\n\n\
<i>If you're looking for automated warnings, check the Blacklist module!</i>";

const _HELP_NOTES: &str = "\
<b>📝 Notes Module</b>\n\n\
Save data for future users with notes! A phone number, a nice gif, a funny picture — anything!\n\n\
<b>User Commands:</b>\n\
- /get <code>&lt;notename&gt;</code>: Get a note.\n\
- <code>#notename</code>: Same as /get.\n\
- /notes: List all saved notes in this chat.\n\
- /saved: Same as /notes.\n\n\
<b>Admin Commands:</b>\n\
- /save <code>&lt;notename&gt;</code> <code>&lt;text&gt;</code>: Save a new note. Replying to a message will save that message.\n\
- /clear <code>&lt;notename&gt;</code>: Delete the associated note.\n\
- /clearall: Delete ALL notes in a chat. Cannot be undone.\n\n\
<b>Examples:</b>\n\
<code>/save rules Please follow the group rules!</code>\n\
<code>/get rules</code>\n\
<code>#rules</code>";

const _HELP_FILTERS: &str = "\
<b>🔍 Filters Module</b>\n\n\
Filters are case insensitive; every time someone says your trigger words, the bot will reply something else!\n\n\
<b>Commands:</b>\n\
- /filter <code>&lt;keyword&gt;</code> <code>&lt;reply&gt;</code>: Add an auto-reply filter. Quote for multi-word triggers.\n\
- /filters: List all chat filters.\n\
- /stop <code>&lt;keyword&gt;</code>: Stop the bot from replying to that trigger.\n\
- /stopall: Remove ALL filters. Cannot be undone.\n\n\
<b>Examples:</b>\n\
<code>/filter hello Hello there! How are you?</code>\n\
<code>/filter \"hello friend\" Hello back! Long time no see!</code>\n\
<code>/stop hello</code>\n\n\
<i>To save a file, image, gif, or any other attachment, simply reply to the file with:</i>\n\
<code>/filter trigger</code>";

const _HELP_WELCOME: &str = "\
<b>👋 Welcome/Goodbye Module</b>\n\n\
Welcome new members to your groups or say goodbye after they leave!\n\n\
<b>Admin Commands:</b>\n\
- /setwelcome <code>&lt;text&gt;</code>: Set welcome text for group.\n\
- /welcome <code>&lt;on/off&gt;</code>: Enable or disable welcome messages.\n\
- /resetwelcome: Reset the welcome message to default.\n\
- /setgoodbye <code>&lt;text&gt;</code>: Set goodbye text for group.\n\
- /goodbye <code>&lt;on/off&gt;</code>: Enable or disable goodbye messages.\n\
- /resetgoodbye: Reset the goodbye message to default.\n\
- /cleanservice <code>&lt;on/off&gt;</code>: Delete 'x joined the group' notifications.\n\n\
<b>Placeholders:</b>\n\
<code>{first_name}</code>, <code>{last_name}</code>, <code>{full_name}</code>, <code>{username}</code>, <code>{mention}</code>, <code>{id}</code>, <code>{chat_name}</code>\n\n\
<b>Example:</b>\n\
<code>/setwelcome Hey {first_name}, welcome to {chat_name}! Read the /rules first.</code>";

const _HELP_RULES: &str = "\
<b>📏 Rules Module</b>\n\n\
Every chat works with different rules; this module will help make those rules clearer!\n\n\
<b>User Commands:</b>\n\
- /rules: Check the current chat rules.\n\n\
<b>Admin Commands:</b>\n\
- /setrules <code>&lt;text&gt;</code>: Set the rules for this chat.\n\
- /clearrules: Clear the rules for this chat.\n\n\
<b>Example:</b>\n\
<code>/setrules 1. No spam\n2. Be respectful\n3. English only</code>";

const _HELP_BLACKLIST: &str = "\
<b>🚫 Blacklist Module</b>\n\n\
<b>User Commands:</b>\n\
- /blacklist: View current blacklisted words.\n\n\
<b>Admin Commands:</b>\n\
- /addblacklist <code>&lt;word&gt;</code>: Add a word to the blacklist.\n\
- /rmblacklist <code>&lt;word&gt;</code>: Remove a word from the blacklist.\n\
- /blacklistmode <code>&lt;delete/warn/mute/kick/ban&gt;</code>: Set action for blacklisted words.\n\n\
<b>Examples:</b>\n\
<code>/addblacklist spam</code>\n\
<code>/blacklistmode kick</code>\n\n\
<i>Messages containing blacklisted words will trigger the configured action.</i>";

const _HELP_PURGES: &str = "\
<b>🧹 Purges Module</b>\n\n\
<b>Admin Commands:</b>\n\
- /purge: Deletes all messages between this and the replied-to message.\n\
- /purge <code>&lt;number&gt;</code>: Deletes the replied message and X messages following it.\n\
- /del: Deletes the message you replied to.\n\n\
<b>Examples:</b>\n\
→ Reply to a message, then send <code>/purge</code>\n\
→ <code>/del</code> (reply to the offending message)\n\n\
<i>Bot needs \"Delete Messages\" permission.</i>";

const _HELP_PINS: &str = "\
<b>📌 Pins Module</b>\n\n\
Keep your chat up to date on the latest news with pinned messages!\n\n\
<b>User Commands:</b>\n\
- /pinned: Get the current pinned message.\n\n\
<b>Admin Commands:</b>\n\
- /pin: Pin the message you replied to. Add <code>loud</code> or <code>notify</code> to notify members.\n\
- /unpin: Unpin the current pinned message.\n\
- /unpinall: Unpin all pinned messages.\n\n\
<b>Examples:</b>\n\
<code>/pin</code> (reply to a message)\n\
<code>/pin loud</code> (pin with notification)\n\n\
<i>Bot needs \"Pin Messages\" permission.</i>";

const _HELP_ANTIFLOOD: &str = "\
<b>🌊 Antiflood Module</b>\n\n\
You know how sometimes, people join, send 100 messages, and ruin your chat? With antiflood, that happens no more!\n\n\
<b>Admin Commands:</b>\n\
- /flood: Get the current antiflood settings.\n\
- /setflood <code>&lt;number/off&gt;</code>: Set messages limit after which to take action. Set to <code>0</code>, <code>off</code>, or <code>no</code> to disable.\n\
- /setfloodmode <code>&lt;ban/kick/mute&gt;</code>: Choose which action to take on a flooding user.\n\n\
<b>Examples:</b>\n\
<code>/setflood 10</code>\n\
<code>/setfloodmode mute</code>\n\
<code>/setflood off</code>\n\n\
<i>The flood limit should be set between 3 and 100.</i>";

const _HELP_DISABLE: &str = "\
<b>🔒 Disable Module</b>\n\n\
This module allows you to disable commonly used commands, so no one can use them.\n\n\
<b>Admin Commands:</b>\n\
- /disable <code>&lt;command&gt;</code>: Stop users from using the command in this group.\n\
- /enable <code>&lt;command&gt;</code>: Allow users to use the command again.\n\
- /disabled: List the disabled commands in this chat.\n\
- /disableable: List all disableable commands.\n\n\
<b>Examples:</b>\n\
<code>/disable rules</code>\n\
<code>/enable rules</code>\n\n\
<i>Note: Disabled commands are only disabled for non-admins. All admins can still use them.</i>";

const _HELP_LOCKS: &str = "\
<b>🔐 Locks Module</b>\n\n\
<b>Admin Commands:</b>\n\
- /lock <code>&lt;type&gt;</code>: Lock a chat permission.\n\
- /unlock <code>&lt;type&gt;</code>: Unlock a chat permission.\n\
- /locks: View current chat locks.\n\
- /locktypes: Check available lock types.\n\n\
<b>Lock types:</b> sticker, audio, voice, document, video, photo, gif, url, forward, game, location, media, all\n\n\
<b>Examples:</b>\n\
<code>/lock media</code> — locks all media messages in the chat.\n\
<code>/lock url</code> — auto-deletes all messages with URLs.\n\
<code>/unlock all</code> — remove all locks.\n\n\
<i>Locking bots will stop non-admins from adding bots to the chat.</i>";

const _HELP_LOGCHANNEL: &str = "\
<b>📋 Log Channel Module</b>\n\n\
<b>Admin Commands:</b>\n\
- /logchannel: Get log channel info.\n\
- /setlogchannel <code>&lt;channel_id&gt;</code>: Set the log channel.\n\
- /unsetlogchannel: Unset the log channel.\n\n\
<b>Setup:</b>\n\
1. Add the bot to the desired channel (as an admin)\n\
2. Send <code>/setlogchannel &lt;channel_id&gt;</code> in the group\n\n\
<i>Admin actions will be logged to the specified channel.</i>";

const _HELP_REPORTS: &str = "\
<b>📢 Reports Module</b>\n\n\
Don't have time to monitor your group 24/7? Let your members help!\n\n\
<b>User Commands:</b>\n\
- /report <code>&lt;reason&gt;</code>: Reply to a message to report it to admins.\n\
- @admin: Same as /report.\n\n\
<b>Admin Commands:</b>\n\
- /reports <code>&lt;on/off&gt;</code>: Change report setting, or view current status.\n\n\
<b>Example:</b>\n\
→ Reply to a spam message: <code>/report Spam</code>\n\n\
<i>NOTE: Neither /report nor @admin will get triggered if used by admins. You MUST reply to a message to report a user.</i>";

const _HELP_GBANS: &str = "\
<b>🌐 Global Bans Module</b>\n\n\
<b>Owner/Sudo Commands:</b>\n\
- /gban <code>&lt;user&gt;</code> <code>[reason]</code>: Globally ban a user across all groups.\n\
- /ungban <code>&lt;user&gt;</code>: Remove a global ban.\n\
- /gbanlist: List all global bans.\n\n\
<b>Examples:</b>\n\
<code>/gban @spammer Spam across multiple groups</code>\n\
<code>/ungban 123456789</code>\n\n\
<i>Only bot owner/sudo can use these commands. Gbanned users are auto-banned when they join any group.</i>";

const _HELP_BACKUPS: &str = "\
<b>💾 Backups Module</b>\n\n\
<b>Owner Commands:</b>\n\
- /export: Export chat settings as JSON.\n\
- /import: Import settings from a backup file (reply to file).\n\n\
<b>What's included:</b>\n\
Notes, filters, blacklist, rules, warns, locks, and all chat settings.\n\n\
<i>Only group owners can export/import settings.</i>";

const _HELP_CONNECTIONS: &str = "\
<b>🔗 Connections Module</b>\n\n\
This module allows you to connect to a chat's database and manage things without the chat knowing about it!\n\n\
<b>User Commands:</b>\n\
- /connect <code>&lt;chatid&gt;</code>: Connect to the specified chat.\n\
- /disconnect: Disconnect from the current chat.\n\
- /connection: See info about the currently connected chat.\n\n\
<b>Admin Commands:</b>\n\
- /allowconnect <code>&lt;yes/no&gt;</code>: Allow users to connect to this chat or not.\n\n\
<b>Example:</b>\n\
<code>/connect -1001234567890</code>\n\n\
<i>You can retrieve the chat id using /id in your chat. Don't be surprised if the id is negative; all supergroups have negative ids.</i>";

const _HELP_AFK: &str = "\
<b>💤 AFK Module</b>\n\n\
<b>Commands:</b>\n\
- /afk <code>[reason]</code>: Mark yourself as AFK (Away From Keyboard).\n\n\
<b>Auto-triggers:</b>\n\
- Sending <code>brb</code> also sets you AFK.\n\
- Sending any message removes your AFK status.\n\
- Replying to or mentioning an AFK user shows their AFK status and reason.\n\n\
<b>Examples:</b>\n\
<code>/afk Gone for lunch</code>\n\
<code>/afk</code>\n\n\
<i>Let others know you're away!</i>";

const _HELP_BLSTICKERS: &str = "\
<b>🎨 Sticker Blacklist Module</b>\n\n\
<b>Admin Commands:</b>\n\
- /blsticker: View blacklisted sticker sets.\n\
- /addblsticker <code>&lt;set_name&gt;</code>: Blacklist a sticker set (or reply to a sticker).\n\
- /rmblsticker <code>&lt;set_name&gt;</code>: Remove sticker set from blacklist.\n\
- /blstickermode <code>&lt;off/del/warn/mute/kick/ban&gt;</code>: Set action for blacklisted stickers.\n\n\
<b>Examples:</b>\n\
<code>/addblsticker</code> (reply to a sticker)\n\
<code>/blstickermode ban</code>\n\n\
<i>Block specific sticker packs from being used in your chat.</i>";

const _HELP_CHATPERMS: &str = "\
<b>🛡️ Chat Permissions Module</b>\n\n\
<b>Admin Commands:</b>\n\
- /permissions: View current chat permissions.\n\
- /setpermissions <code>key=on/off</code>: Set permissions.\n\n\
<b>Available Keys:</b>\n\
messages, media, photos, videos, audios, documents, stickers, polls, preview, info, invite, pin, topics\n\n\
<b>Examples:</b>\n\
<code>/setpermissions stickers=off polls=off</code>\n\
<code>/setpermissions media=on</code>\n\n\
<i>Control what group members can send.</i>";

const _HELP_USERS: &str = "\
<b>👥 Users Module</b>\n\n\
Automatic background user and chat tracker. This module silently records every message sender and chat the bot is active in.\n\n\
<b>Team Commands:</b>\n\
- /stats: Display bot statistics and system info.\n\
- /chatlist: Generate and send a list of all active chats.\n\n\
<i>Only bot owner/sudo can view this data. Users and chats are automatically tracked in the background.</i>";

const _HELP_MISC: &str = "\
<b>📊 Misc Module</b>\n\n\
<b>Commands:</b>\n\
- /id: Get the current group ID. If used by replying, get that user's ID.\n\
- /info <code>&lt;user&gt;</code>: Get user info. Can be used as a reply or by passing a username/ID.\n\
- /ping: Ping the Telegram server!\n\
- /setlang <code>&lt;en/id&gt;</code>: Set bot language for this chat.\n\
- /settings: Open settings panel.\n\n\
<b>Examples:</b>\n\
<code>/info @username</code>\n\
<code>/id</code> (reply to a message)\n\
<code>/setlang en</code>";

const _HELP_CAPTCHA: &str = "\
<b>🔐 Captcha Module</b>\n\n\
Protect your group from bots and spammers with CAPTCHA verification!\n\n\
<b>Captcha Types:</b>\n\
- <b>Math</b>: Solve simple arithmetic problems.\n\
- <b>Text</b>: Identify text shown in an image.\n\n\
<b>Admin Commands:</b>\n\
- /captcha <code>&lt;on/off&gt;</code>: Enable or disable captcha verification.\n\
- /captchamode <code>&lt;math/text&gt;</code>: Set captcha type.\n\
- /captchatime <code>&lt;1-10&gt;</code>: Set timeout in minutes (default: 2).\n\
- /captchaaction <code>&lt;kick/ban/mute&gt;</code>: Set action for failed verification (default: kick).\n\
- /captchaattempts <code>&lt;1-10&gt;</code>: Set maximum verification attempts (default: 3).\n\n\
<b>Examples:</b>\n\
<code>/captcha on</code>\n\
<code>/captchamode math</code>\n\
<code>/captchatime 5</code>\n\n\
<i>When enabled, new members are auto-muted until they complete the captcha. If they fail or timeout, the configured action is taken.</i>";

const _HELP_DEVS: &str = "\
<b>🛠 Devs Module</b>\n\n\
Bot management and diagnostic commands restricted to the bot owner and trusted developers.\n\n\
<b>Team Commands:</b>\n\
- /stats: Display bot statistics and system info.\n\
- /teamusers: List all team members.\n\n\
<b>Owner Commands:</b>\n\
- /addsudo <code>&lt;user&gt;</code>: Grant sudo permissions to a user.\n\
- /adddev <code>&lt;user&gt;</code>: Grant developer permissions to a user.\n\
- /remsudo <code>&lt;user&gt;</code>: Revoke sudo permissions.\n\
- /remdev <code>&lt;user&gt;</code>: Revoke developer permissions.\n\
- /broadcast <code>&lt;text&gt;</code>: Broadcast a message to all chats.\n\n\
<b>Diagnostic Commands:</b>\n\
- /chatinfo <code>[chat_id]</code>: Display detailed chat information.\n\
- /chatlist: Generate and send a list of all active chats.\n\
- /leavechat <code>&lt;chat_id&gt;</code>: Force the bot to leave a specified chat.\n\
- /botstats: View detailed bot statistics.\n\n\
<b>Examples:</b>\n\
<code>/addsudo @trusted_user</code>\n\
<code>/broadcast Hello everyone!</code>\n\
<code>/leavechat -1001234567890</code>";

const _HELP_SED: &str = "\
<b>✏️ Sed/Regex Module</b>\n\n\
<b>Usage:</b>\n\
Reply to a message with:\n\
<code>s/pattern/replacement/flags</code>\n\n\
<b>Flags:</b>\n\
- <b>i</b> — Case insensitive\n\
- <b>g</b> — Replace all occurrences\n\n\
<b>Delimiters:</b> <code>/</code> <code>:</code> <code>|</code> <code>_</code>\n\n\
<b>Examples:</b>\n\
<code>s/hello/bye/gi</code>\n\
<code>s|typo|fixed|</code>\n\
<code>s/teh/the/g</code>\n\n\
<i>The resulting message cannot be larger than 4096 characters. Special characters (<code>+*.?\\</code>) need to be escaped.</i>";

const _HELP_USERINFO: &str = "\
<b>📋 Bios &amp; Abouts Module</b>\n\n\
<b>Commands:</b>\n\
- /setbio <code>&lt;text&gt;</code>: Set another user's bio (while replying).\n\
- /bio <code>[user]</code>: Get your or another user's bio. Cannot be set by yourself.\n\
- /setme <code>&lt;text&gt;</code>: Set your own info.\n\
- /me <code>[user]</code>: Get your or another user's info.\n\n\
<b>Examples:</b>\n\
<code>/setme Rust enthusiast and bot developer</code>\n\
<code>/setbio This person is cool!</code> (reply to user)\n\
<code>/bio @username</code>";

const _HELP_CLEANER: &str = "\
<b>🧹 Cleaner Module</b>\n\n\
<b>Admin Commands:</b>\n\
- /cleanservice <code>&lt;on/off&gt;</code>: Auto-delete service messages (join, leave, pin notifications, etc.)\n\
- /cleanbluetext <code>&lt;on/off&gt;</code>: Auto-delete unrecognized bot commands (blue text).\n\n\
<b>Examples:</b>\n\
<code>/cleanservice on</code>\n\
<code>/cleanbluetext on</code>\n\n\
<i>Keeps your chat clean from clutter and service messages.</i>";

const _HELP_REACTIONS: &str = "\
<b>⚡ Reactions Module</b>\n\n\
<b>Admin Commands:</b>\n\
- /addreaction <code>&lt;keyword&gt;</code> <code>&lt;emoji&gt;</code>: React when keyword is mentioned.\n\
- /removereaction <code>&lt;keyword&gt;</code>: Remove a reaction.\n\
- /reactions: List all reactions.\n\
- /resetreactions: Clear all reactions.\n\n\
<b>Examples:</b>\n\
<code>/addreaction hello 👋</code>\n\
<code>/addreaction rust 🦀</code>\n\
<code>/removereaction hello</code>\n\n\
<i>Bot reacts with the configured emoji when the keyword is found in a message.</i>";

const _HELP_FEDS: &str = "\
<b>🏛 Federation Module</b>\n\n\
Federated group management — ban users across all your groups with one command!\n\n\
<b>General Commands:</b>\n\
- /newfed <code>&lt;name&gt;</code>: Create a federation.\n\
- /delfed <code>&lt;fed_id&gt;</code>: Delete your federation.\n\
- /fedinfo <code>[fed_id]</code>: Federation info.\n\
- /fedchat: Check this chat's federation.\n\n\
<b>Admin Commands:</b>\n\
- /joinfed <code>&lt;fed_id&gt;</code>: Join a federation.\n\
- /leavefed: Leave current federation.\n\
- /fedpromote <code>&lt;user&gt;</code>: Promote a fed admin.\n\
- /feddemote <code>&lt;user&gt;</code>: Demote a fed admin.\n\
- /fban <code>&lt;user&gt;</code> <code>[reason]</code>: Federation ban.\n\
- /unfban <code>&lt;user&gt;</code>: Remove federation ban.\n\
- /fbanlist <code>[fed_id]</code>: List federation bans.\n\
- /fedrules <code>[text]</code>: View/set fed rules.\n\n\
<b>Examples:</b>\n\
<code>/newfed My Network</code>\n\
<code>/fban @spammer Spam across groups</code>\n\
<code>/fedrules Be respectful in all federated chats</code>";

pub async fn help_command(bot: Bot, msg: Message, pool: db::Pool) -> ResponseResult<()> {
    let lang = i18n::get_chat_lang(&pool, msg.chat.id.0).await;
    let text = help_text(&lang, "help-header");

    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(inline::help_main_keyboard())
        .await?;
    Ok(())
}

pub async fn help_main_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, msg.chat().id.0).await;
    let text = help_text(&lang, "help-header");

    bot.edit_message_text(msg.chat().id, msg.id(), text)
        .parse_mode(ParseMode::Html)
        .reply_markup(inline::help_main_keyboard())
        .await?;
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}

pub async fn help_module_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, msg.chat().id.0).await;
    let data = q.data.as_deref().unwrap_or("");
    let key = match data {
        "help_admin" => "help-admin",
        "help_bans" => "help-bans",
        "help_mutes" => "help-mutes",
        "help_warns" => "help-warns",
        "help_notes" => "help-notes",
        "help_filters" => "help-filters",
        "help_welcome" => "help-welcome",
        "help_rules" => "help-rules",
        "help_blacklist" => "help-blacklist",
        "help_purges" => "help-purges",
        "help_pins" => "help-pins",
        "help_antiflood" => "help-antiflood",
        "help_disable" => "help-disable",
        "help_locks" => "help-locks",
        "help_logchannel" => "help-logchannel",
        "help_reports" => "help-reports",
        "help_gbans" => "help-gbans",
        "help_backups" => "help-backups",
        "help_connections" => "help-connections",
        "help_afk" => "help-afk",
        "help_blstickers" => "help-blstickers",
        "help_chatperms" => "help-chatperms",
        "help_users" => "help-users",
        "help_misc" => "help-misc",
        "help_captcha" => "help-captcha",
        "help_devs" => "help-devs",
        "help_feds" => "help-feds",
        "help_sed" => "help-sed",
        "help_userinfo" => "help-userinfo",
        "help_cleaner" => "help-cleaner",
        "help_reactions" => "help-reactions",
        _ => "help-header",
    };
    let text = help_text(&lang, key);

    let module_suffix = data.strip_prefix("help_").unwrap_or("");
    let keyboard = if module_suffix == "notes" {
        inline::help_notes_keyboard()
    } else {
        inline::help_module_keyboard(module_suffix)
    };
    bot.edit_message_text(msg.chat().id, msg.id(), text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}

pub async fn example_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let data = q.data.as_deref().unwrap_or("");
    let module = data.strip_prefix("example_").unwrap_or("");
    let lang = i18n::get_chat_lang(&pool, msg.chat().id.0).await;
    let key = format!("help-example-{}", module);
    let text = i18n::t(&lang, &key, None);

    bot.edit_message_text(msg.chat().id, msg.id(), text)
        .parse_mode(ParseMode::Html)
        .reply_markup(inline::example_back_keyboard(module))
        .await?;
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}

pub async fn formatting_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let data = q.data.as_deref().unwrap_or("");
    let module = data.strip_prefix("fmt_").unwrap_or("");
    let lang = i18n::get_chat_lang(&pool, msg.chat().id.0).await;
    let text = i18n::t(&lang, "help-formatting", None);

    bot.edit_message_text(msg.chat().id, msg.id(), text)
        .parse_mode(ParseMode::Html)
        .reply_markup(inline::formatting_main_keyboard(module))
        .await?;
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}

pub async fn formatting_sub_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let data = q.data.as_deref().unwrap_or("");
    // data is like "fmtsub_markdown_notes" or "fmtsub_fillings_bans"
    let rest = data.strip_prefix("fmtsub_").unwrap_or("");
    let (sub, module) = match rest.split_once('_') {
        Some((s, m)) => (s, m),
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, msg.chat().id.0).await;
    let key = format!("help-formatting-{}", sub);
    let text = i18n::t(&lang, &key, None);

    bot.edit_message_text(msg.chat().id, msg.id(), text)
        .parse_mode(ParseMode::Html)
        .reply_markup(inline::formatting_sub_keyboard(module))
        .await?;
    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}

pub async fn about_callback(bot: Bot, q: CallbackQuery, pool: db::Pool) -> ResponseResult<()> {
    let msg = match q.message {
        Some(ref m) => m.clone(),
        None => return Ok(()),
    };

    let lang = i18n::get_chat_lang(&pool, msg.chat().id.0).await;
    let text = help_text(&lang, "help-about");

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
