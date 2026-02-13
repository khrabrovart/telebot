use anyhow::{anyhow, Error};
use aws_sdk_apigatewayv2::Client;
use telebot_shared::aws::errors::map_aws_error;

pub struct ApiGatewayClient {
    client: Client,
    api_id: String,
    integration_id: String,
    route_prefix: String,
    region: String,
}

impl ApiGatewayClient {
    pub async fn new() -> Result<Self, Error> {
        let api_id =
            std::env::var("API_ID").map_err(|_| anyhow!("API_ID environment variable not set"))?;

        let integration_id = std::env::var("API_INTEGRATION_ID")
            .map_err(|_| anyhow!("API_INTEGRATION_ID environment variable not set"))?;

        let route_prefix = std::env::var("ROUTE_PREFIX")
            .map_err(|_| anyhow!("ROUTE_PREFIX environment variable not set"))?;

        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        Ok(Self {
            client,
            api_id,
            integration_id,
            route_prefix,
            region: config.region().map(|r| r.as_ref().to_string()).unwrap(),
        })
    }

    fn route_key(&self, bot_id: &str) -> String {
        format!("POST {}{}", self.route_prefix, bot_id)
    }

    pub async fn create_route(&self, bot_id: &str) -> Result<String, anyhow::Error> {
        let route_key = self.route_key(bot_id);

        let response = self
            .client
            .create_route()
            .api_id(&self.api_id)
            .route_key(&route_key)
            .target(format!("integrations/{}", self.integration_id))
            .send()
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to create API Gateway route: {}",
                    e.into_service_error()
                )
            })?;

        let route_key = response.route_key().unwrap_or_default();
        let path = route_key.split_whitespace().last().unwrap_or("/");

        let full_url = format!(
            "https://{}.execute-api.{}.amazonaws.com/{}",
            self.api_id, self.region, path
        );

        Ok(full_url)
    }

    pub async fn delete_route(&self, bot_id: &str) -> Result<(), Error> {
        let route_key = self.route_key(bot_id);

        let routes = self
            .client
            .get_routes()
            .api_id(&self.api_id)
            .send()
            .await
            .map_err(map_aws_error)?;

        if let Some(route) = routes
            .items()
            .iter()
            .find(|r| r.route_key() == Some(&route_key))
        {
            if let Some(route_id) = route.route_id() {
                self.client
                    .delete_route()
                    .api_id(&self.api_id)
                    .route_id(route_id)
                    .send()
                    .await
                    .map_err(map_aws_error)?;
            }
        }
        Ok(())
    }
}
