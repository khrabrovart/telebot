use telebot_shared::data::BotData;
use teloxide::{
    prelude::*,
    types::{InputPollOption, Recipient},
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

    pub async fn send_text(&self, chat_id: Recipient, text: &str) -> Result<(), anyhow::Error> {
        self.bot.send_message(chat_id, text).await?;

        Ok(())
    }

    pub async fn send_poll(
        &self,
        chat_id: Recipient,
        question: &str,
        options: &[String],
    ) -> Result<(), anyhow::Error> {
        let poll_options: Vec<InputPollOption> = options
            .iter()
            .map(|opt| InputPollOption::new(opt.clone()))
            .collect();

        self.bot.send_poll(chat_id, question, poll_options).await?;

        Ok(())
    }
}
