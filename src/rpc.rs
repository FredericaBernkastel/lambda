use std::error::Error;
use std::collections::HashMap;
use path_tree::PathTree;
use json::{JsonValue};
use serde::Deserialize;
use bytes::Bytes;
use actix_session::Session;
use crate::{util, config::Config, auth, model, DB};

enum ErrorCode {
  Success = 0,
  InternalError = 100,
  InvalidLogin = 101
}

pub async fn main(uri: String, post_data: Bytes, db: DB, config: &Config, user: Option<model::User>, session: Session) -> Result<String, Box<dyn Error>> {

  // check cors hash
  {
    let post_data = json::parse(std::str::from_utf8(&post_data)?)?;
    if !util::check_cors_hash(post_data["cors_h"].as_str().ok_or("cors_h")?, config) {
      return Err("unauthorized".into());
    }
  }

  lazy_static! {
    static ref PATH_TREE: PathTree::<&'static str> = {
      let mut tmp = PathTree::<&str>::new();
      for path in vec![
        "/auth/login",
        "/auth/logout",
      ] { tmp.insert(path, path); };
      tmp
    };
  };
  let res = match PATH_TREE.find(uri.as_str()) {
    Some((path, get_data)) => {
      let path = *path;
      let _get_data: HashMap<_, _> = get_data.into_iter().collect();
      if path == "/auth/login" {
        if user.is_some() { return Err("invalid request".into()) }
        auth_login(post_data, db, config, session).await?
      } else {
        let _user = match user {
          Some(user) => user,
          None => return Err("unauthorized".into())
        };

        match path {
          "/auth/logout" => auth_logout(db, session).await?,
          _ => unreachable!()
        }
      }
    },
    None => return Err("route not found".into())
  };
  Ok(res.dump())
}

///auth/login
async fn auth_login(post_data: Bytes, db: DB, config: &Config, session: Session) -> Result<JsonValue, Box<dyn Error>> {
  #[derive(Deserialize)] struct Request {
    login: String,
    password: String
  };
  let request: Request = serde_json::from_slice(post_data.as_ref())?;
  let result = match auth::login(&request.login, &request.password, db, config, session).await {
    Ok(_) => ErrorCode::Success as u32,
    Err(auth::Error::InvalidLogin) => ErrorCode::InvalidLogin as u32,
    Err(auth::Error::InternalError {description}) => {
      eprintln!("Error: {}", description);
      ErrorCode::InternalError as u32
    }
  };
  Ok(object!{
    result: result
  })
}

///auth/logout
async fn auth_logout(db: DB, session: Session) -> Result<JsonValue, Box<dyn Error>> {
  let result = auth::logout(db, session)
    .await
    .map(|_| ErrorCode::Success as u32)
    .map_err(|e| format!("{:?}", e))?;
  Ok(object!{
    result: result
  })
}