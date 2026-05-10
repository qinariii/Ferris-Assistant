## Help
help-header = <b>📖 Ferris Bot Help</b>

    Select a module below to see available commands.
    Use buttons to navigate between modules.

help-admin = <b>👮 Admin Module</b>

    Make it easy to promote and demote users with the admin module!

    <b>User Commands:</b>
    - /adminlist: List the admins in the current chat.

    <b>Admin Commands:</b>
    - /promote <code>&lt;reply/username/userid&gt;</code>: Promote a user.
    - /demote <code>&lt;reply/username/userid&gt;</code>: Demote a user.
    - /title <code>&lt;reply&gt;</code> <code>&lt;custom title&gt;</code>: Set custom admin title (max 16 chars).

    <b>Examples:</b>
    <code>/promote @username</code>
    <code>/title New Title</code> (reply to user)

    <i>Note: Both you and the bot need promote/demote permissions.</i>

help-bans = <b>🚫 Bans Module</b>

    Sometimes users can be annoying and you might want to remove them from your chat!

    <b>User Commands:</b>
    - /kickme: Kicks the user who issued the command.

    <b>Admin Commands:</b>
    - /ban <code>&lt;user&gt;</code>: Ban a user. (via handle, or reply)
    - /sban <code>&lt;user&gt;</code>: Ban silently — no notification and deletes your command.
    - /dban <code>&lt;reply&gt;</code>: Ban a user and delete the replied message.
    - /tban <code>&lt;user&gt;</code> <code>&lt;time&gt;</code>: Temp ban for x time.
    - /kick <code>&lt;user&gt;</code>: Kick a user (they can rejoin).
    - /dkick <code>&lt;reply&gt;</code>: Delete message and kick user.
    - /unban <code>&lt;user&gt;</code>: Unban a user.

    <i>Reply to a message or provide a user ID/username.</i>

help-mutes = <b>🔇 Mutes Module</b>

    <b>Admin Commands:</b>
    - /mute <code>&lt;user&gt;</code>: Silence a user. Can also be used as a reply.
    - /tmute <code>&lt;user&gt;</code> <code>&lt;time&gt;</code>: Mute for x time.
    - /unmute <code>&lt;user&gt;</code>: Unmute a user.

    <i>Muted users cannot send any messages in the group.</i>

help-warns = <b>⚠️ Warns Module</b>

    Keep your members in check with warnings; stop them getting out of control!

    <b>Admin Commands:</b>
    - /warn <code>&lt;user&gt;</code> <code>[reason]</code>: Warn a user.
    - /warns <code>&lt;user&gt;</code>: See a user's warnings.
    - /resetwarns <code>&lt;user&gt;</code>: Reset all warnings to 0.
    - /setwarnlimit <code>&lt;number&gt;</code>: Set number of warnings before action.
    - /setwarnmode <code>&lt;ban/kick/mute&gt;</code>: Set the action on max warns.

help-notes = <b>📝 Notes Module</b>

    Save data for future users with notes!

    <b>User Commands:</b>
    - /get <code>&lt;notename&gt;</code>: Get a note.
    - <code>#notename</code>: Same as /get.
    - /notes: List all saved notes in this chat.

    <b>Admin Commands:</b>
    - /save <code>&lt;notename&gt;</code> <code>&lt;text&gt;</code>: Save a new note.
    - /clear <code>&lt;notename&gt;</code>: Delete the associated note.

help-filters = <b>🔍 Filters Module</b>

    Filters are case insensitive; every time someone says your trigger words, the bot will reply something else!

    <b>Commands:</b>
    - /filter <code>&lt;keyword&gt;</code> <code>&lt;reply&gt;</code>: Add an auto-reply filter.
    - /filters: List all chat filters.
    - /stop <code>&lt;keyword&gt;</code>: Stop the bot from replying to that trigger.
    - /stopall: Remove ALL filters.

help-welcome = <b>👋 Welcome/Goodbye Module</b>

    Welcome new members to your groups or say goodbye after they leave!

    <b>Admin Commands:</b>
    - /setwelcome <code>&lt;text&gt;</code>: Set welcome text for group.
    - /welcome <code>&lt;on/off&gt;</code>: Enable or disable welcome messages.
    - /setgoodbye <code>&lt;text&gt;</code>: Set goodbye text for group.
    - /goodbye <code>&lt;on/off&gt;</code>: Enable or disable goodbye messages.
    - /cleanservice <code>&lt;on/off&gt;</code>: Delete join/leave notifications.

