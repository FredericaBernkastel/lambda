use super::{view, Model, View};
use crate::{
  error::{ErrorKind, Result},
  log_error, routes, schema, util,
  web::DB,
  web::{self, Config},
};
use async_trait::async_trait;
use error_chain::bail;
use futures::core_reexport::pin::Pin;
use futures::Future;
use lazy_static::lazy_static;
use maud::html;
use maud::Markup;
use path_tree::PathTree;
use rusqlite::params;
use serde_json::json;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Context {
  db_pool: DB,
  config: Config,
  pub root_url: String,
  pub user: Option<schema::User>,
  pub get_data: HashMap<String, String>,
  pub path: String,
  pub path_t: String,
}

type PTreeInner = (&'static str, &'static dyn Model<View = ()>);

thread_local! {
  static PATH_TREE: Rc < PathTree :: <PTreeInner>> = {
    Rc::new(routes![PTreeInner;
       "/login" => Login,
       "/home" => Home,

      /*["/graffitis",
       "/graffitis/page/:page"] => Graffitis,
      ["/graffitis/search/:x-data",
       "/graffitis/search/:x-data/page/:page"] => GraffitisSearch,

      ["/graffiti/add",
       "/graffiti/:id/edit"] => GraffitiEdit,
       "/graffiti/:id" => Graffiti,

      ["/authors",
       "/authors/page/:page"] => Authors,
      ["/authors/search/:x-data",
       "/authors/search/:x-data/page/:page"] => AuthorsSearch,

      ["/author/add",
       "/author/:id/edit"] => AuthorEdit,
       "/author/:id" => Author,

       "/tags" => Tags,
       "/help" => Help*/
    ])
  };
}

pub async fn main(
  uri: String,
  db_pool: DB,
  config: Config,
  user: Option<schema::User>,
) -> Result<Markup> {
  let path_tree = PATH_TREE.with(|x| x.clone());

  match path_tree.find(uri.as_str()) {
    Some((_route, data)) => {
      let (path, model) = (_route.0, _route.1);
      let get_data: HashMap<_, _> = data
        .into_iter()
        .map(|(arg, value)| (arg.to_string(), value.to_string()))
        .collect();

      let ctx = Rc::new(Context {
        db_pool,
        root_url: config.web.root_url.clone(),
        config,
        user,
        get_data,
        path: uri.clone(),
        path_t: path.to_string(),
      });

      let page = match (path, &ctx.user) {
        ("/login", None) | (_, Some(_)) => model.exec(ctx.clone()).await?,
        (_, _) => bail!("unauthorized"),
      };

      Root { body: page }.exec_sync(ctx)
    }
    None => bail!(ErrorKind::RouteNotFound),
  }
}

struct Root {
  body: Markup,
}
#[async_trait(?Send)]
impl Model for Root {
  type View = view::Root<'static>;
  fn exec_sync(&self, ctx: Rc<Context>) -> Result<Markup> {
    let cors_h = util::gen_cors_hash(util::get_timestamp(), &ctx.config);

    let js_glob = json!({
      "path_t": ctx.path_t,
      "data": ctx.get_data,
      "root_url": ctx.root_url,
      "rpc": format!("{}rpc/", ctx.root_url),
      "cors_h": cors_h,
      "gmaps_api_key": ctx.config.web.gmaps_api_key
    });

    view::Root {
      body: &self.body,
      js_glob,
    }
    .render(&ctx)
  }
}

struct Login;
#[async_trait(?Send)]
impl Model for Login {
  async fn exec(&self, ctx: Rc<Context>) -> Result<Markup> {
    view::Login.render(&ctx)
  }
}

