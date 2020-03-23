use sha2::{Sha256, Digest};
use hex_slice::AsHex;
use actix_session::Session;
use actix_web::web;
use crate::{util, config::Config, model, DB};

#[derive(Debug)]
pub enum Error {
  InternalError { description: String },
  InvalidLogin
}
impl From<rusqlite::Error> for Error {
  fn from(error: rusqlite::Error) -> Self {
    return Error::InternalError { description: error.to_string() };
}}
impl<T> From<actix_web::error::BlockingError<T>> for Error
  where T: std::fmt::Debug {
  fn from(error: actix_web::error::BlockingError<T>) -> Self {
    return Error::InternalError { description: error.to_string() };
}}

pub fn password_hash(password: &String, config: &Config) -> String {
  format!("{:02x}", Sha256::digest(format!("{}{}", password, config.web.secret_key).as_bytes()))
}

pub fn gen_ssid() -> String {
  use rand::prelude::*;

  let mut data = [0u8; 32];
  rand::thread_rng().fill_bytes(&mut data);
  format!("{:02x}", data.plain_hex(false))
}

pub async fn check_session(ssid: &String, db: DB) -> Option<model::User> {
  let ssid = ssid.clone();
  if ssid.len() != 64 { return None; }

  web::block(move || -> Result<model::User, Error> {
    let db = db.get().unwrap();

    // query session
    let session = db.query_row("select * from `sessions` where `id` = :ssid", params![ssid], |row| {
      Ok(model::Session{
        id: row.get(0)?,
        uid: row.get(1)?,
        expires: row.get::<_, i64>(2)? as u64,
      })
    })?;

    // check for expired
    if session.expires < util::get_timestamp() {
      return Err(Error::InvalidLogin);
    }

    // query user
    Ok(db.query_row("select * from `users` where `id` = :id", params![session.uid], |row| {
      Ok(model::User {
        id: row.get(0)?,
        login: row.get(1)?,
        password: row.get(2)?,
      })
    })?)
  }).await
    .map_err(|e| eprintln!("{}", e.to_string()))
    .ok()
}

pub async fn login(login: &String, password: &String, db: DB, config: &Config, session: Session) -> Result<(), Error> {
  // query user by login
  let user = web::block({
    let login = login.clone();
    let db = db.clone();
    move || {
    db.get().unwrap().query_row("select * from `users` where `login` = :login", params![login], |row| {
      Ok(model::User {
        id: row.get(0)?,
        login: row.get(1)?,
        password: row.get(2)?,
      })
    })
  }}).await.map_err(|_| Error::InvalidLogin)?;

  // check password hash
  if user.password != password_hash(password, config) { return Err(Error::InvalidLogin); }

  let timestamp = util::get_timestamp();

  // generate ssid
  let ssid = gen_ssid();
  let expires = timestamp + 2592000; // 1 month
  let ssid_cookie = ssid.clone();

  web::block({
    let db = db.clone();
    move || -> Result<(), Error> {
      let db = db.get().unwrap();

      // TODO: export iface to cron
      // delete expired sessions
      db.execute_named(
        "delete from `sessions` where `uid` = :uid and `expires` < :timestamp",
        named_params![
        ":uid": user.id,
        ":timestamp": timestamp as i64
      ])?;

      db.execute_named(
        "insert into sessions (`id`, `uid`, `expires`) values (:id, :uid, :expires)",
        named_params![
        ":id": ssid,
        ":uid": user.id,
        ":expires": expires as i64
      ])?;

      Ok(())
    }
  }).await?;

  // set session cookie
  session.set("ssid", ssid_cookie)
    .map_err(|e| Error::InternalError {description: e.to_string()})?;

  Ok(())
}

pub async fn logout(db: DB, session: Session) -> Result<(), Error> {
  let ssid = match session.get::<String>("ssid").ok().flatten() {
    Some(ssid) => ssid,
    None => return Err(Error::InvalidLogin)
  };

  web::block(move || {
    db.get().unwrap().execute(
      "delete from `sessions` where `id` = :ssid",
      params![ssid]
    )
  }).await?;

  session.remove("ssid");
  Ok(())
}