use crate::{
  error::{Error, ErrorKind::{self, NoneError}, Result},
  schema, util,
  util::json_path,
  web::{self, auth, Config, DB},
  log_error
};
use actix_session::Session;
use error_chain::bail;
use lazy_static::lazy_static;
use path_tree::PathTree;
use regex::Regex;
use rusqlite::{named_params, params, Transaction, NO_PARAMS};
use serde::Deserialize;
use serde_json::{from_value as from_json, json, Value as JsonValue};

#[repr(u8)]
enum Opcode {
  Success = 0,
  //InternalError = 100,
  InvalidLogin = 101,
  InvalidRequest = 102,
}

pub async fn main(
  uri: String,
  mut post_data: JsonValue,
  db_pool: DB,
  config: Config,
  user: Option<schema::User>,
  session: Session,
) -> Result<String> {
  // check cors hash
  {
    if !util::check_cors_hash(
      json_path::<String>(&mut post_data, "/cors_h")?.as_str(),
      &config,
    ) {
      bail!("unauthorized");
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
        "/search/author_names",
        "/search/tag_names",
        "/search/locations",
        "/tags/edit",
      ] {
        tmp.insert(path, path);
      }
      tmp
    };
  };

  let ctr = Controller {
    db_pool,
    config,
    post_data,
  };

  let res = match PATH_TREE.find(uri.as_str()) {
    Some((path, _get_data)) => {
      let path = *path;
      //let get_data: HashMap<_, _> = get_data.into_iter().collect();
      match path {
        "/auth/login" => match user {
          Some(_) => bail!(ErrorKind::InvalidRequest),
          None => ctr.auth_login(session).await?,
        },
        _ => {
          let _user = match user {
            Some(user) => user,
            None => bail!("unauthorized"),
          };

          match path {
            "/auth/logout" => ctr.auth_logout(session).await?,
            "/graffiti/add" => ctr.graffiti_add().await?,
            "/graffiti/edit" => ctr.graffiti_edit().await?,
            "/graffiti/delete" => ctr.graffiti_delete().await?,
            "/graffiti/store_image" => ctr.store_image(vec![(480, 360), (100, 75)]).await?,
            "/author/add" => ctr.author_add().await?,
            "/author/edit" => ctr.author_edit().await?,
            "/author/delete" => ctr.author_delete().await?,
            "/author/store_image" => ctr.store_image(vec![(170, 226), (56, 75)]).await?,
            "/search/author_names" => ctr.search_author_names().await?,
            "/search/tag_names" => ctr.search_tag_names().await?,
            "/search/locations" => ctr.search_locations().await?,
            "/tags/edit" => ctr.tags_edit().await?,
            _ => unreachable!(),
          }
        }
      }
    }
    None => bail!("route not found"),
  };
  Ok(res.to_string())
}

struct Controller {
  db_pool: DB,
  config: Config,
  post_data: JsonValue,
}

impl Controller {
  ///auth/login
  async fn auth_login(mut self, session: Session) -> Result<JsonValue> {
    #[derive(Deserialize)]
    struct Request {
      login: String,
      password: String,
    }
    let request: Request = from_json(self.post_data.take())?;
    let result = match auth::login(
      &request.login,
      &request.password,
      self.db_pool,
      &self.config,
      session,
    )
    .await
    {
      Ok(_) => Opcode::Success,
      Err(Error(ErrorKind::InvalidLogin, _)) => Opcode::InvalidLogin,
      Err(e) => bail!(e),
    };
    Ok(json!({ "result": result as u8 }))
  }

  ///auth/logout
  async fn auth_logout(self, session: Session) -> Result<JsonValue> {
    let result = auth::logout(self.db_pool, session).await;
    let result = match result {
      Ok(_) => Opcode::Success,
      Err(Error(ErrorKind::InvalidLogin, _)) => Opcode::InvalidLogin,
      Err(e) => bail!(e),
    };
    Ok(json!({ "result": result as u8 }))
  }

