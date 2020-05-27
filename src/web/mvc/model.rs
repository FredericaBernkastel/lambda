#![allow(non_camel_case_types)]
use crate::{
  error::Result,
  schema, util,
  web::DB,
  web::{self, Config},
};
use error_chain::bail;
use lazy_static::lazy_static;
use maud::Markup;
use num_traits::FromPrimitive;
use path_tree::PathTree;
use rusqlite::params;
use serde_json::json;
use std::collections::HashMap;

pub struct Model {
  db_pool: DB,
  config: Config,
  pub root_url: String,
  pub user: Option<schema::User>,
  pub get_data: HashMap<String, String>,
  path: String,
}

pub async fn main(
  uri: String,
  db_pool: DB,
  config: Config,
  user: Option<schema::User>,
) -> Result<Markup> {
  lazy_static! {
    static ref PATH_TREE: PathTree::<&'static str> = {
      let mut tmp = PathTree::<&str>::new();
      for path in vec![
        "/login",
        "/home",
        "/graffitis",
        "/graffitis/page/:page",
        "/graffiti/add",
        "/graffiti/:id",
        "/graffiti/:id/edit",
        "/authors",
        "/authors/page/:page",
        "/author/add",
        "/author/:id",
        "/author/:id/edit",
        "/tags",
        "/help",
      ] {
        tmp.insert(path, path);
      }
      tmp
    };
  };

  match PATH_TREE.find(uri.as_str()) {
    Some((path, data)) => {
      let path = *path;
      let get_data: HashMap<_, _> = data
        .into_iter()
        .map(|(arg, value)| (arg.to_string(), value.to_string()))
        .collect();

      let model = Model {
        db_pool,
        root_url: config.web.root_url.clone(),
        config,
        user,
        get_data,
        path: path.to_string(),
      };

      let page = match path {
        "/login" => model.m_login().await?,
        _ => {
          if model.user.is_none() {
            bail!("unauthorized");
          }

          match path {
            "/home" => model.m_home().await?,

            "/graffitis" => model.m_graffitis().await?, // --------------
            "/graffitis/page/:page" => model.m_graffitis().await?, // ---

            "/graffiti/add" => model.m_graffiti_edit().await?, // -------
            "/graffiti/:id" => model.m_graffiti().await?,      //       |
            "/graffiti/:id/edit" => model.m_graffiti_edit().await?, // --

            "/authors" => model.m_authors().await?, // ------------------
            "/authors/page/:page" => model.m_authors().await?, // -------

            "/author/add" => model.m_author_edit().await?, //------------
            "/author/:id" => model.m_author().await?,      //           |
            "/author/:id/edit" => model.m_author_edit().await?, // ------

            "/tags" => model.m_tags().await?,
            "/help" => model.m_help().await?,
            _ => unreachable!(),
          }
        }
      };

      Ok(model.m_root(page)?)
    }
    None => bail!("route not found"),
  }
}

impl Model {
  fn m_root(&self, page: Markup) -> Result<Markup> {
    let cors_h = util::gen_cors_hash(util::get_timestamp(), &self.config);

    let js_glob = json!({
      "path_t": self.path,
      "data": self.get_data,
      "root_url": self.root_url,
      "rpc": format!("{}rpc/", self.root_url),
      "cors_h": cors_h,
      "gmaps_api_key": self.config.web.gmaps_api_key
    });

    self.v_root(page, js_glob)
  }

  async fn m_login(&self) -> Result<Markup> {
    self.v_login()
  }

