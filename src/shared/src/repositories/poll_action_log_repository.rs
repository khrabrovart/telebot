use anyhow::Error;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use tracing::warn;

use crate::{aws::errors, data::PollActionLog, env};

pub struct PollActionLogRepository {
    client: Client,
    table_name: String,
}

impl PollActionLogRepository {
    pub async fn new() -> Result<Self, Error> {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        let table_name = env::get_env_var("POLL_ACTION_LOG_TABLE")?;

        Ok(Self { client, table_name })
    }

    pub async fn get(&self, id: &str) -> Result<PollActionLog, Error> {
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
            None => Err(anyhow::anyhow!("Poll action log not found for id: {}", id)),
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
