use sha2::{Sha256, Digest};
use hex_slice::AsHex;
use actix_session::Session;
use actix_web::web;
use rusqlite::{params, named_params};
use crate::{util, config::Config, model, DB};
use crate::web_error::WebError;

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

  web::block(move || -> Result<model::User, WebError> {
    let db = db.get()?;

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
      return Err(WebError::InvalidLogin);
    }

    // query user
    let user = db.query_row("select * from `users` where `id` = :id", params![session.uid], |row| {
      Ok(model::User {
        id: row.get(0)?,
        login: row.get(1)?,
        password: row.get(2)?,
      })
    })?;

    Ok(user)
  }).await
    .map_err(|e| eprintln!("{:?}", e))
    .ok()
}

pub async fn login(login: &String, password: &String, db: DB, config: &Config, session: Session) -> Result<(), WebError> {
  // query user by login
  let user = web::block({
    let login = login.clone();
    let db = db.clone();
    move || {
      db.get()?.query_row("select * from `users` where `login` = :login", params![login], |row| {
        Ok(model::User {
          id: row.get(0)?,
          login: row.get(1)?,
          password: row.get(2)?,
        })
      }).map_err(|e| e.into(): WebError)
  }}).await.map_err(|_| WebError::InvalidLogin)?;

  // check password hash
  if user.password != password_hash(password, config) { return Err(WebError::InvalidLogin); }

  let timestamp = util::get_timestamp();

  // generate ssid
  let ssid = gen_ssid();
  let expires = timestamp + 2592000; // 1 month
  let ssid_cookie = ssid.clone();

  web::block({
    let db = db.clone();
    move || -> Result<(), WebError> {
      let mut db = db.get()?;
      let transaction = db.transaction()?;

      // TODO: export iface to cron
      // delete expired sessions
      transaction.execute_named(
        "delete from `sessions` where `uid` = :uid and `expires` < :timestamp",
        named_params![
        ":uid": user.id,
        ":timestamp": timestamp as i64
      ])?;

      transaction.execute_named(
        "insert into sessions (`id`, `uid`, `expires`) values (:id, :uid, :expires)",
        named_params![
        ":id": ssid,
        ":uid": user.id,
        ":expires": expires as i64
      ])?;

      transaction.commit()?;

      Ok(())
    }
  }).await?;

  // set session cookie
  session.set("ssid", ssid_cookie)
    .map_err(|e| WebError::InternalError {d: e.to_string()})?;

  Ok(())
}

pub async fn logout(db: DB, session: Session) -> Result<(), WebError> {
  let ssid = match session.get::<String>("ssid").ok().flatten() {
    Some(ssid) => ssid,
    None => return Err(WebError::InvalidLogin)
  };

  web::block(move || {
    db.get()?.execute(
      "delete from `sessions` where `id` = :ssid",
      params![ssid]
    ).map_err(|e| e.into(): WebError)
  }).await?;

  session.remove("ssid");
  Ok(())
}

pub async fn get_user(db: DB, session: &Session) -> Option<model::User>{
  // check session
  match session.get::<String>("ssid").ok().flatten() {
    Some(ssid) => check_session(&ssid, db).await,
    None => None
  }
}