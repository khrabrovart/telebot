use anyhow::Error;
use aws_sdk_dynamodb::{types::AttributeValue, Client};

use crate::{aws, data::PollActionLog, env};

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
            .map_err(aws::errors::map_aws_error)?;

        match result.item {
            Some(item) => Ok(serde_dynamo::from_item(item)?),
            None => Err(anyhow::anyhow!("Poll action log not found for id: {}", id)),
        }
    }

    pub async fn put(&self, item: &PollActionLog) -> Result<(), Error> {
        let current_version = item.version;

        let mut item = item.clone();
        item.version += 1;

        let item = serde_dynamo::to_item(item)?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .condition_expression("attribute_not_exists(Id) OR Version = :version")
            .expression_attribute_values(":version", AttributeValue::N(current_version.to_string()))
            .send()
            .await
            .map_err(aws::errors::map_aws_error)?;

        Ok(())
    }
}
