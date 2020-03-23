use std::time::SystemTime;
use sha2::{Sha256, Digest};
use crate::config::Config;

pub fn get_timestamp() -> u64 {
  SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}

pub fn gen_cors_hash(timestamp: u64, config: &Config) -> String {
  format!("{:x},{}", Sha256::digest(
    format!("{}{}", config.web.secret_key, timestamp).as_bytes()),
    timestamp
  )
}

pub fn check_cors_hash(hash: &str, config: &Config) -> bool {
  let tokens:Vec<&str> = hash.split(",").collect();
  if tokens.len() != 2 { return false; }
  let timestamp = match tokens[1].parse::<u64>(){
    Ok(t) => t,
    Err(_) => return false
  };
  gen_cors_hash(timestamp, config) == hash &&
    timestamp < get_timestamp()
}