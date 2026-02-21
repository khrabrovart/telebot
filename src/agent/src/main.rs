use agent_lambda::AppContext;
use lambda_http::{run, service_fn, Error};

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

    let config = aws_config::load_from_env().await;
    let app = AppContext::from_config(config).await;

    run(service_fn(|event| agent_lambda::handle(event, &app))).await
}
