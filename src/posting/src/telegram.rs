use crate::ssm::SsmClient;
use crate::{Post, PostContent};
use teloxide::{
    prelude::*,
    types::{InputPollOption, Recipient},
};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum TelegramError {
    #[error("BOT_TOKEN_PARAMETER environment variable not set")]
    MissingSsmName,

    #[error("Failed to get bot token from SSM: {0}")]
    SsmError(String),

    #[error("Failed to send message: {0}")]
    SendMessageFailed(String),

    #[error("Failed to send poll: {0}")]
    SendPollFailed(String),

    #[error("Post validation error: {0}")]
    ValidationError(String),
}

pub struct TelegramClient {
    bot: Bot,
}

impl TelegramClient {
    pub async fn from_ssm(ssm: &SsmClient) -> Result<Self, TelegramError> {
        let ssm_name =
            std::env::var("BOT_TOKEN_PARAMETER").map_err(|_| TelegramError::MissingSsmName)?;

        let token = ssm
            .get_secure_parameter(&ssm_name)
            .await
            .map_err(|e| TelegramError::SsmError(e.to_string()))?;

        Ok(Self {
            bot: Bot::new(token),
        })
    }

    pub async fn send_post(&self, post: &Post) -> Result<i32, TelegramError> {
        post.validate()
            .map_err(|e| TelegramError::ValidationError(e.to_string()))?;

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
    ) -> Result<i32, TelegramError> {
        info!("Sending text message to chat");

        let message = self
            .bot
            .send_message(chat_id, text)
            .await
            .map_err(|e| TelegramError::SendMessageFailed(e.to_string()))?;

        let message_id = message.id.0;
        info!(message_id, "Text message sent successfully");

        Ok(message_id)
    }

    async fn send_poll(
        &self,
        chat_id: Recipient,
        question: &str,
        options: &[String],
    ) -> Result<i32, TelegramError> {
        info!("Sending poll to chat");

        let poll_options: Vec<InputPollOption> = options
            .iter()
            .map(|opt| InputPollOption::new(opt.clone()))
            .collect();

        let message = self
            .bot
            .send_poll(chat_id, question, poll_options)
            .await
            .map_err(|e| TelegramError::SendPollFailed(e.to_string()))?;

        let message_id = message.id.0;
        info!(message_id, "Poll sent successfully");

        Ok(message_id)
    }
}
