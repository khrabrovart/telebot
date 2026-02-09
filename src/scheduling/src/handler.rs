use crate::{Post, SchedulerClient, StreamAction};
use aws_lambda_events::event::dynamodb::Event;
use lambda_runtime::{Error, LambdaEvent};
use serde_dynamo;
use tracing::{info, warn};

pub async fn handle(event: LambdaEvent<Event>) -> Result<(), Error> {
    let (payload, _context) = event.into_parts();

    let scheduler = SchedulerClient::new().await;

    for record in payload.records {
        let event_name = record.event_name.clone();
        info!(event_name = %event_name, "Processing DynamoDB stream record");

        let action = StreamAction::from_event_name(&event_name);

        match action {
            StreamAction::Insert | StreamAction::Modify => {
                let new_image = match &record.change.new_image {
                    image if image.is_empty() => {
                        warn!("No new image found in record, skipping");
                        continue;
                    }
                    image => image,
                };

                let post: Result<Post, _> = serde_dynamo::from_item(new_image.clone());
                let post = match post {
                    Ok(post) => post,
                    Err(e) => {
                        warn!(error = %e, "Failed to parse post, skipping");
                        continue;
                    }
                };

                info!(
                    id = %post.id,
                    schedule = %post.schedule,
                    is_active = post.is_active,
                    is_ready = post.is_ready,
                    "Parsed post"
                );

                if !post.is_ready {
                    warn!(id = %post.id, "Post is not fully configured, skipping");
                    continue;
                }

                info!(id = %post.id, "Creating/updating scheduler");
                scheduler.create_or_update_schedule(&post).await?;
            }
            StreamAction::Remove => {
                let old_image = match &record.change.old_image {
                    image if image.is_empty() => {
                        warn!("No old image found in record, skipping");
                        continue;
                    }
                    image => image,
                };

                let post: Result<Post, _> = serde_dynamo::from_item(old_image.clone());
                let post = match post {
                    Ok(post) => post,
                    Err(e) => {
                        warn!(error = %e, "Failed to parse post from old image, skipping");
                        continue;
                    }
                };

                info!(id = %post.id, "Deleting scheduler for removed record");
                scheduler.delete_schedule(&post.id).await?;
            }
            StreamAction::Unknown => {
                warn!(event_name = %event_name, "Unknown event type, skipping");
            }
        }
    }

    Ok(())
}