help-rules = <b>📏 Rules Module</b>

    Every chat works with different rules; this module will help make those rules clearer!

    <b>Commands:</b>
    - /rules: Check the current chat rules.
    - /setrules <code>&lt;text&gt;</code>: Set the rules for this chat.
    - /clearrules: Clear the rules for this chat.

help-blacklist = <b>🚫 Blacklist Module</b>

    <b>Commands:</b>
    - /blacklist: View current blacklisted words.
    - /addblacklist <code>&lt;word&gt;</code>: Add a word to the blacklist.
    - /rmblacklist <code>&lt;word&gt;</code>: Remove a word from the blacklist.
    - /blacklistmode <code>&lt;delete/warn/mute/kick/ban&gt;</code>: Set action for blacklisted words.

help-purges = <b>🧹 Purges Module</b>

    <b>Admin Commands:</b>
    - /purge: Deletes all messages between this and the replied-to message.
    - /del: Deletes the message you replied to.

    <i>Bot needs "Delete Messages" permission.</i>

help-pins = <b>📌 Pins Module</b>

    <b>Admin Commands:</b>
    - /pin: Pin the message you replied to.
    - /unpin: Unpin the current pinned message.
    - /unpinall: Unpin all pinned messages.

    <i>Bot needs "Pin Messages" permission.</i>

help-antiflood = <b>🌊 Antiflood Module</b>

    Prevent flooding in your chat!

    <b>Admin Commands:</b>
    - /flood: Get the current antiflood settings.
    - /setflood <code>&lt;number/off&gt;</code>: Set messages limit.
    - /setfloodmode <code>&lt;ban/kick/mute&gt;</code>: Choose action on flooding user.

help-disable = <b>🔒 Disable Module</b>

    Disable commonly used commands in your group.

    <b>Admin Commands:</b>
    - /disable <code>&lt;command&gt;</code>: Stop users from using the command.
    - /enable <code>&lt;command&gt;</code>: Allow users to use the command again.
    - /disabled: List the disabled commands.

help-locks = <b>🔐 Locks Module</b>

    <b>Admin Commands:</b>
    - /lock <code>&lt;type&gt;</code>: Lock a chat permission.
    - /unlock <code>&lt;type&gt;</code>: Unlock a chat permission.
    - /locks: View current chat locks.
    - /locktypes: Check available lock types.

help-logchannel = <b>📋 Log Channel Module</b>

    <b>Admin Commands:</b>
    - /logchannel: Get log channel info.
    - /setlogchannel <code>&lt;channel_id&gt;</code>: Set the log channel.
    - /unsetlogchannel: Unset the log channel.

help-reports = <b>📢 Reports Module</b>

    Let your members help moderate!

    <b>Commands:</b>
    - /report <code>&lt;reason&gt;</code>: Reply to a message to report it to admins.
    - @admin: Same as /report.
    - /reports <code>&lt;on/off&gt;</code>: Toggle reporting.

help-gbans = <b>🌐 Global Bans Module</b>

    <b>Owner/Sudo Commands:</b>
    - /gban <code>&lt;user&gt;</code> <code>[reason]</code>: Globally ban a user.
    - /ungban <code>&lt;user&gt;</code>: Remove a global ban.
    - /gbanlist: List all global bans.

help-backups = <b>💾 Backups Module</b>

    <b>Owner Commands:</b>
    - /export: Export chat settings as JSON.
    - /import: Import settings from a backup file.

help-connections = <b>🔗 Connections Module</b>

    Connect to a chat's database and manage things remotely!

    <b>Commands:</b>
    - /connect <code>&lt;chatid&gt;</code>: Connect to the specified chat.
    - /disconnect: Disconnect from the current chat.
    - /connection: See info about the currently connected chat.

help-afk = <b>💤 AFK Module</b>

    <b>Commands:</b>
    - /afk <code>[reason]</code>: Mark yourself as AFK.
    - Sending <code>brb</code> also sets you AFK.
    - Sending any message removes your AFK status.

help-blstickers = <b>🎨 Sticker Blacklist Module</b>

    <b>Admin Commands:</b>
    - /blsticker: View blacklisted sticker sets.
    - /addblsticker <code>&lt;set_name&gt;</code>: Blacklist a sticker set.
    - /rmblsticker <code>&lt;set_name&gt;</code>: Remove sticker set from blacklist.
    - /blstickermode <code>&lt;off/del/warn/mute/kick/ban&gt;</code>: Set action.

