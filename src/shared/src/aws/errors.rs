use anyhow::{anyhow, Error};
use aws_sdk_dynamodb::{error::SdkError, operation::RequestId};
use aws_smithy_types::error::metadata::ProvideErrorMetadata;

pub fn map_aws_error<E, R>(err: SdkError<E, R>) -> Error
where
    E: ProvideErrorMetadata + std::error::Error + Send + Sync + 'static,
    R: std::fmt::Debug,
{
    match err {
        SdkError::ServiceError(context) => {
            let meta = context.err().meta();
            let code = meta.code().unwrap_or("UnknownCode");
            let message = meta.message().unwrap_or("No message provided");
            let req_id = meta.request_id().unwrap_or("NoReqID");

            anyhow!(
                "AWS Service Error: [{}] {} (Request ID: {})",
                code,
                message,
                req_id
            )
            .context(format!("Full Service Error: {:?}", context.err()))
        }
        _ => anyhow!("AWS SDK Error: {:?}", err),
    }
}
