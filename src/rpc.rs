use std::error::Error;
use std::collections::HashMap;
use path_tree::PathTree;
use json::{JsonValue};
use serde::{Serialize, Deserialize};
use bytes::Bytes;
use actix_web::web;
use actix_session::Session;
use crate::{util, config::Config, auth, model, DB};

enum ErrorCode {
  Success = 0,
  InternalError = 100,
  InvalidLogin = 101
}

#[derive(Debug)]
pub enum RpcError {
  InternalError { description: String },
  InvalidLogin
}
impl From<rusqlite::Error> for RpcError {
  fn from(error: rusqlite::Error) -> Self {
    return RpcError::InternalError { description: error.to_string() };
  }}
impl<T> From<actix_web::error::BlockingError<T>> for RpcError
  where T: std::fmt::Debug {
  fn from(error: actix_web::error::BlockingError<T>) -> Self {
    return RpcError::InternalError { description: error.to_string() };
  }}

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
        "/graffiti/add",
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
          "/graffiti/add" => graffiti_add(post_data, db).await?,
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
    Err(RpcError::InvalidLogin) => ErrorCode::InvalidLogin as u32,
    Err(RpcError::InternalError {description}) => {
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

///graffiti/add
async fn graffiti_add(post_data: Bytes, db: DB) -> Result<JsonValue, Box<dyn Error>> {
  #[derive(Serialize, Deserialize)] struct Graffiti {
    complaint_id: String,
    datetime: Option<i64>,
    shift_time: u8,
    intervening: String,
    companions: u32,
    notes: String,
  };
  #[derive(Serialize, Deserialize)] struct Location {
    country: String,
    city: String,
    street: String,
    place: String,
    property: String,
    gps_long: Option<f64>,
    gps_lat: Option<f64>
  };
  #[derive(Serialize, Deserialize)] struct Request {
    graffiti: Graffiti,
    location: Location
  };
  let request: Request = serde_json::from_slice(post_data.as_ref())?;

  let graffiti_id = web::block(move || -> Result<i64, RpcError> {
    let db = db.get().unwrap();

    // insert graffiti
    db.execute_named("
      insert into `graffiti` (
        `complaint_id`,
        `datetime`,
        `shift_time`,
        `intervening`,
        `companions`,
        `notes`
      ) values (
        :complaint_id,
        :datetime,
        :shift_time,
        :intervening,
        :companions,
        :notes
      )", named_params![
      ":complaint_id":  request.graffiti.complaint_id,
      ":datetime":      request.graffiti.datetime,
      ":shift_time":    request.graffiti.shift_time,
      ":intervening":   request.graffiti.intervening,
      ":companions":    request.graffiti.companions,
      ":notes":         request.graffiti.notes,
    ])?;
    let graffiti_id = db.last_insert_rowid();

    // insert location
    db.execute_named("
      insert into `location` (
        `graffiti_id`,
        `country`,
        `city`,
        `street`,
        `place`,
        `property`,
        `gps_long`,
        `gps_lat`
      ) values (
        :graffiti_id,
        :country,
        :city,
        :street,
        :place,
        :property,
        :gps_long,
        :gps_lat
      )", named_params![
      ":graffiti_id": graffiti_id,
      ":country": request.location.country,
      ":city": request.location.city,
      ":street": request.location.street,
      ":place": request.location.place,
      ":property": request.location.property,
      ":gps_long": request.location.gps_long,
      ":gps_lat": request.location.gps_lat,
    ])?;

    Ok(graffiti_id)
  }).await.map_err(|e| e.to_string())?;

  Ok(object!{
    result: ErrorCode::Success as u32,
    id: graffiti_id
  })
}