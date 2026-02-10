use anyhow::{anyhow, Error};
use aws_sdk_dynamodb::{types::AttributeValue as AwsAttributeValue, Client};
use serde::de::DeserializeOwned;

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
    ) -> Result<Option<T>, Error> {
        let result = self
            .client
            .get_item()
            .table_name(table_name)
            .key("Id", AwsAttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        if let Some(item) = result.item {
            Ok(Some(serde_dynamo::from_item(item)?))
        } else {
            Ok(None)
        }
    }
}
