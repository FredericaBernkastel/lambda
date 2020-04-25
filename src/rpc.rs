use std::error::Error;
use std::collections::HashMap;
use path_tree::PathTree;
use json::{JsonValue};
use serde::{Serialize, Deserialize};
use bytes::Bytes;
use actix_web::web;
use actix_session::Session;
use regex::Regex;
use crate::{util, config::Config, auth, model, DB, DBConn};

enum ErrorCode {
  Success = 0,
  InternalError = 100,
  InvalidLogin = 101,
  InvalidRequest = 102
}

#[derive(Debug)]
pub enum RpcError {
  InternalError { d: String },
  InvalidLogin,
  InvalidRequest
}

impl From<rusqlite::Error> for RpcError {
  fn from(error: rusqlite::Error) -> Self {
    return RpcError::InternalError { d: error.to_string() };
  }}
impl<T> From<actix_web::error::BlockingError<T>> for RpcError
  where T: std::fmt::Debug {
  fn from(error: actix_web::error::BlockingError<T>) -> Self {
    return RpcError::InternalError { d: error.to_string() };
  }}
impl From<std::io::Error> for RpcError {
  fn from(error: std::io::Error) -> Self {
    return RpcError::InternalError { d: error.to_string() };
  }}
impl From<&str> for RpcError {
  fn from(error: &str) -> Self {
    return RpcError::InternalError { d: error.to_string() };
  }}
impl From<image::error::ImageError> for RpcError {
  fn from(error: image::error::ImageError) -> Self {
    return RpcError::InternalError { d: error.to_string() };
  }}
impl From<std::str::Utf8Error> for RpcError {
  fn from(error: std::str::Utf8Error) -> Self {
    return RpcError::InternalError { d: error.to_string() };
  }}
impl From<json::Error> for RpcError {
  fn from(error: json::Error) -> Self {
    return RpcError::InternalError { d: error.to_string() };
  }}
impl From<base64::DecodeError> for RpcError {
  fn from(error: base64::DecodeError) -> Self {
    return RpcError::InternalError { d: error.to_string() };
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
        "/graffiti/edit",
        "/graffiti/delete",
        "/graffiti/store_image",
        "/author/add",
        "/author/edit",
        "/author/delete",
        "/author/store_image",
        "/search/author_names"
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
          "/graffiti/edit" => graffiti_edit(post_data, db).await?,
          "/graffiti/delete" => graffiti_delete(post_data, db).await?,
          "/graffiti/store_image" => store_image(post_data, db, vec![(480, 360), (100, 75)]).await,
          "/author/add" => author_add(post_data, db).await?,
          "/author/edit" => author_edit(post_data, db).await?,
          "/author/delete" => author_delete(post_data, db).await?,
          "/author/store_image" => store_image(post_data, db, vec![(170, 226), (56, 75)]).await,
          "/search/author_names" => search_author_names(post_data, db).await?,
          _ => unreachable!()
        }
      }
    },
    None => return Err("route not found".into())
  };
  Ok(res.dump())
}

