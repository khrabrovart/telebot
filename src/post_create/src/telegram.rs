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
        topic_id: Option<MessageId>,
        text: &str,
    ) -> Result<Message, anyhow::Error> {
        let mut request = self
            .bot
            .send_message(chat_id, text)
            .parse_mode(ParseMode::Html);

        if let Some(topic_id) = topic_id {
            let thread_id = ThreadId(topic_id);
            request = request.message_thread_id(thread_id);
        }

        let message = request.await?;

        Ok(message)
    }

    pub async fn send_poll(
        &self,
        chat_id: Recipient,
        topic_id: Option<MessageId>,
        question: &str,
        options: &[String],
    ) -> Result<Message, anyhow::Error> {
        let poll_options: Vec<InputPollOption> = options
            .iter()
            .map(|opt| InputPollOption::new(opt.clone()))
            .collect();

        let mut request = self
            .bot
            .send_poll(chat_id, question, poll_options)
            .is_anonymous(false);

        if let Some(topic_id) = topic_id {
            let thread_id = ThreadId(topic_id);
            request = request.message_thread_id(thread_id);
        }

        let message = request.await?;

        Ok(message)
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
}