  ///graffiti/add
  async fn graffiti_add(mut self) -> Result<JsonValue> {
    #[derive(Deserialize)]
    struct Graffiti {
      complaint_id: String,
      datetime: Option<i64>,
      shift_time: u8,
      intervening: String,
      companions: u32,
      notes: String,
    }
    #[derive(Deserialize)]
    struct Location {
      country: String,
      city: String,
      street: String,
      place: String,
      property: String,
      gps_long: Option<f64>,
      gps_lat: Option<f64>,
    }
    #[derive(Deserialize)]
    struct Author {
      id: u32,
      indubitable: bool,
    }
    #[derive(Deserialize)]
    struct Request {
      graffiti: Graffiti,
      location: Location,
      authors: Vec<Author>,
      images: Vec<String>,
      tags: Vec<String>,
    }
    let mut request: Request = from_json(self.post_data.take())?;
    request.authors.sort_unstable_by(|a, b| a.id.cmp(&b.id));
    request.authors.dedup_by(|a, b| a.id == b.id);

    let graffiti_id = web::block(self.db_pool, |mut _db| -> Result<_> {
      let transaction = _db.transaction()?;

      // insert graffiti
      transaction.execute_named(
        "insert into `graffiti` (
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
        )",
        named_params![
          ":complaint_id":  request.graffiti.complaint_id,
          ":datetime":      request.graffiti.datetime,
          ":shift_time":    request.graffiti.shift_time,
          ":intervening":   request.graffiti.intervening,
          ":companions":    request.graffiti.companions,
          ":notes":         request.graffiti.notes,
        ],
      )?;
      let graffiti_id = transaction.last_insert_rowid();

      // insert location
      transaction.execute_named(
        "insert into `location` (
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
        )",
        named_params![
          ":graffiti_id": graffiti_id,
          ":country": request.location.country,
          ":city": request.location.city,
          ":street": request.location.street,
          ":place": request.location.place,
          ":property": request.location.property,
          ":gps_long": request.location.gps_long,
          ":gps_lat": request.location.gps_lat,
        ],
      )?;

      // insert graffiti_author
      {
        let mut stmt = transaction.prepare(
          "insert into graffiti_author (
            graffiti_id,
            author_id,
            indubitable
          )
          values (
            :graffiti_id,
            :author_id,
            :indubitable
          )",
        )?;
        for author in request.authors {
          stmt.execute_named(named_params![
            ":graffiti_id": graffiti_id,
            ":author_id": author.id,
            ":indubitable": author.indubitable,
          ])?;
        }
      }