fn images_ctr(
  images_folder: &str,
  old_images: Vec<String>,
  new_images: Vec<String>,
  db: &DBConn,
  sql_delete: &str,
  sql_insert: &str,
  foreign_id: u32
) -> Result<(), RpcError> {

  lazy_static! {
    static ref REG_SHA256: Regex = Regex::new(r"^[0-9a-f]{64}$").unwrap();
  }

  for image in new_images.iter() {
    if !REG_SHA256.is_match(image) {
      return Err(RpcError::InvalidRequest);
    }
  }

  // 1. delete the deleted images
  db.execute(sql_delete, params![foreign_id])?;

  for image in old_images.iter() {
    if !new_images.contains(image) {
      for p in vec![0, 1, 2].iter(){
        let path = format!("{}/{}/{}_p{}.jpg", images_folder, image.get(0..=1).ok_or("")?, image, p);
        std::fs::remove_file(path)?;
      }
    }
  }
  // 2. move new images from temp dir
  {
    let mut stmt_insert = db.prepare(sql_insert)?;
    let mut stmt_delete_tmp = db.prepare("delete from `tmp_store_image` where `id` = :hash")?;

    for (id, image) in new_images.iter().enumerate() {
      if !old_images.contains(image) {
        let prefix = image.get(0..=1).ok_or("")?;
        std::fs::create_dir_all(format!("{}/{}", images_folder, prefix))?;
        for p in vec![0, 1, 2].iter(){
          let path = format!("{}/{}/{}_p{}.jpg", images_folder, prefix, image, p);
          std::fs::rename(format!("data/tmp/{}_p{}.jpg", image, p), path)?;
        }
        stmt_delete_tmp.execute(params![image])?;
      }
      // 3. insert into database
      stmt_insert.execute_named(named_params![
          ":id": foreign_id,
          ":hash": image,
          ":order": id as u32,
        ])?;
    }
  }
  Ok(())
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
    Err(RpcError::InvalidRequest) => ErrorCode::InvalidRequest as u32,
    Err(RpcError::InternalError {d}) => {
      eprintln!("Error: {}", d);
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
  #[derive(Serialize, Deserialize)] struct Author {
    id: u32,
    indubitable: bool
  }
  #[derive(Serialize, Deserialize)] struct Request {
    graffiti: Graffiti,
    location: Location,
    authors: Vec<Author>,
    images: Vec<String>
  };
  let mut request: Request = serde_json::from_slice(post_data.as_ref())?;
  request.authors.sort_unstable_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
  request.authors.dedup_by(|a, b| a.id == b.id);

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

    // insert graffiti_author
    {
      let mut stmt = db.prepare("
        insert into graffiti_author (
          graffiti_id,
          author_id,
          indubitable
        )
        values (
          :graffiti_id,
          :author_id,
          :indubitable
        )")?;
      for author in request.authors.iter() {
        stmt.execute_named(named_params![
          ":graffiti_id": graffiti_id,
          ":author_id": author.id,
          ":indubitable": author.indubitable,
        ])?;
      }
    }

    images_ctr(
      "data/static/img/graffiti",
      vec![],
      request.images,
      &db,
      "delete from `graffiti_image` where `graffiti_id` = :id",
      "insert into `graffiti_image` (
        `graffiti_id`,
        `hash`,
        `order`
      ) values (
        :id,
        :hash,
        :order
      )",
      graffiti_id as u32
    )?;

    Ok(graffiti_id)
  }).await.map_err(|e| e.to_string())?;

  Ok(object!{
    result: ErrorCode::Success as u32,
    id: graffiti_id
  })
}

