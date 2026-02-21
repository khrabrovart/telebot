use anyhow::Error;
use telebot_shared::data::BotData;
use teloxide::types::{Recipient, Update};

use crate::TelegramBotClient;

pub async fn validate_access(
    update: &Update,
    chat_id: Recipient,
    bot_data: &BotData,
    bot: &TelegramBotClient,
) -> Result<bool, Error> {
    let sender_id = update
        .from()
        .map(|u| u.username.as_ref().unwrap().clone())
        .unwrap();

    let admins = &bot_data.admins;

    if !admins.contains(&sender_id) {
        bot.send_text(
            chat_id,
            "У вас недостаточно прав для выполнения этого действия",
        )
        .await?;

        return Ok(false);
    }

    Ok(true)
}
