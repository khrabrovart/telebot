use crate::{Post, SsmClient, TelegramBotClient};
use lambda_runtime::{Error, LambdaEvent};
use serde::Deserialize;
use telebot_shared::DynamoDbClient;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
pub struct SchedulerEvent {
    pub posting_data_id: String,
}

pub async fn handle(event: LambdaEvent<SchedulerEvent>) -> Result<(), Error> {
    let (payload, _context) = event.into_parts();

    info!(posting_data_id = %payload.posting_data_id, "Received event");

    let table_name = std::env::var("POSTING_DATA_TABLE")
        .expect("POSTING_DATA_TABLE environment variable not set");

    let db = DynamoDbClient::new().await;
    let post: Post = db.get_item(&table_name, &payload.posting_data_id).await?;

    info!(
        post_id = %post.id,
        content = ?post.content,
        is_active = post.is_active,
        is_ready = post.is_ready,
        "Post retrieved from DynamoDB"
    );

    if !post.is_active {
        warn!(post_id = %post.id, "Post is not active, skipping");
        return Ok(());
    }

    if !post.is_ready {
        warn!(post_id = %post.id, "Post is not fully configured, skipping");
        return Ok(());
    }

    let ssm = SsmClient::from_env().await?;
    let client = TelegramBotClient::from_ssm(&ssm).await?;

    client.send_post(&post).await?;

    Ok(())
}
