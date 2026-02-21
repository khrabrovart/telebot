use crate::{aws::errors, data::PostingRule, env};
use anyhow::Error;
use aws_sdk_dynamodb::{types::AttributeValue, Client};

pub struct PostingRuleRepository {
    client: Client,
    table_name: String,
}

impl PostingRuleRepository {
    pub async fn new(dynamodb_client: Client) -> Result<Self, Error> {
        let table_name = env::get_env_var("POSTING_RULES_TABLE")?;

        Ok(Self {
            client: dynamodb_client,
            table_name,
        })
    }

    pub async fn get(&self, id: &str) -> Result<Option<PostingRule>, Error> {
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

    pub async fn get_all(&self) -> Result<Vec<PostingRule>, Error> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .send()
            .await
            .map_err(errors::map_aws_error)?;

        let items = result
            .items
            .unwrap()
            .into_iter()
            .map(serde_dynamo::from_item)
            .collect::<Result<Vec<PostingRule>, _>>()?;

        Ok(items)
    }

    pub async fn put_item(&self, item: &PostingRule) -> Result<(), Error> {
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
