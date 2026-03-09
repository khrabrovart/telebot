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

    pub async fn get_by_chat_and_message(
        &self,
        chat_id: i64,
        message_id: i32,
    ) -> Result<Option<Post>, Error> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("ChatId", AttributeValue::N(chat_id.to_string()))
            .key("MessageId", AttributeValue::N(message_id.to_string()))
            .send()
            .await
            .map_err(errors::map_aws_error)?;

        match result.item {
            Some(item) => Ok(serde_dynamo::from_item(item)?),
            None => Ok(None),
        }
    }

    pub async fn get_most_recent_by_posting_rule(
        &self,
        posting_rule_id: &str,
    ) -> Result<Option<Post>, Error> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("PostingRuleIndex")
            .key_condition_expression("PostingRuleId = :rule_id")
            .expression_attribute_values(":rule_id", AttributeValue::S(posting_rule_id.to_string()))
            .scan_index_forward(false)
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
