use aws_sdk_dynamodb::{types::AttributeValue, Client};
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
    table_name: String,
}

impl DynamoDbClient {
    pub async fn new(table_name: String) -> Self {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        Self { client, table_name }
    }

    pub async fn get_item<T: DeserializeOwned>(&self, id: &str) -> Result<T, DynamoDbError> {
        info!(id, table = %self.table_name, "Fetching item from DynamoDB");

        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("Id", AttributeValue::S(id.to_string()))
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
