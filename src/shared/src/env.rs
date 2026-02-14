use anyhow::{anyhow, Error};

pub fn get_env_var(key: &str) -> Result<String, Error> {
    match std::env::var(key) {
        Ok(val) => Ok(val),
        Err(_) => Err(anyhow!("{} environment variable is not set", key)),
    }
}
