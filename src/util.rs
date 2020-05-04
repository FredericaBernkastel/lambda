use std::time::{SystemTime, UNIX_EPOCH, Duration};
use sha2::{Sha256, Digest};
use chrono::{Utc, prelude::DateTime};
use maud::{html, Markup};
use crate::{
  web_error::WebError,
  config::Config
};

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

pub fn markup_br(text: String) -> Markup {
  html! {
    @for line in text.lines() {
      (line) br;
    }}
}

pub fn strip_slashes(mut uri: String) -> String {
  if uri.ends_with('/') { uri.pop(); }
  "/".to_string() + &uri
}

pub fn redirect(path: &str, config: &Config) -> actix_web::HttpResponse {
  actix_web::HttpResponse::Found()
    .header("location", config.web.root_url.clone() + path)
    .finish()
}

pub async fn read_payload(mut payload: actix_web::web::Payload, config: &Config) -> Result<bytes::Bytes, actix_web::Error> {
  use futures::StreamExt;

  // payload is a stream of Bytes objects
  let mut post_data = bytes::BytesMut::new();
  while let Some(chunk) = payload.next().await {
    let chunk = chunk?;
    // limit max size of in-memory payload
    if (post_data.len() + chunk.len()) > config.web.max_request_size as usize {
      return Err(actix_web::error::ErrorBadRequest("overflow"));
    }
    post_data.extend_from_slice(&chunk);
  }
  Ok(post_data.freeze())
}

pub fn json_path<'a, T: serde::Deserialize<'a>>(data: &'a mut serde_json::Value, path: &'a str) -> Result<T, WebError> {
  let value = data.pointer_mut(path)
      .map(serde_json::Value::take)
      .ok_or(format!("unable to extract value \"{}\"", path))?;
  Ok(T::deserialize(value)?)
}