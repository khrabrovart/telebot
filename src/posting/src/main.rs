use lambda_runtime::{service_fn, Error};
use posting_lambda::handle;
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

    info!("Starting Posting Lambda");

    lambda_runtime::run(service_fn(handle)).await
}
