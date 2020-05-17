use crate::{
  config::Config,
  error::{ErrorKind, Result},
  schema, util,
  web::{self, DB},
};
use actix_session::Session;
use error_chain::bail;
use hex_slice::AsHex;
use rusqlite::{named_params, params, OptionalExtension};
use sha2::{Digest, Sha256};

pub fn password_hash(password: &str, config: &Config) -> String {
  format!(
    "{:02x}",
    Sha256::digest(format!("{}{}", password, config.web.secret_key).as_bytes())
  )
}

pub fn gen_ssid() -> String {
  use rand::prelude::*;

  let mut data = [0u8; 32];
  rand::thread_rng().fill_bytes(&mut data);
  format!("{:02x}", data.plain_hex(false))
}

pub async fn check_session(ssid: String, db_pool: DB) -> Result<Option<schema::User>> {
  //let ssid = ssid.clone();
  if ssid.len() != 64 {
    bail!(ErrorKind::InvalidRequest)
  }

  let res = web::block(db_pool, move |db| -> Result<Option<schema::User>> {
    // query session
    let session = match db
      .query_row("select * from `sessions` where `id` = :ssid", params![ssid], |row| {
        Ok(schema::Session {
          id: row.get(0)?,
          uid: row.get(1)?,
          expires: row.get::<_, i64>(2)? as u64,
        })
      })
      .optional()?
    {
      Some(session) => session,
      None => return Ok(None),
    }; // invalid session

    // check for expired
    if session.expires < util::get_timestamp() {
      return Ok(None);
    }

    // query user
    let user = db.query_row("select * from `users` where `id` = :id", params![session.uid], |row| {
      Ok(schema::User {
        id: row.get(0)?,
        login: row.get(1)?,
        password: row.get(2)?,
      })
    })?;

    Ok(Some(user))
  })
  .await?;
  Ok(res)
}

pub async fn login(login: &str, password: &str, db_pool: DB, config: &Config, session: Session) -> Result<()> {
  // query user by login
  let user = web::block(db_pool.clone(), {
    let login = login.to_string();
    move |db| -> Result<_> {
      Ok(
        db.query_row("select * from `users` where `login` = :login", params![login], |row| {
          Ok(schema::User {
            id: row.get(0)?,
            login: row.get(1)?,
            password: row.get(2)?,
          })
        })?,
      )
    }
  })
  .await
  .map_err(|_| ErrorKind::InvalidLogin)?;

  // check password hash
  if user.password != password_hash(password, config) {
    bail!(ErrorKind::InvalidLogin);
  }

  let timestamp = util::get_timestamp();

  // generate ssid
  let ssid = gen_ssid();
  let expires = timestamp + 2592000; // 1 month

  web::block(db_pool, {
    let ssid = ssid.clone();
    move |mut _db| -> Result<_> {
      let transaction = _db.transaction()?;

      // TODO: export iface to cron
      // delete expired sessions
      transaction.execute_named(
        "delete from `sessions` where `uid` = :uid and `expires` < :timestamp",
        named_params![
          ":uid": user.id,
          ":timestamp": timestamp as i64
        ],
      )?;

      transaction.execute_named(
        "insert into sessions (`id`, `uid`, `expires`) values (:id, :uid, :expires)",
        named_params![
          ":id": ssid,
          ":uid": user.id,
          ":expires": expires as i64
        ],
      )?;

      transaction.commit()?;

      Ok(())
    }
  })
  .await?;

  // set session cookie
  session.set("ssid", ssid).map_err(|_| "unable to set cookie")?;

  Ok(())
}

pub async fn logout(db_pool: DB, session: Session) -> Result<()> {
  let ssid = session
    .get::<String>("ssid")
    .ok()
    .flatten()
    .ok_or(ErrorKind::InvalidLogin)?;

  web::block(db_pool, move |db| -> Result<_> {
    db.execute("delete from `sessions` where `id` = :ssid", params![ssid])?;
    Ok(())
  })
  .await?;

  session.remove("ssid");
  Ok(())
}

pub async fn get_user(db_pool: DB, session: &Session) -> Result<Option<schema::User>> {
  // check session
  match session.get::<String>("ssid").ok().flatten() {
    Some(ssid) => check_session(ssid, db_pool).await,
    None => Ok(None),
  }
}