      // insert graffiti tags
      {
        let mut stmt = transaction.prepare("insert into tag (name) values (:name)")?;
        for tag in &request.tags {
          stmt.execute(params![tag])?;
        }

        let mut stmt = transaction.prepare(
          "insert into graffiti_tag (
            graffiti_id,
            tag_id
          )
          values (
            :graffiti_id,
            (
             select id
               from tag
              where name = :tag_name
            )
          )",
        )?;
        for tag in request.tags {
          stmt.execute_named(named_params![":graffiti_id": graffiti_id, ":tag_name": tag])?;
        }
      }

      Self::images_ctr(
        "data/static/img/graffiti",
        vec![],
        request.images.into_iter().map(|x| x.to_owned()).collect(),
        &transaction,
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
        graffiti_id as u32,
      )?;

      transaction.commit()?;

      Ok(graffiti_id)
    })
    .await?;

    Ok(json!({
      "result": Opcode::Success as u8,
      "id": graffiti_id
    }))
  }

  ///graffiti/edit
  async fn graffiti_edit(mut self) -> Result<JsonValue> {
    #[derive(Deserialize)]
    struct Graffiti {
      id: u32,
      complaint_id: String,
      datetime: Option<i64>,
      shift_time: u8,
      intervening: String,
      companions: u32,
      notes: String,
    }
    #[derive(Deserialize)]
    struct Location {
      country: String,
      city: String,
      street: String,
      place: String,
      property: String,
      gps_long: Option<f64>,
      gps_lat: Option<f64>,
    }
    #[derive(Deserialize)]
    struct Author {
      id: u32,
      indubitable: bool,
    }
    #[derive(Deserialize)]
    struct Request {
      graffiti: Graffiti,
      location: Location,
      authors: Vec<Author>,
      images: Vec<String>,
      tags: Vec<String>,
    }
    let mut request: Request = from_json(self.post_data.take())?;
    request.authors.sort_unstable_by(|a, b| a.id.cmp(&b.id));
    request.authors.dedup_by(|a, b| a.id == b.id);

    web::block(self.db_pool, |mut _db| -> Result<_> {
      let transaction = _db.transaction()?;

      // update graffiti
      transaction.execute_named(
        "update `graffiti`
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
        ],
      )?;

      // update location
      transaction.execute_named(
        "update `location`
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
        ],
      )?;

      // update graffiti_author
      {
        transaction.execute(
          "delete from `graffiti_author` where `graffiti_id` = :id",
          params![request.graffiti.id],
        )?;
        let mut stmt = transaction.prepare(
          "insert into graffiti_author (
            graffiti_id,
            author_id,
            indubitable
          )
          values (
            :graffiti_id,
            :author_id,
            :indubitable
          )",
        )?;
        for author in request.authors {
          stmt.execute_named(named_params![
            ":graffiti_id": request.graffiti.id,
            ":author_id": author.id,
            ":indubitable": author.indubitable,
          ])?;
        }
      }

      // update graffiti tags
      {
        let mut stmt = transaction.prepare("insert into tag (name) values (:name)")?;
        for tag in &request.tags {
          stmt.execute(params![tag])?;
        }
        transaction.execute(
          "delete from `graffiti_tag` where `graffiti_id` = :id",
          params![request.graffiti.id],
        )?;

        let mut stmt = transaction.prepare(
          "insert into graffiti_tag (
            graffiti_id,
            tag_id
          )
          values (
            :graffiti_id,
            (
             select id
               from tag
              where name = :tag_name
            )
          )",
        )?;
        for tag in request.tags {
          stmt.execute_named(named_params![
            ":graffiti_id": request.graffiti.id,
            ":tag_name": tag
          ])?;
        }
        // GC
        transaction.execute("delete from `tag` where `count` = 0", NO_PARAMS)?;
      }

      let old_images = transaction
        .prepare(
          "select `hash` from `graffiti_image`
          where `graffiti_id` = :id
          order by `order` asc",
        )?
        .query_map(params![request.graffiti.id], |row| {
          Ok(row.get::<_, String>(0)?)
        })?
        .filter_map(std::result::Result::ok)
        .collect();

      Self::images_ctr(
        "data/static/img/graffiti",
        old_images,
        request.images.into_iter().map(|x| x.to_owned()).collect(),
        &transaction,
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
        request.graffiti.id,
      )?;

      transaction.commit()?;

      Ok(())
    })
    .await?;

    Ok(json!({ "result": Opcode::Success as u8 }))
  }

  ///graffiti/delete
  async fn graffiti_delete(mut self) -> Result<JsonValue> {
    #[derive(Deserialize)]
    struct Request {
      id: u32,
    }
    let request: Request = from_json(self.post_data.take())?;

    web::block(self.db_pool, move |mut _db| -> Result<_> {
      let transaction = _db.transaction()?;
      // remove images
      {
        let images_folder = "data/static/img/graffiti";
        let mut stmt = transaction.prepare(
          "select hash
            from graffiti_image
           where graffiti_id = :graffiti_id",
        )?;
        let images = stmt
          .query_map(params![request.id], |row| Ok(row.get::<_, String>(0)?))?
          .filter_map(std::result::Result::ok);
        for image in images {
          for p in 0..=2 {
            let path = format!("{}/{}/{}_p{}.jpg", images_folder, image.get(0..=1).ok_or(NoneError)?, image, p);
            std::fs::remove_file(&path)
              .map_err(|e| log_error!(e, format!("graffiti_delete\nUnable to delete file: {}", path)))
              .ok();
          }
        }
      }

      #[rustfmt::skip] {
        transaction.execute("delete from `location` where `graffiti_id` = :id", params![request.id])?;
        transaction.execute("delete from `graffiti_image` where `graffiti_id` = :id", params![request.id])?;
        transaction.execute("delete from `graffiti_author` where `graffiti_id` = :id", params![request.id])?;
        transaction.execute("delete from `graffiti_tag` where `graffiti_id` = :id", params![request.id])?;
        transaction.execute("delete from `tag` where `count` = 0", NO_PARAMS)?; // GC
        transaction.execute("delete from `graffiti` where `id` = :id", params![request.id])?;
      }

      transaction.commit()?;
      Ok(())
    })
    .await?;

    Ok(json!({ "result": Opcode::Success as u8 }))
  }

  ///author/add
  async fn author_add(mut self) -> Result<JsonValue> {
    #[derive(Deserialize)]
    struct Request {
      name: String,
      age: Option<u32>,
      height: Option<u32>,
      handedness: Option<u8>,
      home_city: String,
      social_networks: String,
      notes: String,
      images: Vec<String>,
    }
    let request: Request = from_json(self.post_data.take())?;

    if request.name.is_empty() {
      return Ok(json!({ "result": Opcode::InvalidRequest as u8 }));
    }

    let author_id = web::block(self.db_pool, |mut _db| -> Result<_> {
      let transaction = _db.transaction()?;

      // insert author
      transaction.execute_named(
        "insert into `author` (
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
        )",
        named_params![
          ":name":            request.name,
          ":age":             request.age,
          ":height":          request.height,
          ":handedness":      request.handedness,
          ":home_city":       request.home_city,
          ":social_networks": request.social_networks,
          ":notes":           request.notes,
        ],
      )?;
      let author_id = transaction.last_insert_rowid();

      Self::images_ctr(
        "data/static/img/author",
        vec![],
        request.images.into_iter().map(|x| x.to_owned()).collect(),
        &transaction,
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
        author_id as u32,
      )?;

      transaction.commit()?;

      Ok(author_id)
    })
    .await?;

    Ok(json!({
      "result": Opcode::Success as u8,
      "id": author_id
    }))
  }

  ///author/edit
  async fn author_edit(mut self) -> Result<JsonValue> {
    #[derive(Deserialize)]
    struct Request {
      id: u32,
      name: String,
      age: Option<u32>,
      height: Option<u32>,
      handedness: Option<u8>,
      home_city: String,
      social_networks: String,
      notes: String,
      images: Vec<String>,
    }
    let request: Request = from_json(self.post_data.take())?;

    if request.name.is_empty() {
      return Ok(json!({ "result": Opcode::InvalidRequest as u8 }));
    }

    web::block(self.db_pool, |mut _db| -> Result<_> {
      let transaction = _db.transaction()?;

      // update graffiti
      transaction.execute_named(
        "update `author`
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
        ],
      )?;

      let old_images = transaction
        .prepare(
          "select `hash` from `author_image`
          where `author_id` = :id
          order by `order` asc",
        )?
        .query_map(params![request.id], |row| Ok(row.get::<_, String>(0)?))?
        .filter_map(std::result::Result::ok)
        .collect();

      Self::images_ctr(
        "data/static/img/author",
        old_images,
        request.images.into_iter().map(|x| x.to_owned()).collect(),
        &transaction,
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
        request.id,
      )?;

      transaction.commit()?;

      Ok(())
    })
    .await?;

    Ok(json!({ "result": Opcode::Success as u8 }))
  }

  ///author/delete
  async fn author_delete(mut self) -> Result<JsonValue> {
    #[derive(Deserialize)]
    struct Request {
      id: u32,
    }
    let request: Request = from_json(self.post_data.take())?;

    web::block(self.db_pool, move |mut _db| -> Result<_> {
      let transaction = _db.transaction()?;
      // remove images
      {
        let images_folder = "data/static/img/author";
        let mut stmt = transaction.prepare("select `hash` from `author_image` where `author_id` = :author_id")?;
        let images = stmt
          .query_map(params![request.id], |row| Ok(row.get::<_, String>(0)?))?
          .filter_map(std::result::Result::ok);
        for image in images {
          for p in 0..=2 {
            let path = format!("{}/{}/{}_p{}.jpg", images_folder, image.get(0..=1).ok_or(NoneError)?, image, p);
            std::fs::remove_file(&path)
              .map_err(|e| log_error!(e, format!("author_delete\nUnable to delete file: {}", path)))
              .ok();
          }
        }
      }
      #[rustfmt::skip] {
        transaction.execute("delete from `author_image` where `author_id` = :id", params![request.id])?;
        transaction.execute("delete from `graffiti_author` where `author_id` = :id", params![request.id])?;
        transaction.execute("delete from `author` where `id` = :id", params![request.id])?;
      }

      transaction.commit()?;
      Ok(())
    })
    .await?;

    Ok(json!({ "result": Opcode::Success as u8 }))
  }

  ///store_image
  async fn store_image(mut self, sizes: Vec<(u32, u32)>) -> Result<JsonValue> {
    use image::{imageops::FilterType, ImageFormat};

    let temp_id = web::block(self.db_pool.clone(), move |mut _db| -> Result<_> {
      let transaction = _db.transaction()?;

      let image = {
        let image_b64 = json_path::<String>(&mut self.post_data, "/data")?;
        if image_b64.get(0..=22) != Some("data:image/jpeg;base64,") {
          bail!(ErrorKind::InvalidRequest);
        }
        image::load_from_memory_with_format(
          base64::decode(image_b64.get(23..).ok_or(ErrorKind::InvalidRequest)?)?.as_slice(),
          ImageFormat::Jpeg,
        )?
      };

      let temp_id = auth::gen_ssid();

      // p0
      image.save(format!("data/tmp/{}_p0.jpg", temp_id))?;

      // generate thumbnails
      for (i, (width, height)) in sizes.into_iter().enumerate() {
        image
          .resize(width, height, FilterType::Lanczos3)
          .save(format!("data/tmp/{}_p{}.jpg", temp_id, i + 1))?;
      }

      transaction.execute(
        "insert into tmp_store_image (
          id,
          timestamp
        ) values (
          :id,
          :timestamp
        )",
        params![temp_id, util::get_timestamp() as i64],
      )?;
      //garbage collector
      {
        let expired = util::get_timestamp() - 86400; // 1 day
        let images = transaction
          .prepare("select `id` from `tmp_store_image` where `timestamp` < :timestamp")?
          .query_map(params![expired as i64], |row| Ok(row.get(0)?))?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<String>;
        for image in images {
          for p in 0..=2 {
            let path = format!("data/tmp/{}_p{}.jpg", image, p);
            std::fs::remove_file(&path)
              .map_err(|e| log_error!(e, format!("store_image::gc\nUnable to delete file: {}", path)))
              .ok();
          }
        }
        transaction.execute(
          "delete from `tmp_store_image` where `timestamp` < :timestamp",
          params![expired as i64],
        )?;

        transaction.commit()?;
      }
      Ok(temp_id)
    })
    .await?;

    Ok(json!({
      "result": Opcode::Success as u8,
      "temp_id": temp_id
    }))
  }

  ///search/author_names
  async fn search_author_names(mut self) -> Result<JsonValue> {
    #[derive(Deserialize)]
    struct Request {
      term: String,
    }
    let request: Request = from_json(self.post_data.take())?;
    struct Row {
      id: u32,
      name: String,
    }
    let names = web::block(self.db_pool, move |db| -> Result<_> {
      let mut stmt = db.prepare(
        "select id,
                name
           from author
          where name like :term
          limit 10",
      )?;
      let term = format!("%{}%", request.term);
      let names: Vec<Row> = stmt
        .query_map(params![term], |row| {
          Ok(Row {
            id: row.get(0)?,
            name: row.get(1)?,
          })
        })?
        .filter_map(std::result::Result::ok)
        .collect();
      Ok(names)
    })
    .await?;

    let names: Vec<JsonValue> = names
      .into_iter()
      .map(|x| {
        json!({
          "id": x.id,
          "name": x.name
        })
      })
      .collect();

    Ok(json!({ "result": names }))
  }

  ///search/tag_names
  async fn search_tag_names(mut self) -> Result<JsonValue> {
    #[derive(Deserialize)]
    struct Request {
      term: String,
    }
    let request: Request = from_json(self.post_data.take())?;
    struct Row {
      id: u32,
      name: String,
    }
    let tags = web::block(self.db_pool, move |db| -> Result<_> {
      let mut stmt = db.prepare(
        "select id, name
           from tag
          where name like :term
          limit 10",
      )?;
      let term = format!("%{}%", request.term);
      let tags: Vec<Row> = stmt
        .query_map(params![term], |row| {
          Ok(Row {
            id: row.get(0)?,
            name: row.get(1)?,
          })
        })?
        .filter_map(std::result::Result::ok)
        .collect();
      Ok(tags)
    })
    .await?;

    let tags: Vec<JsonValue> = tags
      .into_iter()
      .map(|x| {
        json!({
          "id": x.id,
          "name": x.name
        })
      })
      .collect();

    Ok(json!({ "result": tags }))
  }

  ///search/locations
  async fn search_locations(mut self) -> Result<JsonValue> {
    #[derive(Deserialize)]
    struct Request {
      term: String,
    }
    let request: Request = from_json(self.post_data.take())?;
    let names = web::block(self.db_pool, move |db| -> Result<_> {
      let mut stmt = db.prepare(
        "select country,
               city,
               street
          from location
         where country like :term or 
               city like :term or 
               street like :term
          limit 10",
      )?;

      let term = format!("%{}%", request.term);
      let mut names: Vec<String> = stmt
        .query_map(params![term], |row| {
          Ok(vec![row.get(0)?, row.get(1)?, row.get(2)?])
        })?
        .filter_map(std::result::Result::ok)
        .flatten()
        .filter(|x: &String| x.to_lowercase().contains(&request.term.to_lowercase()))
        .collect();
      names.sort_unstable();
      names.dedup();
      Ok(names)
    })
    .await?;

    let names: Vec<JsonValue> = names.into_iter().map(|x| json!({ "name": x })).collect();
    Ok(json!({ "result": names }))
  }

  ///tags_edit
  async fn tags_edit(mut self) -> Result<JsonValue> {
    #[repr(u8)]
    #[derive(serde_repr::Deserialize_repr)]
    enum TagOpcode {
      Delete = 1,
      Rename = 2,
      Merge = 3,
    }
    #[derive(Deserialize)]
    struct Request {
      opcode: TagOpcode,
      lhside: String,
      rhside: String,
    }
    let request: Request = from_json(self.post_data.take())?;
    web::block(self.db_pool, move |mut _db| -> Result<_> {
      let transaction = _db.transaction()?;

      match request.opcode {
        TagOpcode::Delete => {
          transaction
            .prepare(
              "delete from graffiti_tag
              where tag_id = (
                               select id
                                 from tag
                                where name = :tag
                             )",
            )?
            .execute(params![request.lhside])?;
          transaction
            .prepare("delete from tag where name = :tag")?
            .execute(params![request.lhside])?;
        }
        TagOpcode::Rename => {
          transaction
            .prepare(
              "update tag
             set name = :rhside
           where name = :lhside",
            )?
            .execute_named(named_params![
              ":lhside": request.lhside,
              ":rhside": request.rhside
            ])?;
        }
        TagOpcode::Merge => {
          transaction
            .prepare(
              // the (graffiti_id, tag_id) unique contraint will ensure that the set is distinct
              "update graffiti_tag
               set tag_id = (
                     select id
                       from tag
                      where name = :rhside
                   )
             where tag_id = (
                     select id
                       from tag
                      where name = :lhside
                   )",
            )?
            .execute_named(named_params![
              ":lhside": request.lhside,
              ":rhside": request.rhside,
            ])?;
          transaction
            .prepare("delete from tag where name = :lhside")?
            .execute(params![request.lhside])?;
          transaction
            .prepare(
              "update tag
               set count = (
                 select count( * ) 
                   from graffiti_tag
                  where tag_id = (
                     select id
                       from tag
                      where name = :rhside
                   )
               )
             where id = (
                select id
                  from tag
                 where name = :rhside
              )",
            )?
            .execute_named(named_params![":rhside": request.rhside])?;
        }
      }

      transaction.commit()?;
      Ok(())
    })
    .await?;
    Ok(json!({ "result": Opcode::Success as u8 }))
  }

  fn images_ctr(
    images_folder: &str,
    old_images: Vec<String>,
    new_images: Vec<String>,
    transaction: &Transaction,
    sql_delete: &str,
    sql_insert: &str,
    foreign_id: u32,
  ) -> Result<()> {
    lazy_static! {
      static ref REG_SHA256: Regex = Regex::new(r"^[0-9a-f]{64}$").unwrap();
    }

    for image in &new_images {
      if !REG_SHA256.is_match(image) {
        bail!(ErrorKind::InvalidRequest);
      }
    }

    // 1. delete the deleted images
    transaction.execute(sql_delete, params![foreign_id])?;

    for image in &old_images {
      if !new_images.contains(image) {
        for p in 0..=2 {
          let path = format!(
            "{}/{}/{}_p{}.jpg",
            images_folder,
            image.get(0..=1).ok_or(NoneError)?,
            image,
            p
          );
          std::fs::remove_file(path)?;
        }
      }
    }
    // 2. move new images from temp dir
    {
      let mut stmt_insert = transaction.prepare(sql_insert)?;
      let mut stmt_delete_tmp =
        transaction.prepare("delete from `tmp_store_image` where `id` = :hash")?;

      for (id, image) in new_images.iter().enumerate() {
        if !old_images.contains(image) {
          let prefix = image.get(0..=1).ok_or(NoneError)?;
          std::fs::create_dir_all(format!("{}/{}", images_folder, prefix))?;
          for p in 0..=2 {
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
}
