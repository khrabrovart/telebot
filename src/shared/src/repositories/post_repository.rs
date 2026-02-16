use anyhow::Error;
use aws_sdk_dynamodb::{types::AttributeValue, Client};

use crate::{aws::errors, data::Post, env};

pub struct PostRepository {
    client: Client,
    table_name: String,
}

impl PostRepository {
    pub async fn new(dynamodb_client: Client) -> Result<Self, Error> {
        let table_name = env::get_env_var("POSTS_TABLE")?;

        Ok(Self {
            client: dynamodb_client,
            table_name,
        })
    }

    pub async fn get(&self, id: &str) -> Result<Post, Error> {
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
            None => Err(anyhow::anyhow!("Post not found for id: {}", id)),
        }
    }

    pub async fn put(&self, item: &Post) -> Result<(), Error> {
        let item = serde_dynamo::to_item(item)?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(errors::map_aws_error)?;

        Ok(())
    }
}