struct Home;
#[async_trait(?Send)]
impl Model for Home {
  async fn exec(&self, ctx: Rc<Context>) -> Result<Markup> {
    web::block(ctx.db_pool.clone(), move |db| -> Result<_> {
      Ok(view::Home {
        graffitis_recent: db
          .prepare(
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
            Ok(view::HomeGraffiti {
              id: row.get(0)?,
              thumbnail: row.get(1)?,
              coords: (|| Some([row.get::<_, f64>(2).ok()?, row.get::<_, f64>(3).ok()?]))(),
            })
          })?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<_>,
        graffitis_last_checked: db
          .prepare(
            "select a.id as `0`,
                  b.hash as `1`
             from graffiti a
                  left join graffiti_image b on b.graffiti_id = a.id and
                                                b.`order` = 0
            order by a.last_viewed desc
            limit 0, 4",
          )?
          .query_map(params![], |row| {
            Ok(view::HomeGraffiti {
              id: row.get(0)?,
              thumbnail: row.get(1)?,
              coords: None,
            })
          })?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<_>,
        authors_last_checked: db
          .prepare(
            "select id as `0`,
                  name as `1`
             from author
            order by last_viewed desc
            limit 0, 6",
          )?
          .query_map(params![], |row| {
            Ok(view::HomeAuthor {
              id: row.get(0)?,
              name: row.get(1)?,
            })
          })?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<_>,
      })
    })
    .await?
    .render(&ctx)
  }
}

impl Context {
  pub fn svg_sprite(&self, id: &str, classname: &str, title: &str) -> Markup {
    html! {
      svg.(classname) {
        @if !title.is_empty() {
          title { (title) }
        }
        use xlink:href={ (self.root_url) "static/img/sprite.svg#" (id) }{  }
      }
    }
  }

  pub fn mar_header(&self) -> Result<Markup> {
    Ok(html! {
      .popup-wrapper#error {
        .popup {
          p.title { "Error!" }
          .inner {
            .message {  }
            .actions-wrapper {
              span.action-btn#close { "Ok" }
            }
          }
        }
      }

      .popup-wrapper#warning {
        .popup {
          p.title { (self.svg_sprite("exclamation-triangle", "", "")) }
          .inner {
            .message {  }
            .actions-wrapper {
              span.action-btn.red#ok { "Ok" }
              span.action-btn#cancel { "Cancel" }
            }
          }
        }
      }

      .header {
        .container {
          .logo { "Graffiti database" }
          .nav-menu {
            .pages {
              a href={ (self.root_url) "views/home" } { "Home" }
              a href={ (self.root_url) "views/graffitis" } { "Graffiti" }
              a href={ (self.root_url) "views/authors" } { "Authors" }
              a href={ (self.root_url) "views/tags" } { "Tags" }
              a href={ (self.root_url) "views/help" } { "Help" }
            }
            .languages {
              a href={ (self.root_url) "es/views" (self.path) } title="Español" alt="Español" {
                img src={ (self.root_url) "static/img/es.svg" };
              }
              a href={ (self.root_url) "en/views" (self.path) } title="English" alt="English" {
                img src={ (self.root_url) "static/img/uk.svg" };
              }
            }
            .user {
              (self.svg_sprite("user", "icon-user", ""))
              span.login { (self.user.as_ref()?.login) }
              (self.svg_sprite("sign-out-alt", "logout", "logout"))
            }
          }
        }
      }
    })
  }
}

