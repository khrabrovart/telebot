use aws_config::SdkConfig;
use telebot_shared::aws::DynamoDbClient;

pub struct AppContext {
    pub dynamodb: DynamoDbClient,
}

impl AppContext {
    pub async fn from_config(config: SdkConfig) -> Self {
        let dynamodb = DynamoDbClient::new_slim(config).await;

        Self { dynamodb }
    }
}
