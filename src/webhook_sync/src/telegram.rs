use telebot_shared::data::BotData;
use teloxide::{prelude::*, types::AllowedUpdate};

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

    pub async fn set_webhook(
        &self,
        url: &str,
        allowed_updates: Vec<AllowedUpdate>,
    ) -> Result<(), anyhow::Error> {
        let url = url.parse()?;
        self.bot
            .set_webhook(url)
            .allowed_updates(allowed_updates)
            .await?;
        Ok(())
    }
}
