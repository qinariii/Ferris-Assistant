use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn start_keyboard(bot_username: &str) -> InlineKeyboardMarkup {
    let add_url = format!("https://t.me/{}?startgroup=true", bot_username);
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📖 Help & Commands", "help_main"),
            InlineKeyboardButton::callback("ℹ️ About", "about"),
        ],
        vec![
            InlineKeyboardButton::callback("⚙️ Settings", "settings"),
            InlineKeyboardButton::url(
                "➕ Add to Group",
                add_url.parse().unwrap(),
            ),
        ],
    ])
}

pub fn help_main_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("👮 Admin", "help_admin"),
            InlineKeyboardButton::callback("🚫 Bans", "help_bans"),
            InlineKeyboardButton::callback("🔇 Mutes", "help_mutes"),
        ],
        vec![
            InlineKeyboardButton::callback("⚠️ Warns", "help_warns"),
            InlineKeyboardButton::callback("📝 Notes", "help_notes"),
            InlineKeyboardButton::callback("🔍 Filters", "help_filters"),
        ],
        vec![
            InlineKeyboardButton::callback("👋 Welcome", "help_welcome"),
            InlineKeyboardButton::callback("📏 Rules", "help_rules"),
            InlineKeyboardButton::callback("🚫 Blacklist", "help_blacklist"),
        ],
        vec![
            InlineKeyboardButton::callback("🧹 Purges", "help_purges"),
            InlineKeyboardButton::callback("📌 Pins", "help_pins"),
            InlineKeyboardButton::callback("🌊 Antiflood", "help_antiflood"),
        ],
        vec![
            InlineKeyboardButton::callback("🔒 Disable", "help_disable"),
            InlineKeyboardButton::callback("🔒 Locks", "help_locks"),
            InlineKeyboardButton::callback("📋 Logs", "help_logchannel"),
        ],
        vec![
            InlineKeyboardButton::callback("📢 Reports", "help_reports"),
            InlineKeyboardButton::callback("🌐 Gbans", "help_gbans"),
            InlineKeyboardButton::callback("💾 Backups", "help_backups"),
        ],
        vec![
            InlineKeyboardButton::callback("🔗 Connect", "help_connections"),
            InlineKeyboardButton::callback("💤 AFK", "help_afk"),
            InlineKeyboardButton::callback("🎨 Stickers", "help_blstickers"),
        ],
        vec![
            InlineKeyboardButton::callback("🛡️ Perms", "help_chatperms"),
            InlineKeyboardButton::callback("👥 Users", "help_users"),
            InlineKeyboardButton::callback("📊 Misc", "help_misc"),
        ],
        vec![
            InlineKeyboardButton::callback("🔐 Captcha", "help_captcha"),
            InlineKeyboardButton::callback("🛠 Devs", "help_devs"),
            InlineKeyboardButton::callback("🏛 Feds", "help_feds"),
        ],
        vec![
            InlineKeyboardButton::callback("✏️ Sed", "help_sed"),
            InlineKeyboardButton::callback("📋 Bio", "help_userinfo"),
            InlineKeyboardButton::callback("🧹 Cleaner", "help_cleaner"),
            InlineKeyboardButton::callback("⚡ React", "help_reactions"),
        ],
        vec![InlineKeyboardButton::callback("« Back", "start_back")],
    ])
}

pub fn back_to_help_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "« Back to Help",
        "help_main",
    )]])
}

pub fn admin_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Promote", "admin_promote"),
            InlineKeyboardButton::callback("Demote", "admin_demote"),
        ],
        vec![InlineKeyboardButton::callback("« Back to Help", "help_main")],
    ])
}

pub fn ban_confirm_keyboard(user_id: u64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("✅ Yes, Ban", format!("ban_confirm_{}", user_id)),
        InlineKeyboardButton::callback("❌ Cancel", "ban_cancel"),
    ]])
}

pub fn unban_keyboard(user_id: u64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🔓 Unban",
        format!("unban_{}", user_id),
    )]])
}

pub fn mute_keyboard(user_id: u64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🔊 Unmute",
        format!("unmute_{}", user_id),
    )]])
}

pub fn warn_keyboard(user_id: u64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Remove Warn", format!("rmwarn_{}", user_id)),
    ]])
}

pub fn warn_mode_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Ban", "warnmode_ban"),
            InlineKeyboardButton::callback("Kick", "warnmode_kick"),
            InlineKeyboardButton::callback("Mute", "warnmode_mute"),
        ],
        vec![InlineKeyboardButton::callback("« Back", "settings")],
    ])
}

pub fn welcome_settings_keyboard(enabled: bool) -> InlineKeyboardMarkup {
    let toggle_text = if enabled {
        "✅ Welcome ON - Tap to disable"
    } else {
        "❌ Welcome OFF - Tap to enable"
    };
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(toggle_text, "welcome_toggle")],
        vec![
            InlineKeyboardButton::callback("Set Welcome Text", "welcome_set"),
            InlineKeyboardButton::callback("Preview", "welcome_preview"),
        ],
        vec![InlineKeyboardButton::callback("« Back", "settings")],
    ])
}

pub fn goodbye_settings_keyboard(enabled: bool) -> InlineKeyboardMarkup {
    let toggle_text = if enabled {
        "✅ Goodbye ON - Tap to disable"
    } else {
        "❌ Goodbye OFF - Tap to enable"
    };
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(toggle_text, "goodbye_toggle")],
        vec![
            InlineKeyboardButton::callback("Set Goodbye Text", "goodbye_set"),
            InlineKeyboardButton::callback("Preview", "goodbye_preview"),
        ],
        vec![InlineKeyboardButton::callback("« Back", "settings")],
    ])
}

pub fn rules_keyboard(chat_id: i64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "📏 Rules",
        format!("rules_{}", chat_id),
    )]])
}

pub fn settings_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("👋 Welcome", "set_welcome"),
            InlineKeyboardButton::callback("👋 Goodbye", "set_goodbye"),
        ],
        vec![
            InlineKeyboardButton::callback("⚠️ Warn Mode", "set_warnmode"),
            InlineKeyboardButton::callback("🌊 Antiflood", "set_antiflood"),
        ],
        vec![
            InlineKeyboardButton::callback("🚫 Blacklist", "set_blacklist"),
            InlineKeyboardButton::callback("🌐 Language", "set_language"),
        ],
        vec![InlineKeyboardButton::callback("« Back", "start_back")],
    ])
}

pub fn antiflood_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("5 messages", "af_set_5"),
            InlineKeyboardButton::callback("10 messages", "af_set_10"),
            InlineKeyboardButton::callback("15 messages", "af_set_15"),
        ],
        vec![
            InlineKeyboardButton::callback("Off", "af_set_0"),
            InlineKeyboardButton::callback("Mode: Ban", "af_mode_ban"),
            InlineKeyboardButton::callback("Mode: Mute", "af_mode_mute"),
        ],
        vec![InlineKeyboardButton::callback("« Back", "settings")],
    ])
}

pub fn language_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🇬🇧 English", "lang_en"),
            InlineKeyboardButton::callback("🇮🇩 Indonesia", "lang_id"),
        ],
        vec![InlineKeyboardButton::callback("« Back", "settings")],
    ])
}

pub fn close_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "❌ Close",
        "close",
    )]])
}

pub fn purge_confirm_keyboard(from_id: i64, to_id: i64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("✅ Yes", format!("purge_{}_{}", from_id, to_id)),
        InlineKeyboardButton::callback("❌ No", "close"),
    ]])
}
