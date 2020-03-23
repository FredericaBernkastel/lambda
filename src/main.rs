#![feature(proc_macro_hygiene)]
#![feature(try_trait)]
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate clap;
#[macro_use] extern crate rusqlite;
#[macro_use] extern crate actix_web;
#[macro_use] extern crate json;
mod model;
mod config;
mod auth;
mod util;
mod cli;
mod rpc;
mod templates;

use actix_web::{get, web, guard, App, HttpServer, HttpResponse, Result, middleware, error::BlockingError};
use actix_session::{CookieSession, Session};

type DB = web::Data<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>>;
type DBConn = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;
type Config = web::Data<config::Config>;

fn redirect(path: &str, config: &Config) -> HttpResponse {
  HttpResponse::Found().header("location", config.web.root_url.clone() + path).finish()
}

async fn get_user(db: DB, session: &Session) -> Option<model::User>{
  // check session
  match session.get::<String>("ssid").ok().flatten() {
    Some(ssid) => auth::check_session(&ssid, db).await,
    None => None
  }
}

fn strip_slashes(mut uri: String) -> String {
  if uri.ends_with('/') { uri.pop(); }
  "/".to_string() + &uri
}

#[get("/views/{uri:.+}")]
async fn sv_views(uri: web::Path<String>, db: DB, config: Config, session: Session) -> Result<HttpResponse> {
  let t0 = std::time::Instant::now();
  let uri = strip_slashes(uri.to_string());

  let user = get_user(db.clone(), &session).await;
  if user.is_none() && (uri != "/login") { return Ok(redirect("views/login", &config)); };
  if user.is_some() && (uri == "/login") { return Ok(redirect("views/home",  &config)); };

  let res = web::block(move || {
    let db = db.get().unwrap();

    templates::main(uri, db, config.get_ref(), user)
      .map_err(|e| {
        eprintln!("Error: {:?}", e);
        e.to_string()
      })
  }).await
    .map(|res| HttpResponse::Ok()
      .content_type("text/html; charset=utf-8")
      .body(res.into_string()))
    .map_err(|e| match e {
      BlockingError::Error(e) => HttpResponse::Forbidden().body(e),
      BlockingError::Canceled => HttpResponse::InternalServerError().finish()
    })?;

  println!("profiling: {:?}", std::time::Instant::now().duration_since(t0));
  Ok(res)
}

#[post("/rpc/{uri:.+}")]
async fn sv_rpc(uri: web::Path<String>, post_data: bytes::Bytes, db: DB, config: Config, session: Session) -> Result<HttpResponse, HttpResponse> {
  let t0 = std::time::Instant::now();
  let uri = strip_slashes(uri.to_string());

  let user = get_user(db.clone(), &session).await;
  if user.is_none() && (uri != "/auth/login") { return Ok(HttpResponse::Unauthorized().finish()); };

  let res = rpc::main(uri, post_data, db, config.get_ref(), user, session)
    .await
    .map(|res| HttpResponse::Ok().json(res))
    .map_err(|e| {
      eprintln!("Error: {:?}", e);
      HttpResponse::Forbidden().body(e.to_string())
    });

  println!("profiling: {:?}", std::time::Instant::now().duration_since(t0));
  res
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  std::env::set_var("RUST_LOG", "actix_web=debug");
  env_logger::init();

  let config = config::load();

  // r2d2 pool
  let manager = r2d2_sqlite::SqliteConnectionManager::file(&config.server.db_path);
  let db_pool = r2d2::Pool::new(manager).unwrap();

  cli::load(&config, &db_pool);

  HttpServer::new({
    let config = config.clone();
    move || {
      App::new()
        .data(db_pool.clone())
        .data(config.to_owned())
        .wrap(middleware::Logger::default())
        .wrap(middleware::DefaultHeaders::new().header("content-type", "text/plain; charset=utf-8").content_type())
        .wrap(CookieSession::signed(config.web.secret_key.as_bytes()).secure(false))
        .service(sv_views)
        .service(sv_rpc)
        .service(actix_files::Files::new("/static", "./data/static"))

        .default_service(
          // 404 for GET request
          web::resource("")
            .route(web::get().to(|| HttpResponse::NotFound()))
            // all requests that are not `GET`
            .route(
              web::route()
                .guard(guard::Not(guard::Get()))
                .to(HttpResponse::MethodNotAllowed),
            ),
        )
  }})
    .bind(&config.server.bind_addr)?
    .run()
    .await
}