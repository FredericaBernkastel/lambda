use crate::{config::Config, error::Result};
use chrono::{prelude::DateTime, Utc};
use error_chain::bail;
use maud::{html, Markup};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[macro_export] macro_rules! map(
  { $($key:expr => $value:expr),+ } => {
    {
      let mut m = ::std::collections::HashMap::<String, _>::new();
      $(
          m.insert($key.into(), $value);
      )+
      m
    }
  };
);

pub fn get_timestamp() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs()
}

pub fn format_timestamp(timestamp: u64, fmt: &str) -> String {
  DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(timestamp))
    .format(fmt)
    .to_string()
}

pub fn gen_cors_hash(timestamp: u64, config: &Config) -> String {
  format!(
    "{:x},{}",
    Sha256::digest(format!("{}{}", config.web.secret_key, timestamp).as_bytes()),
    timestamp
  )
}

pub fn check_cors_hash(hash: &str, config: &Config) -> bool {
  let tokens: Vec<&str> = hash.split(",").collect();
  if tokens.len() != 2 {
    return false;
  }
  let timestamp = match tokens[1].parse::<u64>() {
    Ok(t) => t,
    Err(_) => return false,
  };
  gen_cors_hash(timestamp, config) == hash && timestamp < get_timestamp()
}

pub fn markup_br(text: String) -> Markup {
  html! {
  @for line in text.lines() {
    (line) br;
  }}
}

pub fn strip_slashes(mut uri: String) -> String {
  if uri.ends_with('/') {
    uri.pop();
  }
  "/".to_string() + &uri
}

pub fn redirect(path: &str, config: &Config) -> actix_web::HttpResponse {
  actix_web::HttpResponse::Found()
    .header("location", config.web.root_url.clone() + path)
    .finish()
}

pub async fn read_payload(
  mut payload: actix_web::web::Payload,
  config: &Config,
) -> Result<bytes::Bytes> {
  use futures::StreamExt;

  // payload is a stream of Bytes objects
  let mut post_data = bytes::BytesMut::new();
  while let Some(chunk) = payload.next().await {
    let chunk = chunk?;
    // limit max size of in-memory payload
    if (post_data.len() + chunk.len()) > config.web.max_request_size as usize {
      bail!("overflow");
    }
    post_data.extend_from_slice(&chunk);
  }
  Ok(post_data.freeze())
}

pub fn json_path<'a, T: serde::Deserialize<'a>>(
  data: &'a mut serde_json::Value,
  path: &'a str,
) -> Result<T> {
  let value = data
    .pointer_mut(path)
    .map(serde_json::Value::take)
    .ok_or(format!("unable to extract value \"{}\"", path))?;
  Ok(T::deserialize(value)?)
}

pub struct DynQuery {
  pub sql: String,
  pub params: Vec<(String, Box<dyn rusqlite::ToSql>)>,
}

impl DynQuery {
  pub fn new() -> Self {
    Self {
      sql: String::new(),
      params: vec![],
    }
  }

  pub fn push(&mut self, sql: &str) -> &mut Self {
    self.sql.push_str(sql);
    self
  }

  pub fn bind(&mut self, k: String, v: impl rusqlite::ToSql + 'static) -> &mut Self
  {
    self.params.push((k, box v));
    self
  }
}

pub fn datetime_variable(datetime: &str) -> Option<i64> {
  use chrono::prelude::*;
  Some(
    DateTime::<Utc>::from_utc(
      NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M")
        .ok()
        .or(
          NaiveDate::parse_from_str(&datetime, "%Y-%m-%d")
            .ok()
            .map(|x| x.and_hms(0, 0, 0)),
        )?,
      Utc,
    )
    .timestamp(),
  )
}

pub fn b64_gunzip_deserialize_t<'a, T: serde::Deserialize<'a>>(data: &str) -> Result<T> {
  let base64 = base64::decode_config(data, base64::URL_SAFE)?;
  let gz = flate2::read::GzDecoder::new(base64.as_slice())
    // zip bomb DoS
    .take(16 * 1024);
  let mut de = serde_json::Deserializer::from_reader(gz);
  Ok(T::deserialize(&mut de)?)
}