///graffiti/edit
async fn graffiti_edit(post_data: Bytes, db: DB) -> Result<JsonValue, Box<dyn Error>> {
  #[derive(Serialize, Deserialize)] struct Graffiti {
    id: u32,
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
  #[derive(Serialize, Deserialize)] struct Author {
    id: u32,
    indubitable: bool
  }
  #[derive(Serialize, Deserialize)] struct Request {
    graffiti: Graffiti,
    location: Location,
    authors: Vec<Author>,
    images: Vec<String>
  };
  let mut request: Request = serde_json::from_slice(post_data.as_ref())?;
  request.authors.sort_unstable_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
  request.authors.dedup_by(|a, b| a.id == b.id);

  web::block(move || -> Result<(), RpcError> {
    let db = db.get().unwrap();

    // update graffiti
    db.execute_named("
      update `graffiti`
        set `complaint_id` = :complaint_id,
            `datetime` = :datetime,
            `shift_time` = :shift_time,
            `intervening` = :intervening,
            `companions` = :companions,
            `notes` = :notes
        where `id` = :id",
      named_params![
        ":id":            request.graffiti.id,
        ":complaint_id":  request.graffiti.complaint_id,
        ":datetime":      request.graffiti.datetime,
        ":shift_time":    request.graffiti.shift_time,
        ":intervening":   request.graffiti.intervening,
        ":companions":    request.graffiti.companions,
        ":notes":         request.graffiti.notes,
    ])?;

    // update location
    db.execute_named("
      update `location`
        set `country` = :country,
            `city` = :city,
            `street` = :street,
            `place` = :place,
            `property` = :property,
            `gps_long` = :gps_long,
            `gps_lat` = :gps_lat
        where `graffiti_id` = :graffiti_id",
      named_params![
        ":graffiti_id": request.graffiti.id,
        ":country": request.location.country,
        ":city": request.location.city,
        ":street": request.location.street,
        ":place": request.location.place,
        ":property": request.location.property,
        ":gps_long": request.location.gps_long,
        ":gps_lat": request.location.gps_lat,
    ])?;

    // update graffiti_author
    {
      db.execute("delete from `graffiti_author` where `graffiti_id` = :id", params![request.graffiti.id])?;
      let mut stmt = db.prepare("
        insert into graffiti_author (
          graffiti_id,
          author_id,
          indubitable
        )
        values (
          :graffiti_id,
          :author_id,
          :indubitable
        )")?;
      for author in request.authors.iter() {
        stmt.execute_named(named_params![
          ":graffiti_id": request.graffiti.id,
          ":author_id": author.id,
          ":indubitable": author.indubitable,
        ])?;
      }
    }

    let old_images = db.prepare("
        select `hash` from `graffiti_image`
        where `graffiti_id` = :id
        order by `order` asc")?
    .query_map(params![request.graffiti.id], |row| {
      Ok(row.get::<_, String>(0)?)
    })?.filter_map(Result::ok).collect();

    images_ctr(
      "data/static/img/graffiti",
      old_images,
      request.images,
      &db,
      "delete from `graffiti_image` where `graffiti_id` = :id",
      "insert into `graffiti_image` (
        `graffiti_id`,
        `hash`,
        `order`
      ) values (
        :id,
        :hash,
        :order
      )",
      request.graffiti.id
    )?;

    Ok(())
  }).await.map_err(|e| e.to_string())?;

  Ok(object!{
    result: ErrorCode::Success as u32
  })
}

///graffiti/delete
async fn graffiti_delete(post_data: Bytes, db: DB) -> Result<JsonValue, Box<dyn Error>> {
  #[derive(Serialize, Deserialize)] struct Request {
    id: u32
  };
  let request: Request = serde_json::from_slice(post_data.as_ref())?;

  web::block(move || -> Result<(), RpcError> {
    let db = db.get().unwrap();
    // remove images
    {
      let images_folder = "data/static/img/graffiti";
      let mut stmt = db.prepare(
        "select `hash` from `graffiti_image` where `graffiti_id` = :graffiti_id")?;
      let images = stmt.query_map(params![request.id], |row| {
        Ok(row.get::<_, String>(0)?)
      })?.filter_map(Result::ok);
      for image in images {
        for p in vec![0, 1, 2].iter(){
          let path = format!("{}/{}/{}_p{}.jpg", images_folder, image.get(0..=1).ok_or("")?, image, p);
          std::fs::remove_file(path).ok();
        }
      }
    }

    db.execute("delete from `location` where `graffiti_id` = :id", params![request.id])?;
    db.execute("delete from `graffiti_image` where `graffiti_id` = :id", params![request.id])?;
    db.execute("delete from `graffiti_author` where `graffiti_id` = :id", params![request.id])?;
    db.execute("delete from `graffiti` where `id` = :id", params![request.id])?;
    Ok(())
  }).await.map_err(|e| e.to_string())?;

  Ok(object!{
    result: ErrorCode::Success as u32
  })
}

