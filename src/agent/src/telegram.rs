use telebot_shared::data::BotData;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardMarkup, MessageId, ParseMode, Recipient},
};

pub struct TelegramBotClient {
    pub bot_id: String,
    bot: Bot,
}

impl TelegramBotClient {
    pub async fn new(bot_data: &BotData) -> Result<Self, anyhow::Error> {
        Ok(Self {
            bot_id: bot_data.id.clone(),
            bot: Bot::new(bot_data.token.clone()),
        })
    }

    pub async fn send_text(&self, chat_id: Recipient, text: &str) -> Result<(), anyhow::Error> {
        self.bot
            .send_message(chat_id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        Ok(())
    }

    pub async fn send_text_with_markup(
        &self,
        chat_id: Recipient,
        text: &str,
        markup: &InlineKeyboardMarkup,
    ) -> Result<(), anyhow::Error> {
        self.bot
            .send_message(chat_id, text)
            .reply_markup(markup.clone())
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        Ok(())
    }

    pub async fn edit_message_text_with_markup(
        &self,
        chat_id: Recipient,
        message_id: MessageId,
        text: &str,
        markup: &InlineKeyboardMarkup,
    ) -> Result<(), anyhow::Error> {
        self.bot
            .edit_message_text(chat_id, message_id, text)
            .reply_markup(markup.clone())
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        Ok(())
    }
}
