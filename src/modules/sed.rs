use regex::RegexBuilder;
use teloxide::prelude::*;
use teloxide::types::ReplyParameters;

const DELIMITERS: &[char] = &['/', ':', '|', '_'];

struct SedResult {
    pattern: String,
    replacement: String,
    flags: String,
}

fn parse_sed(text: &str) -> Option<SedResult> {
    if text.len() < 3 || !text.starts_with('s') {
        return None;
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 3 || !DELIMITERS.contains(&chars[1]) {
        return None;
    }

    let delim = chars[1];

    // Count delimiters (need at least 2)
    let delim_count = chars[2..].iter().filter(|&&c| c == delim).count();
    if delim_count < 1 {
        return None;
    }

    // Parse pattern
    let mut i = 2;
    let mut pattern = String::new();
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            if chars[i + 1] == delim {
                // Escaped delimiter: strip backslash, keep delimiter char
                pattern.push(delim);
            } else {
                // Other escape (e.g. \d, \w, \s): preserve both chars so
                // the regex engine can interpret them correctly.
                pattern.push('\\');
                pattern.push(chars[i + 1]);
            }
            i += 2;
            continue;
        }
        if chars[i] == delim {
            i += 1;
            break;
        }
        pattern.push(chars[i]);
        i += 1;
    }

    if pattern.is_empty() {
        return None;
    }

    // Parse replacement (only need to handle escaped delimiter here)
    let mut replacement = String::new();
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            if chars[i + 1] == delim {
                // Escaped delimiter in replacement
                replacement.push(delim);
                i += 2;
                continue;
            }
            // Other backslash sequences in replacement pass through as-is
            replacement.push('\\');
            replacement.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if chars[i] == delim {
            i += 1;
            break;
        }
        replacement.push(chars[i]);
        i += 1;
    }

    // Parse flags
    let flags: String = chars[i..].iter().collect::<String>().to_lowercase();

    Some(SedResult {
        pattern,
        replacement,
        flags,
    })
}

/// Check if a message is a sed command and apply it to the replied message
pub async fn check_sed(bot: Bot, msg: Message) -> ResponseResult<()> {
    let text = match msg.text() {
        Some(t) => t,
        None => return Ok(()),
    };

    // Must start with s followed by a delimiter
    if text.len() < 4 || !text.starts_with('s') {
        return Ok(());
    }

    let chars: Vec<char> = text.chars().collect();
    if !DELIMITERS.contains(&chars[1]) {
        return Ok(());
    }

    let reply = match msg.reply_to_message() {
        Some(r) => r,
        None => return Ok(()),
    };

    let original_text = reply
        .text()
        .or_else(|| reply.caption())
        .unwrap_or("");

    if original_text.is_empty() {
        return Ok(());
    }

    let sed = match parse_sed(text) {
        Some(s) => s,
        None => return Ok(()),
    };

    // Limit pattern length to prevent abuse
    if sed.pattern.len() > 200 {
        bot.send_message(msg.chat.id, "❌ Pattern too long (max 200 chars).")
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;
        return Ok(());
    }

    let case_insensitive = sed.flags.contains('i');
    let global = sed.flags.contains('g');

    let re = match RegexBuilder::new(&sed.pattern)
        .case_insensitive(case_insensitive)
        .size_limit(1 << 16) // 64 KB compiled size limit to prevent ReDoS
        .build()
    {
        Ok(r) => r,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid regex pattern.")
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
            return Ok(());
        }
    };

    // Safety: check if regex would replace entire message
    if let Some(m) = re.find(original_text) {
        if m.start() == 0 && m.end() == original_text.len() && sed.replacement.is_empty() {
            return Ok(());
        }
    }

    let result = if global {
        re.replace_all(original_text, sed.replacement.as_str())
    } else {
        re.replace(original_text, sed.replacement.as_str())
    };

    let result = result.trim().to_string();

    if result.is_empty() || result == original_text {
        return Ok(());
    }

    if result.len() > 4096 {
        bot.send_message(msg.chat.id, "❌ Result is too long!")
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;
        return Ok(());
    }

    bot.send_message(msg.chat.id, result)
        .reply_parameters(ReplyParameters::new(reply.id))
        .await?;

    Ok(())
}
