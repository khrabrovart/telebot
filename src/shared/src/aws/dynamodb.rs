use aws_config::SdkConfig;
use aws_sdk_dynamodb::Client;

pub struct DynamoDbClient {
    pub client: Client,
}

impl DynamoDbClient {
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        Self { client }
    }

    pub async fn new_slim(config: SdkConfig) -> Self {
        let client = Client::new(&config);
        Self { client }
    }
}
