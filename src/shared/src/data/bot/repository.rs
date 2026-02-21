use crate::{
    aws::{errors, DynamoDbClient},
    data::bot::core::BotData,
    env,
};
use anyhow::Error;
use aws_sdk_dynamodb::{types::AttributeValue, Client};

pub struct BotDataRepository {
    client: Client,
    table_name: String,
}

impl BotDataRepository {
    pub async fn new(dynamodb: &DynamoDbClient) -> Result<Self, Error> {
        let table_name = env::get_env_var("BOTS_TABLE")?;

        Ok(Self {
            client: dynamodb.client.clone(),
            table_name,
        })
    }

    pub async fn get(&self, id: &str) -> Result<Option<BotData>, Error> {
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
}