  async fn m_home(&self) -> Result<Markup> {
    type Graffiti = home_Graffiti;

    let (graffitis_recent, graffitis_last_checked, authors_last_checked) =
      web::block(self.db_pool.clone(), move |db| -> Result<_> {
        Ok((
          // graffitis_recent
          db.prepare(
            "select a.id as `0`,
                    b.hash as `1`,
                    c.gps_lat as `2`,
                    c.gps_long as `3`
               from graffiti a
                    left join graffiti_image b on b.graffiti_id = a.id and
                                                  b.`order` = 0
                    left join location c on c.graffiti_id = a.id
              order by a.id desc
              limit 0, 8",
          )?
          .query_map(params![], |row| {
            Ok(Graffiti {
              id: row.get(0)?,
              thumbnail: row.get(1)?,
              coords: (|| Some([row.get::<_, f64>(2).ok()?, row.get::<_, f64>(3).ok()?]))(),
            })
          })?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<Graffiti>,
          // graffitis_last_checked
          db.prepare(
            "select a.id as `0`,
                    b.hash as `1`
               from graffiti a
                    left join graffiti_image b on b.graffiti_id = a.id and
                                                  b.`order` = 0
              order by a.last_viewed desc
              limit 0, 4",
          )?
          .query_map(params![], |row| {
            Ok(Graffiti {
              id: row.get(0)?,
              thumbnail: row.get(1)?,
              coords: None,
            })
          })?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<Graffiti>,
          // authors_last_checked
          db.prepare(
            "select id as `0`,
                    name as `1`
               from author
              order by last_viewed desc
              limit 0, 6",
          )?
          .query_map(params![], |row| Ok((row.get(0)?, row.get(1)?)))?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<(u32, String)>,
        ))
      })
      .await?;

    self.v_home(
      graffitis_recent,
      graffitis_last_checked,
      authors_last_checked,
    )
  }

  async fn m_graffitis(&self) -> Result<Markup> {
    type Graffiti = graffitis_Graffiti;

    let page: i64 = self.get_data.get("page").unwrap_or(&"1".into()).parse()?;

    let (graffitis, total) = web::block(self.db_pool.clone(), {
      let rows_per_page = self.config.web.rows_per_page;
      move |db| -> Result<_> {
        Ok((
          // graffitis
          db.prepare(
            "with sub1 as (
              select id,
                     datetime,
                     views
                from graffiti
               order by id desc
               limit :page * :limit, :limit
            )
            select sub1.id as `0`,
                   sub1.datetime as `1`,
                   sub1.views as `2`,
                   a.city as `3`,
                   b.hash as `4`
              from sub1
                   left join location a on a.graffiti_id = sub1.id
                   left join graffiti_image b on b.graffiti_id = sub1.id and
                                            b.`order` = 0",
          )?
          .query_map(params![page - 1, rows_per_page], |row| {
            Ok(Graffiti {
              id: row.get(0)?,
              datetime: row.get(1)?,
              views: row.get(2)?,
              city: row.get(3)?,
              thumbnail: row.get(4)?,
            })
          })?
          .filter_map(std::result::Result::ok)
          .collect(),
          // total
          db.query_row("select count(*) from graffiti", params![], |row| {
            Ok(row.get::<_, u32>(0)?)
          })?,
        ))
      }
    })
    .await?;

    let mar_navigation = self.mar_navigation(
      "{}views/graffitis/page/{}",
      page,
      self.config.web.rows_per_page as i64,
      total as i64,
    )?;

