use std::time::{SystemTime, UNIX_EPOCH, Duration};
use sha2::{Sha256, Digest};
use chrono::{Utc, prelude::DateTime};
use crate::config::Config;

pub fn get_timestamp() -> u64 {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub fn format_timestamp(timestamp: u64, fmt: &str) -> String {
  DateTime::<Utc>::from(
    UNIX_EPOCH + Duration::from_secs(timestamp)
  )
    .format(fmt)
    .to_string()
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