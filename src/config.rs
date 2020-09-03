use crate::error::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub server: ServerConfig,
  pub web: WebConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
  pub bind_addr: String,
  pub db_path: String,
  pub env_vars: Vec<[String; 2]>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebConfig {
  pub root_url: String,
  pub secret_key: String,
  pub gmaps_api_key: String,
  pub max_request_size: u32,
  pub rows_per_page: u32,
}

pub fn load(path: &str) -> Result<Config> {
  Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
}
