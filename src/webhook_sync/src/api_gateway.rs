use anyhow::{anyhow, Error};
use aws_sdk_apigatewayv2::Client;

pub struct ApiGatewayClient {
    client: Client,
    api_id: String,
    integration_id: String,
    route_prefix: String,
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
        })
    }

    fn route_key(&self, bot_id: &str) -> String {
        format!("POST {}{}", self.route_prefix, bot_id)
    }

    pub async fn create_route(&self, bot_id: &str) -> Result<(), anyhow::Error> {
        let route_key = self.route_key(bot_id);

        self.client
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

        Ok(())
    }

    pub async fn delete_route(&self, bot_id: &str) -> Result<(), Error> {
        let route_key = self.route_key(bot_id);

        let routes = self
            .client
            .get_routes()
            .api_id(&self.api_id)
            .send()
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to get API Gateway routes: {}",
                    e.into_service_error()
                )
            })?;

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
                    .map_err(|e| {
                        anyhow!(
                            "Failed to delete API Gateway route: {}",
                            e.into_service_error()
                        )
                    })?;
            }
        }
        Ok(())
    }
}
