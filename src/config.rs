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
  pub password_salt: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebConfig {
  pub root_url: String
}

pub fn load() -> Config {
  let path = "data/config.toml";
  std::fs::read_to_string(path)
    .map_err(|e| e.to_string())
    .and_then(|file| toml::from_str(&file)
      .map_err(|e| e.to_string()))
    .unwrap_or_else(|e| panic!("Unable to load \"{}\"\n\n{}", path, e))
}