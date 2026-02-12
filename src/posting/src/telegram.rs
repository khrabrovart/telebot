use telebot_shared::data::BotData;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InputPollOption, MessageId, ParseMode, Recipient, ThreadId},
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

    pub async fn send_text(
        &self,
        chat_id: Recipient,
        topic_id: Option<String>,
        text: &str,
    ) -> Result<MessageId, anyhow::Error> {
        let text = Self::escape_chars(text);
        let mut request = self
            .bot
            .send_message(chat_id, text)
            .parse_mode(ParseMode::MarkdownV2);

        if let Some(topic_id) = topic_id {
            let thread_id = ThreadId(MessageId(topic_id.parse::<i32>()?));
            request = request.message_thread_id(thread_id);
        }

        let message = request.await?;

        Ok(message.id)
    }

    pub async fn send_poll(
        &self,
        chat_id: Recipient,
        topic_id: Option<String>,
        question: &str,
        options: &[String],
    ) -> Result<MessageId, anyhow::Error> {
        let poll_options: Vec<InputPollOption> = options
            .iter()
            .map(|opt| InputPollOption::new(opt.clone()))
            .collect();

        let mut request = self
            .bot
            .send_poll(chat_id, question, poll_options)
            .is_anonymous(false);

        if let Some(topic_id) = topic_id {
            let thread_id = ThreadId(MessageId(topic_id.parse::<i32>()?));
            request = request.message_thread_id(thread_id);
        }

        let message = request.await?;

        Ok(message.id)
    }

    pub async fn pin_message(
        &self,
        chat_id: Recipient,
        message_id: MessageId,
    ) -> Result<(), anyhow::Error> {
        self.bot
            .pin_chat_message(chat_id, message_id)
            .disable_notification(true)
            .await?;

        Ok(())
    }

    fn escape_chars(text: &str) -> String {
        let mut result = text.to_string();
        result = result.replace("{", "\\{").replace("}", "\\}");
        result
    }
}
