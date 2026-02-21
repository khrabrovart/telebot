use crate::{
    processor::{callback_query, message, poll_answer},
    TelegramBotClient,
};
use anyhow::Error;
use telebot_shared::{aws::DynamoDbClient, data::BotData};
use teloxide::types::{Update, UpdateKind};

pub async fn process(
    update: &Update,
    bot_data: &BotData,
    db: &DynamoDbClient,
) -> Result<(), Error> {
    let bot = TelegramBotClient::new(bot_data).await?;

    if let UpdateKind::Message(msg) = &update.kind {
        message::process(msg, update, &bot, bot_data).await?;
        return Ok(());
    }

    if let UpdateKind::PollAnswer(poll_answer) = &update.kind {
        poll_answer::process(poll_answer, &bot, db).await?;
        return Ok(());
    }

    if let UpdateKind::CallbackQuery(callback_query) = &update.kind {
        callback_query::process(callback_query, update, &bot, bot_data, db).await?;
        return Ok(());
    }

    Ok(())
}
