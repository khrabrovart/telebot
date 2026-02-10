use crate::{SchedulerClient, StreamAction};
use aws_lambda_events::event::dynamodb::Event;
use lambda_runtime::{Error, LambdaEvent};
use serde_dynamo;
use telebot_shared::data::PostingRule;
use tracing::warn;

pub async fn handle(event: LambdaEvent<Event>) -> Result<(), Error> {
    let (payload, _context) = event.into_parts();

    let scheduler = SchedulerClient::new().await?;

    if let Some(record) = payload.records.get(0) {
        let action = StreamAction::from_event_name(&record.event_name);

        match action {
            StreamAction::Insert | StreamAction::Modify => {
                let posting_rule: PostingRule =
                    serde_dynamo::from_item(record.change.new_image.clone())?;

                process_update(&posting_rule, &scheduler).await?;
            }
            StreamAction::Remove => {
                let posting_rule: PostingRule =
                    serde_dynamo::from_item(record.change.old_image.clone())?;

                process_remove(&posting_rule, &scheduler).await?;
            }
            StreamAction::Unknown => {
                warn!(event_name = %record.event_name, "Unknown event type, skipping");
                return Err(format!("Unknown event type: {}", record.event_name).into());
            }
        }
    }

    Ok(())
}

async fn process_update(
    posting_rule: &PostingRule,
    scheduler: &SchedulerClient,
) -> Result<(), Error> {
    if !posting_rule.is_valid() {
        scheduler.delete_schedule(&posting_rule.id).await?;
        return Ok(());
    }

    scheduler.create_or_update_schedule(posting_rule).await?;

    Ok(())
}

async fn process_remove(
    posting_rule: &PostingRule,
    scheduler: &SchedulerClient,
) -> Result<(), Error> {
    scheduler.delete_schedule(&posting_rule.id).await?;
    Ok(())
}
