use aws_sdk_dynamodb::{types::AttributeValue as AwsAttributeValue, Client};
use serde::de::DeserializeOwned;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum DynamoDbError {
    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("DynamoDB error: {0}")]
    ClientError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

pub struct DynamoDbClient {
    client: Client,
}

impl DynamoDbClient {
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        Self { client }
    }

    pub async fn get_item<T: DeserializeOwned>(
        &self,
        table_name: &str,
        id: &str,
    ) -> Result<T, DynamoDbError> {
        info!(id, table = %table_name, "Fetching item from DynamoDB");

        let result = self
            .client
            .get_item()
            .table_name(table_name)
            .key("Id", AwsAttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|e| DynamoDbError::ClientError(e.to_string()))?;

        let item = result
            .item
            .ok_or_else(|| DynamoDbError::NotFound(id.to_string()))?;

        serde_dynamo::from_item(item)
            .map_err(|e| DynamoDbError::DeserializationError(e.to_string()))
    }
}
