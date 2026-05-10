use teloxide::prelude::*;

/// Kick a user safely: ban then unban with exponential backoff.
/// Replaces the `ban -> sleep(1s) -> unban` pattern which is prone to race conditions.
pub async fn safe_kick(bot: &Bot, chat_id: ChatId, user_id: UserId) -> Result<(), teloxide::RequestError> {
    bot.ban_chat_member(chat_id, user_id).await?;

    for attempt in 0u32..3 {
        match bot.unban_chat_member(chat_id, user_id).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt == 2 {
                    return Err(e);
                }
                let delay = tokio::time::Duration::from_millis(200 * (1u64 << attempt));
                tokio::time::sleep(delay).await;
            }
        }
    }
    Ok(())
}