help-chatperms = <b>🛡️ Chat Permissions Module</b>

    <b>Admin Commands:</b>
    - /permissions: View current chat permissions.
    - /setpermissions <code>key=on/off</code>: Set permissions.

help-users = <b>👥 Users Module</b>

    Automatic background user and chat tracker.

    <b>Team Commands:</b>
    - /stats: Display bot statistics.
    - /chatlist: List all active chats.

help-misc = <b>📊 Misc Module</b>

    <b>Commands:</b>
    - /id: Get the current group ID.
    - /info <code>&lt;user&gt;</code>: Get user info.
    - /setlang <code>&lt;en/id&gt;</code>: Set bot language.
    - /settings: Open settings panel.

help-captcha = <b>🔐 Captcha Module</b>

    Protect your group from bots with CAPTCHA verification!

    <b>Admin Commands:</b>
    - /captcha <code>&lt;on/off&gt;</code>: Enable or disable captcha.
    - /captchamode <code>&lt;math/text&gt;</code>: Set captcha type.
    - /captchatime <code>&lt;1-10&gt;</code>: Set timeout in minutes.
    - /captchaaction <code>&lt;kick/ban/mute&gt;</code>: Set failure action.

help-devs = <b>🛠 Devs Module</b>

    Bot management commands for owner and developers.

    <b>Commands:</b>
    - /addsudo, /remsudo: Manage sudo users.
    - /adddev, /remdev: Manage developers.
    - /teamusers: List team members.
    - /broadcast <code>&lt;text&gt;</code>: Broadcast to all chats.
    - /botstats: View detailed statistics.

help-feds = <b>🏛 Federation Module</b>

    Federated group management — ban users across all your groups!

    <b>Commands:</b>
    - /newfed <code>&lt;name&gt;</code>: Create a federation.
    - /delfed: Delete your federation.
    - /joinfed <code>&lt;fed_id&gt;</code>: Join a federation.
    - /leavefed: Leave current federation.
    - /fban <code>&lt;user&gt;</code>: Federation ban.
    - /unfban <code>&lt;user&gt;</code>: Remove federation ban.
    - /fbanlist: List federation bans.

help-sed = <b>✏️ Sed/Regex Module</b>

    <b>Usage:</b>
    Reply to a message with:
    <code>s/pattern/replacement/flags</code>

    <b>Flags:</b> <b>i</b> — Case insensitive, <b>g</b> — Replace all

help-userinfo = <b>📋 Bios &amp; Abouts Module</b>

    <b>Commands:</b>
    - /setbio <code>&lt;text&gt;</code>: Set another user's bio (reply).
    - /bio <code>[user]</code>: Get a user's bio.
    - /setme <code>&lt;text&gt;</code>: Set your own info.
    - /me <code>[user]</code>: Get user info.

help-cleaner = <b>🧹 Cleaner Module</b>

    <b>Admin Commands:</b>
    - /cleanservice <code>&lt;on/off&gt;</code>: Auto-delete service messages.
    - /cleanbluetext <code>&lt;on/off&gt;</code>: Auto-delete unrecognized bot commands.

help-reactions = <b>⚡ Reactions Module</b>

    <b>Admin Commands:</b>
    - /addreaction <code>&lt;keyword&gt;</code> <code>&lt;emoji&gt;</code>: React when keyword is mentioned.
    - /removereaction <code>&lt;keyword&gt;</code>: Remove a reaction.
    - /reactions: List all reactions.
    - /resetreactions: Clear all reactions.

help-about = <b>ℹ️ About Ferris Bot</b>

    🦀 Built with Rust using Teloxide v0.17.0
    📦 PostgreSQL database with sqlx
    ⚡ Redis caching &amp; async performance

    <b>Features:</b>
    • Full group management (30+ modules)
    • Multi-language support (EN/ID)
    • Federation system
    • Modular architecture

    Made with ❤️ by Arumi

## Formatting
help-formatting = <b>📝 Formatting</b>

    Ferris Bot supports a large number of formatting options to make your messages more expressive. Take a look by clicking the buttons below!

