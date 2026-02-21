use anyhow::Error;
use telebot_shared::data::BotData;
use teloxide::{
    dispatching::dialogue::GetChatId,
    types::{Message, Recipient, Update},
};

use crate::{
    processor::{access_validator, menus},
    TelegramBotClient,
};

pub async fn process(
    message: &Message,
    update: &Update,
    bot: &TelegramBotClient,
    bot_data: &BotData,
) -> Result<(), Error> {
    let chat_id: Recipient = match update.chat_id().unwrap().as_user() {
        Some(user) => user.into(),
        None => {
            return Ok(());
        }
    };

    match access_validator::validate_access(update, chat_id.clone(), bot_data, bot).await? {
        true => (),
        false => return Ok(()),
    }

    if let Some("/start") = message.text() {
        bot.send_text_with_markup(chat_id.clone(), "🏠 Главное меню", &menus::main_menu())
            .await?;
    }

    Ok(())
}
