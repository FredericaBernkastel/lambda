use std::time::SystemTime;
use sha2::{Sha256, Digest};
use rand::prelude::*;
use hex_slice::AsHex;
use crate::config::Config;
use crate::model;

type DB = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub fn password_hash(password: String, config: &Config) -> String {
  format!("{:x}", Sha256::digest((password + config.server.password_salt.as_str()).as_bytes()))
}

fn gen_ssid() -> String {
  let mut data = [0u8; 32];
  rand::thread_rng().fill_bytes(&mut data);
  format!("{:x}", data.plain_hex(false))
}

fn get_timestamp() -> u64 {
  SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}

pub fn check_session(ssid: String, db: DB) -> Option<model::User> {
  if ssid.len() != 64 { return None; }

  // query session
  let session = match db.query_row("select * from `sessions` where `ssid` = :ssid", params![ssid], |row| {
    Ok(model::Session{
      id: row.get(0)?,
      uid: row.get(1)?,
      expires: row.get::<_, i64>(2)? as u64,
    })
  }).ok() {
    Some(session) => session,
    None => return None
  };

  // check for expired
  if session.expires < get_timestamp() {
    return None;
  }

  // query user
  db.query_row("select * from `users` where `id` = :id", params![session.uid], |row| {
    Ok(model::User {
      id: row.get(0)?,
      login: row.get(1)?,
      password: row.get(2)?,
    })
  }).ok()
}