/*
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
      "{root_url}views/graffitis/page/{id}",
      page,
      self.config.web.rows_per_page as i64,
      total as i64,
    )?;

    self.v_graffitis(graffitis, mar_navigation, None, vec![])
  }

  async fn m_graffitis_search(&self) -> Result<Markup> {
    type Graffiti = graffitis_Graffiti;

    let page: i64 = self.get_data.get("page").unwrap_or(&"1".into()).parse()?;
    let data = self.get_data.get("x-data")?;

    let request = util::b64_gunzip_deserialize_t::<graffitis_SearchOpts>(data)?;

    let (graffitis, aggregate_gps, total) = web::block(self.db_pool.clone(), {
      let rows_per_page = self.config.web.rows_per_page;
      let request = request.clone();
      move |db| -> Result<_> {
        let mut dyn_stmt = util::DynQuery::new();
        dyn_stmt.push(&format!(
          "select e.id as `0`,
                   e.datetime as `1`,
                   e.views as `2`,
                   a.city as `3`,
                   b.hash as `4`,
                   a.gps_lat as `5`,
                   a.gps_long as `6`
              from graffiti e
                   left join location a on a.graffiti_id = e.id
                   left join graffiti_image b on b.graffiti_id = e.id and
                                                 b.`order` = 0
                   left join graffiti_author c on c.graffiti_id = e.id
                   {tags}
             where true",
          tags = request
            .tags
            .iter()
            .enumerate()
            .map(|(i, _)| {
              format!(
                "inner join graffiti_tag tag_j{i} on
                  tag_j{i}.graffiti_id = e.id and tag_j{i}.tag_id = :tag{i}",
                i = i
              )
            })
            .collect::<Vec<String>>()
            .join("\n")
        ));
        request
          .tags
          .into_iter()
          .enumerate()
          .for_each(|(i, (tag, _))| {
            dyn_stmt.bind(format!(":tag{}", i), tag);
          });

        if let Some(country) = request.country {
          dyn_stmt
            .push(" and a.country = :country")
            .bind(":country".to_owned(), country);
        }
        if let Some(city) = request.city {
          dyn_stmt
            .push(" and a.city = :city")
            .bind(":city".to_owned(), city);
        }
        if let Some(street) = request.street {
          dyn_stmt
            .push(" and a.street = :street")
            .bind(":street".to_owned(), street);
        }
        if let Some(place) = request.place {
          dyn_stmt
            .push(" and a.place = :place")
            .bind(":place".to_owned(), place);
        }
        if let Some(property) = request.property {
          dyn_stmt
            .push(" and a.property = :property")
            .bind(":property".to_owned(), property);
        }
        if let Some(date_before) = request.date_before {
          dyn_stmt.push(" and e.datetime < :date_before").bind(
            ":date_before".to_owned(),
            util::datetime_variable(&date_before)?
          );
        }
        if let Some(date_after) = request.date_after {
          dyn_stmt.push(" and e.datetime > :date_after").bind(
            ":date_after".to_owned(),
            util::datetime_variable(&date_after)?,
          );
        }
        if let Some(author_count) = request.authors_number {
          dyn_stmt
            .push(" and e.author_count = :author_count")
            .bind(":author_count".to_owned(), author_count);
        }
        for (i, author) in request.authors.iter().enumerate() {
          let op = if i == 0 { "and" } else { "or" };
          if author.indubitable {
            dyn_stmt.push(&format!(
              " {} (c.author_id, c.indubitable) in ( values (:author{}, true) )",
              op, i
            ))
          } else {
            dyn_stmt.push(&format!(" {} c.author_id in (:author{})", op, i))
          }
          .bind(format!(":author{}", i), author.id);
        }
        dyn_stmt
          .push(
            " group by e.id
            order by e.id desc
            limit :page * :limit, :limit",
          )
          .bind(":page".to_owned(), page - 1)
          .bind(":limit".to_owned(), rows_per_page);

        let mut params: Vec<(&str, &dyn rusqlite::ToSql)> = vec![];
        let mut params_count: Vec<(&str, &dyn rusqlite::ToSql)> = vec![];
        for (k, v) in &dyn_stmt.params {
          params.push((k, v));
          match k.as_str() {
            ":page" => params_count.push((k, &0)),
            ":limit" => params_count.push((k, &-1)),
            _ => params_count.push((k, v)),
          };
        }

        //graffitis
        let graffitis = db
          .prepare(&dyn_stmt.sql)?
          .query_map_named(params.as_slice(), |row| {
            Ok(Graffiti {
              id: row.get(0)?,
              datetime: row.get(1)?,
              views: row.get(2)?,
              city: row.get(3)?,
              thumbnail: row.get(4)?,
            })
          })?
          .filter_map(std::result::Result::ok)
          .collect();

        let mut total = 0;

        // total & gps
        let aggregate_gps = db
          .prepare(&dyn_stmt.sql)?
          .query_map_named(params_count.as_slice(), |row| {
            total += 1;
            Ok(home_Graffiti {
              id: row.get(0)?,
              thumbnail: row.get(4)?,
              coords: Some([row.get::<_, f64>(5)?, row.get::<_, f64>(6)?]),
            })
          })?
          .filter_map(std::result::Result::ok)
          .collect();

        Ok((graffitis, aggregate_gps, total))
      }
    })
    .await?;

    let mar_navigation = self.mar_navigation(
      &format!("{{root_url}}views/graffitis/search/{}/page/{{id}}", data),
      page,
      self.config.web.rows_per_page as i64,
      total as i64,
    )?;

    self.v_graffitis(graffitis, mar_navigation, Some(request), aggregate_gps)
  }

  async fn m_graffiti_edit(&self) -> Result<Markup> {
    type Graffiti = graffiti_edit_Graffiti;
    type Location = graffiti_edit_Location;
    type Author = graffiti_Author;

    let ((graffiti, location), images, authors, tags) = if self.path_t == "/graffiti/:id/edit" {
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
                  shift_time: serde_json::from_value(row.get::<_, u8>(2)?.into())
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
                    .map(|x| serde_json::from_value(x.into()).ok())
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
            "select b.id, b.name
              from graffiti_tag a
                   inner join tag b on b.id = a.tag_id
             where a.graffiti_id = :graffiti_id",
          )?
          .query_map(params![id], |row| Ok((row.get(0)?, row.get(1)?)))?
          .filter_map(std::result::Result::ok)
          .collect(): Vec<(u32, String)>,
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
          .map_err(|e| log_error!(e, "actix_rt::spawn"))
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
      "{root_url}views/authors/page/{id}",
      page,
      self.config.web.rows_per_page as i64,
      total as i64,
    )?;

    self.v_authors(authors, mar_navigation, None)
  }

  async fn m_authors_search(&self) -> Result<Markup> {
    type Author = authors_Author;

    let page: i64 = self.get_data.get("page").unwrap_or(&"1".into()).parse()?;
    let data = self.get_data.get("x-data")?;

    let request = util::b64_gunzip_deserialize_t::<authors_SearchOpts>(data)?;

    let (authors, total) = web::block(self.db_pool.clone(), {
      let rows_per_page = self.config.web.rows_per_page;
      let request = request.clone();
      move |db| -> Result<_> {
        let mut dyn_stmt = util::DynQuery::new();
        dyn_stmt.push(
          "select id,
                 name,
                 age,
                 home_city,
                 views
            from author a
           where true",
        );
        if let Some(age_min) = request.age_min {
          dyn_stmt
            .push(" and a.age >= :age_min")
            .bind(":age_min".into(), age_min);
        }
        if let Some(age_max) = request.age_max {
          dyn_stmt
            .push(" and a.age < :age_max")
            .bind(":age_max".into(), age_max);
        }
        if let Some(height_min) = request.height_min {
          dyn_stmt
            .push(" and a.height >= :height_min")
            .bind(":height_min".into(), height_min);
        }
        if let Some(height_max) = request.height_max {
          dyn_stmt
            .push(" and a.height < :height_max")
            .bind(":height_max".into(), height_max);
        }
        if let Some(handedness) = request.handedness {
          dyn_stmt
            .push(" and a.handedness = :handedness")
            .bind(":handedness".into(), handedness as u8);
        }
        if request.social_has {
          dyn_stmt.push(" and a.social_networks <> ''");
        }
        if let Some(home_city) = request.home_city {
          dyn_stmt
            .push(" and a.home_city like :home_city")
            .bind(":home_city".into(), format!("%{}%", home_city));
        }

        // companion with
        if request.companions.len() != 0 {
          let mut companions_pmt = "".to_string();
          let mut companions = "".to_string();
          for (i, companion) in request.companions.iter().enumerate() {
            if i != 0 {
              companions_pmt += " intersect ";
              companions += ",";
            }
            companions_pmt += &format!(
              "select graffiti_id
              from graffiti_author
             where author_id = :companion{}",
              i
            );
            companions += &format!("(:companion{})", i);
            dyn_stmt.bind(format!(":companion{}", i), companion.id);
          }

          let companion_1_exclude = if request.companions.len() == 1 {
            format!("and author_id not in ({})", companions)
          } else {
            "".to_string()
          };

          dyn_stmt.sql = format!(
            "with a as ({sub0}),
            sub2 as (
              with sub2_sub1 as ({companions_pmt})
              select author_id
                from graffiti_author
               where graffiti_id in sub2_sub1 {companion_1_exclude}
            )
            select *
              from a
             where a.id in sub2",
            sub0 = dyn_stmt.sql,
            companions_pmt = companions_pmt,
            companion_1_exclude = companion_1_exclude
          );
        }

        // active in
        if request.active_in.len() != 0 {
          let mut terms = "".to_string();
          for (i, term) in request.active_in.into_iter().enumerate() {
            if i != 0 {
              terms += ",";
            }
            terms += &format!(":active_in{}", i);
            dyn_stmt.bind(format!(":active_in{}", i), term);
          }
          dyn_stmt.sql = format!(
            "with a as ({sub0}),
            sub2 as (
              with sub2_sub1 as (
                select graffiti_id
                  from location
                 where country in ({terms}) or
                       city in ({terms}) or
                       street in ({terms})
                )
                select distinct graffiti_author.author_id
                  from sub2_sub1
                       inner join graffiti on graffiti.id = sub2_sub1.graffiti_id
                       inner join graffiti_author on graffiti_author.graffiti_id = graffiti.id
            )
            select *
              from a
             where a.id in sub2",
            sub0 = dyn_stmt.sql,
            terms = terms
          )
        }

        dyn_stmt.push(" limit :page * :limit, :limit")
          .bind(":page".to_owned(), page - 1)
          .bind(":limit".to_owned(), rows_per_page);

        let mut dyn_stmt_count = util::DynQuery::new();
        dyn_stmt_count.push(&format!("select count( * ) from ({})", &dyn_stmt.sql));

        dyn_stmt.sql = format!(
          "with sub1 as ({})
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
          dyn_stmt.sql
        );

        let mut params: Vec<(&str, &dyn rusqlite::ToSql)> = vec![];
        let mut params_count: Vec<(&str, &dyn rusqlite::ToSql)> = vec![];
        for (k, v) in &dyn_stmt.params {
          params.push((k, v));
          match k.as_str() {
            ":page" => params_count.push((k, &0)),
            ":limit" => params_count.push((k, &-1)),
            _ => params_count.push((k, v)),
          };
        }

        Ok((
          //graffitis
          db.prepare(&dyn_stmt.sql)?
            .query_map_named(params.as_slice(), |row| {
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
          db.prepare(&dyn_stmt_count.sql)?
            .query_row_named(params_count.as_slice(), |row| row.get::<_, i64>(0))?,
        ))
      }
    })
    .await?;

    let mar_navigation = self.mar_navigation(
      &format!("{{root_url}}views/authors/search/{}/page/{{id}}", data),
      page,
      self.config.web.rows_per_page as i64,
      total,
    )?;

    self.v_authors(authors, mar_navigation, Some(request))
  }

  async fn m_author_edit(&self) -> Result<Markup> {
    type Author = author_edit_Author;

    let (author, images) = if self.path_t == "/author/:id/edit" {
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
                handedness: serde_json::from_value(row.get::<_, u8>(3)?.into())
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
      aggregate_companions,
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
              handedness: serde_json::from_value(row.get::<_, u8>(3)?.into()).ok(),
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
        // aggregate_companions
        db.prepare(
          "with sub1 as (
            select graffiti_id
              from graffiti_author
             where author_id = :author_id
          ),
          sub2 as (
            select author_id
              from graffiti_author
             where graffiti_id in sub1 and
                   author_id <> :author_id
             group by author_id
          )
          select author.id,
                 author.name
            from sub2
                 inner join author on author.id = sub2.author_id
           order by author.name asc",
        )?
        .query_map(params![id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(std::result::Result::ok)
        .collect(): Vec<(u32, String)>,
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
          .map_err(|e| log_error!(e, "actix_rt::spawn"))
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
      aggregate_companions,
    )
  }

  async fn m_tags(&self) -> Result<Markup> {
    let tags = web::block(self.db_pool.clone(), move |db| -> Result<_> {
      Ok(
        db.prepare(
          "select id, name, count
          from tag
         order by name asc",
        )?
        .query_map(params![], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .filter_map(std::result::Result::ok)
        .collect(): Vec<(u32, String, u32)>,
      )
    })
    .await?;
    self.v_tags(tags)
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

#[derive(Debug, Clone, serde::Deserialize)]
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

#[derive(Default, Clone, serde::Deserialize)]
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
  pub tags: Vec<(u32, String)>,
}

#[derive(Default, Debug, Clone, serde::Deserialize)]
pub struct authors_SearchOpts {
  pub companions: Vec<graffiti_Author>,
  pub age_min: Option<u32>,
  pub age_max: Option<u32>,
  pub height_min: Option<u32>,
  pub height_max: Option<u32>,
  pub handedness: Option<schema::Handedness>,
  pub social_has: bool,
  pub home_city: Option<String>,
  pub active_in: Vec<String>,
}
*/
