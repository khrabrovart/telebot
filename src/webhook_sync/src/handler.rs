use crate::{ApiGatewayClient, StreamAction};
use aws_lambda_events::event::dynamodb::Event;
use lambda_runtime::{Error, LambdaEvent};
use serde_dynamo;
use telebot_shared::data::BotData;
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

                api.create_route(&bot_data.id).await?;
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