///author/add
async fn author_add(post_data: Bytes, db: DB) -> Result<JsonValue, Box<dyn Error>> {
  #[derive(Serialize, Deserialize)] struct Request {
    name: String,
    age: Option<u32>,
    height: Option<u32>,
    handedness: Option<u8>,
    home_city: String,
    social_networks: String,
    notes: String,
    images: Vec<String>
  };
  let request: Request = serde_json::from_slice(post_data.as_ref())?;

  if request.name.is_empty() {
    return Ok(object! {
      result: ErrorCode::InvalidRequest as u32
    })
  }

  let author_id = web::block(move || -> Result<i64, RpcError> {
    let db = db.get().unwrap();

    // insert author
    db.execute_named("
      insert into `author` (
        `name`,
        `age`,
        `height`,
        `handedness`,
        `home_city`,
        `social_networks`,
        `notes`
      ) values (
        :name,
        :age,
        :height,
        :handedness,
        :home_city,
        :social_networks,
        :notes
      )", named_params![
      ":name":            request.name,
      ":age":             request.age,
      ":height":          request.height,
      ":handedness":      request.handedness,
      ":home_city":       request.home_city,
      ":social_networks": request.social_networks,
      ":notes":           request.notes,
    ])?;
    let author_id = db.last_insert_rowid();

    images_ctr(
      "data/static/img/author",
      vec![],
      request.images,
      &db,
      "delete from `author_image` where `author_id` = :id",
      "insert into `author_image` (
        `author_id`,
        `hash`,
        `order`
      ) values (
        :id,
        :hash,
        :order
      )",
      author_id as u32
    )?;

    Ok(author_id)
  }).await.map_err(|e| e.to_string())?;

  Ok(object!{
    result: ErrorCode::Success as u32,
    id: author_id
  })
}

///author/edit
async fn author_edit(post_data: Bytes, db: DB) -> Result<JsonValue, Box<dyn Error>> {
  #[derive(Serialize, Deserialize)] struct Request {
    id: u32,
    name: String,
    age: Option<u32>,
    height: Option<u32>,
    handedness: Option<u8>,
    home_city: String,
    social_networks: String,
    notes: String,
    images: Vec<String>
  };
  let request: Request = serde_json::from_slice(post_data.as_ref())?;

  if request.name.is_empty() {
    return Ok(object! {
      result: ErrorCode::InvalidRequest as u32
    })
  }

  web::block(move || -> Result<(), RpcError> {
    let db = db.get().unwrap();

    // update graffiti
    db.execute_named("
      update `author`
        set `name` = :name,
            `age` = :age,
            `height` = :height,
            `handedness` = :handedness,
            `home_city` = :home_city,
            `social_networks` = :social_networks,
            `notes` = :notes
        where `id` = :id",
      named_params![
        ":id":              request.id,
        ":name":            request.name,
        ":age":             request.age,
        ":height":          request.height,
        ":handedness":      request.handedness,
        ":home_city":       request.home_city,
        ":social_networks": request.social_networks,
        ":notes":           request.notes,
    ])?;

    let old_images = db.prepare("
      select `hash` from `author_image`
      where `author_id` = :id
      order by `order` asc")?
    .query_map(params![request.id], |row| {
      Ok(row.get::<_, String>(0)?)
    })?.filter_map(Result::ok).collect();

    images_ctr(
      "data/static/img/author",
      old_images,
      request.images,
      &db,
      "delete from `author_image` where `author_id` = :id",
      "insert into `author_image` (
        `author_id`,
        `hash`,
        `order`
      ) values (
        :id,
        :hash,
        :order
      )",
      request.id
    )?;

    Ok(())
  }).await.map_err(|e| e.to_string())?;

  Ok(object!{
    result: ErrorCode::Success as u32
  })
}

///author/delete
async fn author_delete(post_data: Bytes, db: DB) -> Result<JsonValue, Box<dyn Error>> {
  #[derive(Serialize, Deserialize)] struct Request {
    id: u32
  };
  let request: Request = serde_json::from_slice(post_data.as_ref())?;

  web::block(move || -> Result<(), RpcError> {
    let db = db.get().unwrap();
    // remove images
    {
      let images_folder = "data/static/img/author";
      let mut stmt = db.prepare(
        "select `hash` from `author_image` where `author_id` = :author_id")?;
      let images = stmt.query_map(params![request.id], |row| {
        Ok(row.get::<_, String>(0)?)
      })?.filter_map(Result::ok);
      for image in images {
        for p in vec![0, 1, 2].iter(){
          let path = format!("{}/{}/{}_p{}.jpg", images_folder, image.get(0..=1).ok_or("")?, image, p);
          std::fs::remove_file(path).ok();
        }
      }
    }
    db.execute("delete from `author_image` where `author_id` = :id", params![request.id])?;
    db.execute("delete from `graffiti_author` where `author_id` = :id", params![request.id])?;
    db.execute("delete from `author` where `id` = :id", params![request.id])?;
    Ok(())
  }).await.map_err(|e| e.to_string())?;

  Ok(object!{
    result: ErrorCode::Success as u32
  })
}

