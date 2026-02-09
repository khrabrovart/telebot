use telebot_shared::{Post, PostContent, SsmClient};
use teloxide::{
    prelude::*,
    types::{InputPollOption, Recipient},
};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum TelegramBotError {
    #[error("BOT_TOKEN_PARAMETER environment variable not set")]
    MissingSsmName,

    #[error("Failed to get bot token from SSM: {0}")]
    SsmError(String),

    #[error("Failed to send message: {0}")]
    SendMessageFailed(String),

    #[error("Failed to send poll: {0}")]
    SendPollFailed(String),
}

pub struct TelegramBotClient {
    bot: Bot,
}

impl TelegramBotClient {
    pub async fn from_ssm(ssm: &SsmClient) -> Result<Self, TelegramBotError> {
        let ssm_name =
            std::env::var("BOT_TOKEN_PARAMETER").map_err(|_| TelegramBotError::MissingSsmName)?;

        let token = ssm
            .get_secure_parameter(&ssm_name)
            .await
            .map_err(|e| TelegramBotError::SsmError(e.to_string()))?;

        Ok(Self {
            bot: Bot::new(token),
        })
    }

    pub async fn send_post(&self, post: &Post) -> Result<i32, TelegramBotError> {
        let chat_id: Recipient = post.chat_id.clone().into();

        match &post.content {
            PostContent::Text { text } => self.send_text_message(chat_id, text).await,
            PostContent::Poll { question, options } => {
                self.send_poll(chat_id, question, options).await
            }
        }
    }

    async fn send_text_message(
        &self,
        chat_id: Recipient,
        text: &str,
    ) -> Result<i32, TelegramBotError> {
        info!("Sending text message to chat");

        let message = self
            .bot
            .send_message(chat_id, text)
            .await
            .map_err(|e| TelegramBotError::SendMessageFailed(e.to_string()))?;

        let message_id = message.id.0;
        info!(message_id, "Text message sent successfully");

        Ok(message_id)
    }

    async fn send_poll(
        &self,
        chat_id: Recipient,
        question: &str,
        options: &[String],
    ) -> Result<i32, TelegramBotError> {
        info!("Sending poll to chat");

        let poll_options: Vec<InputPollOption> = options
            .iter()
            .map(|opt| InputPollOption::new(opt.clone()))
            .collect();

        let message = self
            .bot
            .send_poll(chat_id, question, poll_options)
            .await
            .map_err(|e| TelegramBotError::SendPollFailed(e.to_string()))?;

        let message_id = message.id.0;
        info!(message_id, "Poll sent successfully");

        Ok(message_id)
    }
}
