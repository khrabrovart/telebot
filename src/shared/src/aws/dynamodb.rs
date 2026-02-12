use anyhow::Error;
use aws_sdk_dynamodb::{types::AttributeValue as AwsAttributeValue, Client};
use serde::de::DeserializeOwned;

use crate::aws::errors::map_aws_error;

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
            .map_err(map_aws_error)?;

        if let Some(item) = result.item {
            Ok(Some(serde_dynamo::from_item(item)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all<T: DeserializeOwned>(&self, table_name: &str) -> Result<Vec<T>, Error> {
        let result = self
            .client
            .scan()
            .table_name(table_name)
            .send()
            .await
            .map_err(map_aws_error)?;

        let items = result
            .items
            .unwrap_or_default()
            .into_iter()
            .map(|item| serde_dynamo::from_item(item))
            .collect::<Result<Vec<T>, _>>()?;

        Ok(items)
    }

    pub async fn put_item<T: serde::Serialize>(
        &self,
        table_name: &str,
        item: &T,
    ) -> Result<(), Error> {
        let item = serde_dynamo::to_item(item)?;

        self.client
            .put_item()
            .table_name(table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(map_aws_error)?;

        Ok(())
    }
}
