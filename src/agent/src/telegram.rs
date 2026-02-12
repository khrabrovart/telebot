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
        let text = Self::escape_chars(text);
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
    ) -> Result<(), anyhow::Error> {
        let text = Self::escape_chars(text);
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
    ) -> Result<(), anyhow::Error> {
        let text = Self::escape_chars(text);
        self.bot
            .edit_message_text(chat_id, message_id, text)
            .reply_markup(markup.clone())
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }

    fn escape_chars(text: &str) -> String {
        let mut result = text.to_string();
        result = result
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("&", "&amp;");
        result
    }
}
