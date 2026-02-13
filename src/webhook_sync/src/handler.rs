use crate::{telegram::TelegramBotClient, ApiGatewayClient, StreamAction};
use aws_lambda_events::event::dynamodb::Event;
use lambda_runtime::{Error, LambdaEvent};
use serde_dynamo;
use telebot_shared::data::BotData;
use teloxide::types::AllowedUpdate;
use tracing::info;

pub async fn handle(event: LambdaEvent<Event>) -> Result<(), Error> {
    let (payload, _context) = event.into_parts();

    let api = ApiGatewayClient::new().await?;

    if let Some(record) = payload.records.first() {
        let action = StreamAction::from_event_name(&record.event_name);

        info!(?record, "Received DynamoDB record");

        match action {
            StreamAction::Insert => {
                let bot_data: BotData = serde_dynamo::from_item(record.change.new_image.clone())?;

                let url = api.create_route(&bot_data.id).await?;

                let bot = TelegramBotClient::new(&bot_data).await?;

                let allowed_updates = vec![
                    AllowedUpdate::Message,
                    AllowedUpdate::CallbackQuery,
                    AllowedUpdate::PollAnswer,
                ];

                bot.set_webhook(&url, allowed_updates).await?;
            }
            StreamAction::Remove => {
                let bot_data: BotData = serde_dynamo::from_item(record.change.old_image.clone())?;

                api.delete_route(&bot_data.id).await?;
            }
            StreamAction::Modify => {}
            StreamAction::Unknown => {
                return Err(format!("Unknown event type: {}", record.event_name).into());
            }
        }
    }

    Ok(())
}