///store_image
async fn store_image(post_data: Bytes, db: DB, sizes: Vec<(u32, u32)>) -> JsonValue {
  use image::{ImageFormat, imageops::FilterType};

  web::block( move || -> Result<JsonValue, RpcError> {
    let db = db.get().unwrap();

    let post_data = json::parse(std::str::from_utf8(&post_data)?)?;
    let image = {
      let image_b64 = post_data["data"].as_str().ok_or(RpcError::InvalidRequest)?;
      if image_b64.get(0..=22) != Some("data:image/jpeg;base64,") { return Err(RpcError::InvalidRequest); }
      image::load_from_memory_with_format(
        base64::decode( image_b64.get(23..).ok_or(RpcError::InvalidRequest)?)?.as_slice(),
        ImageFormat::Jpeg
      )?
    };

    let temp_id = auth::gen_ssid();

    // p0
    image.save(format!("data/tmp/{}_p0.jpg", temp_id))?;

    // generate thumbnails
    for (i, size) in sizes.iter().enumerate() {
      image
        .resize(size.0, size.1, FilterType::Lanczos3)
        .save(format!("data/tmp/{}_p{}.jpg", temp_id, i + 1))?;
    }

    db.execute("
      insert into `tmp_store_image`
        (`id`, `timestamp`)
        values(:id, :timestamp)", params![temp_id, util::get_timestamp() as i64])?;
    //garbage collector
    {
      let expired = util::get_timestamp() - 86400; // 1 day
      let mut stmt = db.prepare("select `id` from `tmp_store_image` where `timestamp` < :timestamp")?;
      let images = stmt.query_map(params![expired as i64], |row| {
        Ok(row.get::<_, String>(0)?)
      })?.filter_map(Result::ok);
      for image in images {
        for p in vec![0, 1, 2].iter() {
          std::fs::remove_file(format!("data/tmp/{}_p{}.jpg", image, p))?;
        }
      }
      db.execute("delete from `tmp_store_image` where `timestamp` < :timestamp", params![expired as i64])?;
    }
    Ok(object!{
      result: ErrorCode::Success as u32,
      temp_id: temp_id
    })
  }).await
    .unwrap_or_else(|e| {
      eprintln!("{:?}", e);
      object! {
        result: ErrorCode::InternalError as u32
      }
    })
}

///search/author_names
async fn search_author_names(post_data: Bytes, db: DB) -> Result<JsonValue, Box<dyn Error>> {
  #[derive(Deserialize)] struct Request {
    term: String
  };
  let request: Request = serde_json::from_slice(post_data.as_ref())?;
  struct Row {
    id: u32,
    name: String
  };
  let names = web::block(move || -> Result<Vec<Row>, RpcError> {
    let db = db.get().unwrap();
    let mut stmt = db.prepare("
      select id,
             name
        from author
       where name like :term
       limit 10")?;
    let term = format!("%{}%", request.term);
    let names: Vec<Row> = stmt.query_map(params![term], |row| {
      Ok(Row {
        id: row.get(0)?,
        name: row.get(1)?,
      })
    })?.filter_map(Result::ok).collect();
    Ok(names)
  }).await.map_err(|e| e.to_string())?;
  let names: Vec<JsonValue> = names.iter().map(|x| object! {
    id: x.id,
    name: x.name.clone()
  }).collect();
  Ok(object! {
    result: names
  })
}