    self.v_graffitis(graffitis, mar_navigation)
  }

  async fn m_graffiti_edit(&self) -> Result<Markup> {
    type Graffiti = graffiti_edit_Graffiti;
    type Location = graffiti_edit_Location;
    type Author = graffiti_Author;

    let ((graffiti, location), images, authors, tags) = if self.path == "/graffiti/:id/edit" {
      let id: u32 = self.get_data.get("id")?.parse()?;

      web::block(self.db_pool.clone(), move |db| -> Result<_> {
        Ok((
          db.query_row(
            "select a.complaint_id as `0`,
                   a.datetime as `1`,
                   a.shift_time as `2`,
                   a.intervening as `3`,
                   a.notes as `4`,
                   b.country as `5`,
                   b.city as `6`,
                   b.street as `7`,
                   b.place as `8`,
                   b.property as `9`,
                   b.gps_long as `10`,
                   b.gps_lat as `11`
              from graffiti a
                   left join location b on b.graffiti_id = a.id
             where a.id = :id",
            params![id],
            |row| {
              Ok((
                // graffiti
                Graffiti {
                  id: id.to_string(),
                  complaint_id: row.get(0)?,
                  date: row
                    .get::<_, Option<i64>>(1)?
                    .map_or("".into(), |x| util::format_timestamp(x as u64, "%Y-%m-%d")),
                  time: row
                    .get::<_, Option<i64>>(1)?
                    .map_or("".into(), |x| util::format_timestamp(x as u64, "%H:%M")),
                  shift_time: schema::ShiftTime::from_u8(row.get(2)?)
                    .unwrap_or(schema::ShiftTime::Afternoon),
                  intervening: row.get(3)?,
                  notes: row.get(4)?,
                },
                // location
                Location {
                  country: row.get(5)?,
                  city: row.get(6)?,
                  street: row.get(7)?,
                  place: row.get(8)?,
                  property: row.get(9)?,
                  gps: if let (Some(lat), Some(long)) = (
                    row.get::<_, Option<f64>>(11)?,
                    row.get::<_, Option<f64>>(10)?,
                  ) {
                    format!("{}, {}", lat, long)
                  } else {
                    "".into()
                  },
                },
              ))
            },
          )?,
          // images
          db.prepare(
            "select hash
              from graffiti_image
             where graffiti_id = :id
             order by `order` asc",
          )?
          .query_map(params![id], |row| Ok(row.get::<_, String>(0)?))?
          .filter_map(std::result::Result::ok)
          .collect(),
          // authors
          db.prepare(
            "select a.author_id,
                   a.indubitable,
                   b.name
              from graffiti_author a
                   inner join author b on a.author_id = b.id
             where graffiti_id = :id",
          )?
          .query_map(params![id], |row| {
            Ok(Author {
              id: row.get(0)?,
              indubitable: row.get(1)?,
              name: row.get(2)?,
            })
          })?
          .filter_map(std::result::Result::ok)
          .collect(),
          // tags
          db.prepare(
            "select b.name
              from graffiti_tag a
                   inner join tag b on b.id = a.tag_id
             where a.graffiti_id = :graffiti_id",
          )?
          .query_map(params![id], |row| Ok(row.get(0)?))?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<String>,
        ))
      })
      .await?
    } else {
      (
        (
          // graffiti
          Graffiti {
            id: "#".to_string(),
            complaint_id: "".to_string(),
            date: "".to_string(),
            time: "".to_string(),
            shift_time: schema::ShiftTime::Afternoon,
            intervening: "".to_string(),
            notes: "".to_string(),
          },
          // location
          Location {
            country: "".to_string(),
            city: "".to_string(),
            street: "".to_string(),
            place: "".to_string(),
            property: "".to_string(),
            gps: "".to_string(),
          },
        ),
        // images
        vec![],
        // authors
        vec![],
        // tags
        vec![],
      )
    };

    Ok(self.v_graffiti_edit(graffiti, location, images, authors, tags)?)
  }

  async fn m_graffiti(&self) -> Result<Markup> {
    type Author = graffiti_Author;

    let id: u32 = self.get_data.get("id")?.parse()?;

    let ((graffiti, location), images, authors, tags) =
      web::block(self.db_pool.clone(), move |db| -> Result<_> {
        Ok((
          // (graffiti, location)
          db.query_row(
            "select a.id as `0`,
                   a.complaint_id as `1`,
                   a.datetime as `2`,
                   a.shift_time as `3`,
                   a.intervening as `4`,
                   a.companions as `5`,
                   a.notes as `6`,
                   a.views as `7`,
                   b.graffiti_id as `8`,
                   b.country as `9`,
                   b.city as `10`,
                   b.street as `11`,
                   b.place as `12`,
                   b.property as `13`,
                   b.gps_long as `14`,
                   b.gps_lat as `15`
              from graffiti a
                   left join location b on b.graffiti_id = a.id
             where a.id = :id",
            params![id],
            |row| {
              Ok((
                schema::Graffiti {
                  id: row.get(0)?,
                  complaint_id: row.get(1)?,
                  datetime: row.get(2)?,
                  shift_time: row
                    .get::<_, Option<u8>>(3)?
                    .map(schema::ShiftTime::from_u8)
                    .flatten(),
                  intervening: row.get(4)?,
                  companions: row.get(5)?,
                  notes: row.get(6)?,
                  views: row.get(7)?,
                },
                schema::Location {
                  graffiti_id: row.get(8)?,
                  country: row.get(9)?,
                  city: row.get(10)?,
                  street: row.get(11)?,
                  place: row.get(12)?,
                  property: row.get(13)?,
                  gps_long: row.get(14)?,
                  gps_lat: row.get(15)?,
                },
              ))
            },
          )?,
          // images
          db.prepare(
            "select hash
              from graffiti_image
             where graffiti_id = :id
             order by `order` asc",
          )?
          .query_map(params![id], |row| Ok(row.get(0)?))?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<String>,
          // authors
          db.prepare(
            "select a.author_id,
                   a.indubitable,
                   b.name
              from graffiti_author a
                   inner join author b on a.author_id = b.id
             where graffiti_id = :id",
          )?
          .query_map(params![id], |row| {
            Ok(Author {
              id: row.get(0)?,
              indubitable: row.get(1)?,
              name: row.get(2)?,
            })
          })?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<Author>,
          // tags
          db.prepare(
            "select b.name
              from graffiti_tag a
                   inner join tag b on b.id = a.tag_id
             where a.graffiti_id = :graffiti_id",
          )?
          .query_map(params![id], |row| Ok(row.get(0)?))?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<String>,
        ))
      })
      .await?;

    // update views, non-blocking
    // SLOW
    actix_rt::spawn({
      let db = self.db_pool.get()?;
      async move {
        db.execute(
          "update graffiti
             set views = views + 1,
                 last_viewed = :timestamp
           where id = :id",
          params![util::get_timestamp() as i64, id],
        )
        .ok();
      }
    });

    self.v_graffiti(graffiti, location, images, authors, tags)
  }

  async fn m_authors(&self) -> Result<Markup> {
    type Author = authors_Author;

    let page: i64 = self.get_data.get("page").unwrap_or(&"1".into()).parse()?;
    let (authors, total) = web::block(self.db_pool.clone(), {
      let rows_per_page = self.config.web.rows_per_page;
      move |db| -> Result<_> {
        Ok((
          // authors
          db.prepare(
            "with sub1 as (
              select id,
                     name,
                     age,
                     home_city,
                     views
                from author
               order by id desc
               limit :page * :limit, :limit
            )
            select sub1.id as `0`,
                   sub1.name as `1`,
                   sub1.age as `2`,
                   sub1.home_city as `3`,
                   sub1.views as `4`,
                   a.hash as `5`,
                   count(b.author_id) as `6`
              from sub1
                   left join author_image a on a.author_id = sub1.id and
                                               a.`order` = 0
                   left join graffiti_author b on b.author_id = sub1.id
             group by sub1.id
             order by sub1.id desc",
          )?
          .query_map(params![page - 1, rows_per_page], |row| {
            Ok(Author {
              id: row.get(0)?,
              name: row.get(1)?,
              age: row.get(2)?,
              home_city: row.get(3)?,
              views: row.get(4)?,
              thumbnail: row.get(5)?,
              graffiti: row.get(6)?,
            })
          })?
          .filter_map(std::result::Result::ok)
          .collect(),
          // total
          db.query_row("select count(*) from author", params![], |row| {
            Ok(row.get::<_, u32>(0)?)
          })?,
        ))
      }
    })
    .await?;

    let mar_navigation = self.mar_navigation(
      "{}views/authors/page/{}",
      page,
      self.config.web.rows_per_page as i64,
      total as i64,
    )?;

    self.v_authors(authors, mar_navigation)
  }

  async fn m_author_edit(&self) -> Result<Markup> {
    type Author = author_edit_Author;

    let (author, images) = if self.path == "/author/:id/edit" {
      let id: u32 = self.get_data.get("id")?.parse()?;
      web::block(self.db_pool.clone(), move |db| -> Result<_> {
        Ok((
          // author
          db.query_row(
            "select name as `0`,
                   age as `1`,
                   height as `2`,
                   handedness as `3`,
                   home_city as `4`,
                   social_networks as `5`,
                   notes as `6`
              from author
             where id = :id",
            params![id],
            |row| {
              Ok(Author {
                id: id.to_string(),
                name: row.get(0)?,
                age: row
                  .get::<_, Option<u32>>(1)?
                  .map_or("".into(), |x| x.to_string()),
                height: row
                  .get::<_, Option<u32>>(2)?
                  .map_or("".into(), |x| x.to_string()),
                handedness: schema::Handedness::from_u8(row.get(3)?)
                  .unwrap_or(schema::Handedness::RightHanded),
                home_city: row.get(4)?,
                social_networks: row.get(5)?,
                notes: row.get(6)?,
              })
            },
          )?,
          // images
          db.prepare(
            "select hash
              from author_image
             where author_id = :id
             order by `order` asc",
          )?
          .query_map(params![id], |row| Ok(row.get::<_, String>(0)?))?
          .filter_map(std::result::Result::ok)
          .collect(),
        ))
      })
      .await?
    } else {
      (
        // author
        Author {
          id: "#".to_string(),
          name: "".to_string(),
          age: "".to_string(),
          height: "".to_string(),
          handedness: schema::Handedness::RightHanded,
          home_city: "".to_string(),
          social_networks: "".to_string(),
          notes: "".to_string(),
        },
        // images
        vec![],
      )
    };

    self.v_author_edit(author, images)
  }

  async fn m_author(&self) -> Result<Markup> {
    use rusqlite::OptionalExtension;

    type GraffitiImg = (/* id:  */ u32, /* thumbnail: */ Option<String>);

    let id: u32 = self.get_data.get("id")?.parse()?;

    let (
      author,
      images,
      graffiti_count,
      graffiti_recent,
      graffiti_most_viewed,
      aggregate_counties,
      aggregate_cities,
      aggregate_gps,
    ) = web::block(self.db_pool.clone(), move |db| -> Result<_> {
      Ok((
        // author
        db.query_row(
          "select name as `0`,
                 age as `1`,
                 height as `2`,
                 handedness as `3`,
                 home_city as `4`,
                 social_networks as `5`,
                 notes as `6`,
                 views as `7`
            from author
           where id = :id",
          params![id],
          |row| {
            Ok(schema::Author {
              id: id,
              name: row.get(0)?,
              age: row.get(1)?,
              height: row.get(2)?,
              handedness: schema::Handedness::from_u8(row.get(3)?),
              home_city: row.get(4)?,
              social_networks: row.get(5)?,
              notes: row.get(6)?,
              views: row.get(7)?,
            })
          },
        )?,
        // images
        db.prepare(
          "select hash
            from author_image
           where author_id = :id
           order by `order` asc",
        )?
        .query_map(params![id], |row| Ok(row.get(0)?))?
        .filter_map(std::result::Result::ok)
        .collect(): Vec<String>,
        // graffiti_count
        db.query_row(
          "select count( * ) 
            from graffiti_author
           where author_id = :id",
          params![id],
          |row| Ok(row.get(0)?),
        )?: u32,
        // graffiti_recent
        db.query_row(
          "select a.graffiti_id,
                 c.hash
            from graffiti_author a
                 inner join graffiti b on b.id = a.graffiti_id
                 left join graffiti_image c on c.graffiti_id = b.id and 
                                               c.`order` = 0
           where a.author_id = :id
           order by a.graffiti_id desc
           limit 1",
          params![id],
          |row| Ok((row.get(0)?, row.get(1)?): GraffitiImg),
        )
        .optional()?,
        // graffiti_most_viewed
        db.query_row(
          "select a.graffiti_id,
                 c.hash
            from graffiti_author a
                 inner join graffiti b on b.id = a.graffiti_id
                 left join graffiti_image c on c.graffiti_id = b.id and 
                                               c.`order` = 0
           where a.author_id = :id
           order by b.views desc
           limit 1",
          params![id],
          |row| Ok((row.get(0)?, row.get(1)?): GraffitiImg),
        )
        .optional()?,
        // aggregate_counties
        db.prepare(
          "select b.country,
                 count(b.country) as count 
            from graffiti_author a
                 inner join location b on b.graffiti_id = a.graffiti_id
           where author_id = :id
           group by lower(b.country)
           order by count desc, b.country asc",
        )?
        .query_map(params![id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(std::result::Result::ok)
        .collect(): Vec<(String, u32)>,
        // aggregate_cities
        db.prepare(
          "select b.city,
                 count(b.city) as count
            from graffiti_author a
                 inner join location b on b.graffiti_id = a.graffiti_id
           where author_id = :id
           group by lower(b.city)
           order by count desc, b.city asc",
        )?
        .query_map(params![id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(std::result::Result::ok)
        .collect(): Vec<(String, u32)>,
        // aggregate_gps
        db.prepare(
          "select a.graffiti_id,
                 b.gps_lat,
                 b.gps_long,
                 c.hash
            from graffiti_author a
                 inner join location b on b.graffiti_id = a.graffiti_id
                 left join graffiti_image c on c.graffiti_id = a.graffiti_id and 
                                               c.`order` = 0
           where author_id = :id",
        )?
        .query_map(params![id], |row| {
          Ok(home_Graffiti {
            id: row.get(0)?,
            coords: Some([row.get(1)?, row.get(2)?]),
            thumbnail: row.get(3)?,
          })
        })?
        .filter_map(std::result::Result::ok)
        .collect(): Vec<home_Graffiti>,
      ))
    })
    .await?;

    // update views, non-blocking
    // SLOW
    actix_rt::spawn({
      let db = self.db_pool.get()?;
      async move {
        db.execute(
          "update author
             set views = views + 1,
                 last_viewed = :timestamp
           where id = :id",
          params![util::get_timestamp() as i64, id],
        )
        .ok();
      }
    });

    self.v_author(
      author,
      images,
      graffiti_count,
      graffiti_recent,
      graffiti_most_viewed,
      aggregate_counties,
      aggregate_cities,
      aggregate_gps,
    )
  }

  async fn m_tags(&self) -> Result<Markup> {
    self.v_tags()
  }

  async fn m_help(&self) -> Result<Markup> {
    self.v_help()
  }
}

pub struct graffitis_Graffiti {
  pub id: u32,
  pub datetime: Option<i64>,
  pub views: u32,
  pub city: String,
  pub thumbnail: Option<String>,
}

pub struct graffiti_edit_Graffiti {
  pub id: String,
  pub complaint_id: String,
  pub date: String,
  pub time: String,
  pub shift_time: schema::ShiftTime,
  pub intervening: String,
  pub notes: String,
}

pub struct graffiti_edit_Location {
  pub country: String,
  pub city: String,
  pub street: String,
  pub place: String,
  pub property: String,
  pub gps: String,
}

pub struct graffiti_Author {
  pub id: u32,
  pub indubitable: bool,
  pub name: String,
}

pub struct authors_Author {
  pub id: u32,
  pub name: String,
  pub age: Option<u32>,
  pub graffiti: u32,
  pub home_city: String,
  pub views: u32,
  pub thumbnail: Option<String>,
}

pub struct author_edit_Author {
  pub id: String,
  pub name: String,
  pub age: String,
  pub height: String,
  pub handedness: schema::Handedness,
  pub home_city: String,
  pub social_networks: String,
  pub notes: String,
}

#[derive(serde::Serialize)]
pub struct home_Graffiti {
  pub id: u32,
  pub thumbnail: Option<String>,
  pub coords: Option<[f64; 2]>,
}

#[derive(Default)]
pub struct graffitis_SearchOpts {
  pub country: Option<String>,
  pub city: Option<String>,
  pub street: Option<String>,
  pub place: Option<String>,
  pub property: Option<String>,
  pub date_before: Option<String>,
  pub date_after: Option<String>,
  pub authors_number: Option<u32>,
  pub authors: Vec<graffiti_Author>,
  pub tags: Vec<String>,
}
