use aws_sdk_ssm::Client;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum SsmError {
    #[error("SSM parameter not found: {0}")]
    ParameterNotFound(String),

    #[error("SSM parameter value is empty: {0}")]
    EmptyValue(String),

    #[error("Failed to get SSM parameter: {0}")]
    GetParameterFailed(String),
}

pub struct SsmClient {
    client: Client,
}

impl SsmClient {
    pub async fn from_env() -> Result<Self, aws_sdk_ssm::Error> {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        Ok(Self { client })
    }

    pub async fn get_parameter(
        &self,
        name: &str,
        with_decryption: bool,
    ) -> Result<String, SsmError> {
        info!(parameter_name = %name, "Fetching SSM parameter");

        let response = self
            .client
            .get_parameter()
            .name(name)
            .with_decryption(with_decryption)
            .send()
            .await
            .map_err(|e| SsmError::GetParameterFailed(e.to_string()))?;

        let parameter = response
            .parameter()
            .ok_or_else(|| SsmError::ParameterNotFound(name.to_string()))?;

        let value = parameter
            .value()
            .ok_or_else(|| SsmError::EmptyValue(name.to_string()))?;

        if value.is_empty() {
            return Err(SsmError::EmptyValue(name.to_string()));
        }

        info!(parameter_name = %name, "Successfully retrieved SSM parameter");
        Ok(value.to_string())
    }

    pub async fn get_secure_parameter(&self, name: &str) -> Result<String, SsmError> {
        self.get_parameter(name, true).await
    }
}
