use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_target(false)
        .without_time()
        .init();

    info!("Starting Agent Lambda");

    lambda_runtime::run(service_fn(handler)).await
}

async fn handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let (event, _context) = event.into_parts();
    info!(?event, "Received API Gateway event");

    // Parse Telegram update from body
    let body = match &event.body {
        Some(b) => b,
        None => {
            return Ok(ApiGatewayProxyResponse {
                status_code: 400,
                body: Some(aws_lambda_events::encodings::Body::Text(
                    "Missing body".to_string(),
                )),
                ..Default::default()
            });
        }
    };

    // For now, just echo a simple message back (simulate Telegram reply)
    // In a real implementation, parse the Telegram update and send a reply via Telegram API
    info!(body, "Received Telegram webhook body");

    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        body: Some(aws_lambda_events::encodings::Body::Text(
            "{\"ok\":true}".to_string(),
        )),
        ..Default::default()
    })
}
