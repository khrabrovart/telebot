use anyhow::Error;
use telebot_shared::data::BotData;
use teloxide::{
    prelude::*,
    types::{CallbackQueryId, InlineKeyboardMarkup, MessageId, ParseMode, Recipient},
};

pub struct TelegramBotClient {
    pub bot_id: String,
    bot: Bot,
}

impl TelegramBotClient {
    pub async fn new(bot_data: &BotData) -> Result<Self, Error> {
        Ok(Self {
            bot_id: bot_data.id.clone(),
            bot: Bot::new(bot_data.token.clone()),
        })
    }

    pub async fn send_text(&self, chat_id: Recipient, text: &str) -> Result<(), Error> {
        self.bot
            .send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }

    pub async fn send_text_with_markup(
        &self,
        chat_id: Recipient,
        text: &str,
        markup: &InlineKeyboardMarkup,
    ) -> Result<(), Error> {
        self.bot
            .send_message(chat_id, text)
            .reply_markup(markup.clone())
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }

    pub async fn edit_message_text_with_markup(
        &self,
        chat_id: Recipient,
        message_id: MessageId,
        text: &str,
        markup: &InlineKeyboardMarkup,
    ) -> Result<(), Error> {
        self.bot
            .edit_message_text(chat_id, message_id, text)
            .reply_markup(markup.clone())
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }

    pub async fn get_chat_title(&self, chat_id: Recipient) -> Result<String, Error> {
        let chat = self.bot.get_chat(chat_id).await?;
        Ok(chat
            .title()
            .unwrap_or_else(|| chat.username().unwrap_or("Unknown"))
            .into())
    }

    pub async fn answer_callback_query(
        &self,
        callback_query_id: CallbackQueryId,
    ) -> Result<(), Error> {
        self.bot.answer_callback_query(callback_query_id).await?;

        Ok(())
    }
}