help-formatting-markdown = <b>Markdown Formatting</b>

    You can format your message using <b>bold</b>, <i>italics</i>, <u>underline</u>, and much more. Go ahead and experiment!

    <b>Supported markdown:</b>
    - <code>`code words`</code>: Backticks for monospace. Shows as: <code>code words</code>.
    - <code>_italic words_</code>: Underscores for italic. Shows as: <i>italic words</i>.
    - <code>*bold words*</code>: Asterisks for bold. Shows as: <b>bold words</b>.
    - <code>~strikethrough~</code>: Tildes for strikethrough. Shows as: <s>strikethrough</s>.
    - <code>||spoiler||</code>: Double vertical bars for spoilers. Shows as: <tg-spoiler>Spoiler</tg-spoiler>.
    - <code>```pre```</code>: Triple backticks for preformatted text.
    - <code>__underline__</code>: Double underscores for underline.
    - <code>[hyperlink](example.com)</code>: Creates a hyperlink. Shows as: <a href="https://example.com/">hyperlink</a>.
    - <code>[My Button](buttonurl://example.com)</code>: Creates a button named "My Button" that opens <code>example.com</code>.

    To place buttons on the same row, use <code>:same</code>:
    <code>[button 1](buttonurl://example.com)</code>
    <code>[button 2](buttonurl://example.com:same)</code>

help-formatting-fillings = <b>Fillings</b>

    You can customize the contents of your message with contextual data. For example, mention a user by name in the welcome message, or mention them in a filter!

    <b>Supported fillings:</b>
    - <code>{first_name}</code>: The user's first name.
    - <code>{last_name}</code>: The user's last name.
    - <code>{full_name}</code>: The user's full name.
    - <code>{username}</code>: The user's username. If none, mentions the user instead.
    - <code>{mention}</code>: Mentions the user with their first name.
    - <code>{id}</code>: The user's ID.
    - <code>{chat_name}</code>: The chat's name.

## Example usage per module
help-example-admin = <b>💡 Example Usage — Admin</b>

    <code>/promote @username</code>
    <code>/demote @username</code>
    <code>/title New Title</code> (reply to user)
    <code>/adminlist</code>

help-example-bans = <b>💡 Example Usage — Bans</b>

    <code>/ban @spammer Spamming links</code>
    <code>/tban @user 2h Cooldown</code>
    <code>/dban</code> (reply to offending message)
    <code>/kick @user</code>
    <code>/unban @user</code>

help-example-mutes = <b>💡 Example Usage — Mutes</b>

    <code>/mute @user</code>
    <code>/tmute @user 30m</code>
    <code>/unmute @user</code>

help-example-warns = <b>💡 Example Usage — Warns</b>

    <code>/warn @user For disobeying the rules</code>
    <code>/setwarnlimit 5</code>
    <code>/setwarnmode mute</code>
    <code>/warns @user</code>
    <code>/resetwarns @user</code>

help-example-notes = <b>💡 Example Usage — Notes</b>

    <code>/save rules Please follow the group rules!</code>
    <code>/get rules</code>
    <code>#rules</code>
    <code>/notes</code>
    <code>/clear rules</code>

help-example-filters = <b>💡 Example Usage — Filters</b>

    <code>/filter hello Hello there! How are you?</code>
    <code>/filter "hello friend" Hello back!</code>
    <code>/stop hello</code>
    <code>/filters</code>

    <i>To save a file/image/gif, simply reply to the file with:</i>
    <code>/filter trigger</code>

help-example-welcome = <b>💡 Example Usage — Welcome</b>

    <code>/setwelcome Hey {first_name}, welcome to {chat_name}! Read the /rules first.</code>
    <code>/welcome on</code>
    <code>/setgoodbye Bye {first_name}, we'll miss you!</code>
    <code>/goodbye on</code>
    <code>/cleanservice on</code>

help-example-rules = <b>💡 Example Usage — Rules</b>

    <code>/setrules 1. No spam
    2. Be respectful
    3. English only</code>
    <code>/rules</code>
    <code>/clearrules</code>

help-example-blacklist = <b>💡 Example Usage — Blacklist</b>

    <code>/addblacklist spam</code>
    <code>/blacklistmode kick</code>
    <code>/blacklist</code>
    <code>/rmblacklist spam</code>

help-example-purges = <b>💡 Example Usage — Purges</b>

    → Reply to a message, then send <code>/purge</code>
    → <code>/del</code> (reply to the offending message)

help-example-pins = <b>💡 Example Usage — Pins</b>

    <code>/pin</code> (reply to a message)
    <code>/pin loud</code> (pin with notification)
    <code>/unpin</code>
    <code>/unpinall</code>

help-example-antiflood = <b>💡 Example Usage — Antiflood</b>

    <code>/setflood 10</code>
    <code>/setfloodmode mute</code>
    <code>/setflood off</code>
    <code>/flood</code>

help-example-disable = <b>💡 Example Usage — Disable</b>

    <code>/disable rules</code>
    <code>/enable rules</code>
    <code>/disabled</code>

    <i>Disabled commands are only disabled for non-admins.</i>

help-example-locks = <b>💡 Example Usage — Locks</b>

    <code>/lock media</code> — lock all media
    <code>/lock url</code> — auto-delete URLs
    <code>/unlock all</code> — remove all locks
    <code>/locks</code>
    <code>/locktypes</code>

help-example-logchannel = <b>💡 Example Usage — Log Channel</b>

    1. Add the bot to your channel (as admin)
    2. <code>/setlogchannel -1001234567890</code>
    3. <code>/logchannel</code>
    4. <code>/unsetlogchannel</code>

help-example-reports = <b>💡 Example Usage — Reports</b>

    → Reply to a spam message: <code>/report Spam</code>
    → Or tag <code>@admin</code> in the group
    <code>/reports on</code>

help-example-gbans = <b>💡 Example Usage — Global Bans</b>

    <code>/gban @spammer Spam across multiple groups</code>
    <code>/ungban 123456789</code>
    <code>/gbanlist</code>

help-example-backups = <b>💡 Example Usage — Backups</b>

    <code>/export</code> — export chat settings as JSON
    <code>/import</code> — reply to a backup file to import

help-example-connections = <b>💡 Example Usage — Connections</b>

    <code>/connect -1001234567890</code>
    <code>/disconnect</code>
    <code>/connection</code>

help-example-afk = <b>💡 Example Usage — AFK</b>

    <code>/afk Gone for lunch</code>
    <code>/afk</code>
    Just type <code>brb</code> to go AFK.

help-example-blstickers = <b>💡 Example Usage — Sticker Blacklist</b>

    <code>/addblsticker</code> (reply to a sticker)
    <code>/blstickermode ban</code>
    <code>/blsticker</code>
    <code>/rmblsticker set_name</code>

help-example-chatperms = <b>💡 Example Usage — Chat Permissions</b>

    <code>/setpermissions stickers=off polls=off</code>
    <code>/setpermissions media=on</code>
    <code>/permissions</code>

help-example-users = <b>💡 Example Usage — Users</b>

    <code>/stats</code>
    <code>/chatlist</code>

help-example-misc = <b>💡 Example Usage — Misc</b>

    <code>/info @username</code>
    <code>/id</code> (reply to a message)
    <code>/setlang en</code>
    <code>/settings</code>

help-example-captcha = <b>💡 Example Usage — Captcha</b>

    <code>/captcha on</code>
    <code>/captchamode math</code>
    <code>/captchatime 5</code>
    <code>/captchaaction kick</code>
    <code>/captchaattempts 3</code>

help-example-devs = <b>💡 Example Usage — Devs</b>

    <code>/addsudo @trusted_user</code>
    <code>/broadcast Hello everyone!</code>
    <code>/leavechat -1001234567890</code>
    <code>/botstats</code>
    <code>/teamusers</code>

help-example-feds = <b>💡 Example Usage — Federations</b>

    <code>/newfed My Network</code>
    <code>/joinfed fed_id_here</code>
    <code>/fban @spammer Spam across groups</code>
    <code>/unfban @user</code>
    <code>/fbanlist</code>
    <code>/fedrules Be respectful in all federated chats</code>

help-example-sed = <b>💡 Example Usage — Sed/Regex</b>

    Reply to a message with:
    <code>s/hello/bye/gi</code>
    <code>s|typo|fixed|</code>
    <code>s/teh/the/g</code>

help-example-userinfo = <b>💡 Example Usage — Bios &amp; Abouts</b>

    <code>/setme Rust enthusiast and bot developer</code>
    <code>/setbio This person is cool!</code> (reply to user)
    <code>/bio @username</code>
    <code>/me</code>

help-example-cleaner = <b>💡 Example Usage — Cleaner</b>

    <code>/cleanservice on</code>
    <code>/cleanbluetext on</code>

help-example-reactions = <b>💡 Example Usage — Reactions</b>

    <code>/addreaction hello 👋</code>
    <code>/addreaction rust 🦀</code>
    <code>/removereaction hello</code>
    <code>/reactions</code>
    <code>/resetreactions</code>
