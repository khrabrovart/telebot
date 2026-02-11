use telebot_shared::data::BotData;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InputPollOption, MessageId, Recipient, ThreadId},
};

pub struct TelegramBotClient {
    bot: Bot,
}

impl TelegramBotClient {
    pub async fn new(bot_data: &BotData) -> Result<Self, anyhow::Error> {
        Ok(Self {
            bot: Bot::new(bot_data.token.clone()),
        })
    }

    pub async fn send_text(
        &self,
        chat_id: Recipient,
        topic_id: Option<String>,
        text: &str,
    ) -> Result<(), anyhow::Error> {
        let mut msg = self.bot.send_message(chat_id, text);

        if let Some(topic_id) = topic_id {
            let thread_id = ThreadId(MessageId(topic_id.parse::<i32>()?));
            msg = msg.message_thread_id(thread_id);
        }

        msg.await?;

        Ok(())
    }

    pub async fn send_poll(
        &self,
        chat_id: Recipient,
        topic_id: Option<String>,
        question: &str,
        options: &[String],
    ) -> Result<(), anyhow::Error> {
        let poll_options: Vec<InputPollOption> = options
            .iter()
            .map(|opt| InputPollOption::new(opt.clone()))
            .collect();

        let mut msg = self
            .bot
            .send_poll(chat_id, question, poll_options)
            .is_anonymous(false);

        if let Some(topic_id) = topic_id {
            let thread_id = ThreadId(MessageId(topic_id.parse::<i32>()?));
            msg = msg.message_thread_id(thread_id);
        }

        msg.await?;

        Ok(())
    }
}
