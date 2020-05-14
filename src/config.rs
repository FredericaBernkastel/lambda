use serde::Deserialize;
use crate::error::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub server: ServerConfig,
  pub web: WebConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
  pub bind_addr: String,
  pub db_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebConfig {
  pub root_url: String,
  pub secret_key: String,
  pub max_request_size: u32
}

pub fn load() -> Result<Config> {
  const PATH: &str = "data/config.toml";
  Ok(
    toml::from_str(
      &std::fs::read_to_string(PATH)?
    )?
  )
}