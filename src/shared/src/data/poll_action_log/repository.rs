use crate::{aws::errors, data::poll_action_log::PollActionLog, env};
use anyhow::Error;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use tracing::warn;

pub struct PollActionLogRepository {
    client: Client,
    table_name: String,
}

impl PollActionLogRepository {
    pub async fn new(dynamodb_client: Client) -> Result<Self, Error> {
        let table_name = env::get_env_var("POLL_ACTION_LOG_TABLE")?;

        Ok(Self {
            client: dynamodb_client,
            table_name,
        })
    }

    pub async fn get_by_poll_id(&self, id: &str) -> Result<Option<PollActionLog>, Error> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("Id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(errors::map_aws_error)?;

        match result.item {
            Some(item) => Ok(serde_dynamo::from_item(item)?),
            None => Ok(None),
        }
    }

    pub async fn get_by_chat_and_message(
        &self,
        chat_id: i64,
        message_id: i32,
    ) -> Result<Option<PollActionLog>, Error> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("ChatMessageIndex")
            .key_condition_expression("ChatId = :chat_id AND MessageId = :message_id")
            .expression_attribute_values(":chat_id", AttributeValue::N(chat_id.to_string()))
            .expression_attribute_values(":message_id", AttributeValue::N(message_id.to_string()))
            .limit(1)
            .send()
            .await
            .map_err(errors::map_aws_error)?;

        match result.items {
            Some(items) if !items.is_empty() => Ok(Some(serde_dynamo::from_item(
                items.into_iter().next().unwrap(),
            )?)),
            _ => Ok(None),
        }
    }

    // TODO: Use the result of this method to implement optimistic locking and handle conflicts in the caller

    pub async fn put(&self, item: &PollActionLog) -> Result<bool, Error> {
        let current_version = item.version;

        let mut item = item.clone();
        item.version += 1;

        let item = serde_dynamo::to_item(item)?;

        let result = self
            .client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .condition_expression("attribute_not_exists(Id) OR Version = :version")
            .expression_attribute_values(":version", AttributeValue::N(current_version.to_string()))
            .send()
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(err) => {
                if let Some(service_error) = err.as_service_error() {
                    if service_error.is_conditional_check_failed_exception() {
                        warn!("Conflict: version mismatch");
                        return Ok(false);
                    }
                }

                Err(errors::map_aws_error(err))
            }
        }
    }
}
