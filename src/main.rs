#![feature(proc_macro_hygiene)]
#![feature(str_strip)]
#![feature(try_trait)]
#![feature(type_ascription)]

mod model;
mod web_error;
mod config;
mod auth;
mod util;
mod cli;
mod rpc;
mod templates;

use actix_web::{get, post, web, guard, App, HttpServer, HttpResponse, Result, middleware};
use actix_session::{CookieSession, Session};
use serde_json::Value as JsonValue;

type DB = web::Data<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>>;
type Config = web::Data<config::Config>;

#[get("/views/{uri:.+}")]
async fn sv_views(uri: web::Path<String>, db: DB, config: Config, session: Session) -> Result<HttpResponse> {
  let t0 = std::time::Instant::now();
  let uri = util::strip_slashes(uri.to_string());

  let user = auth::get_user(db.clone(), &session).await;
  if user.is_none() && (uri != "/login") { return Ok(util::redirect("views/login", &config)); };
  if user.is_some() && (uri == "/login") { return Ok(util::redirect("views/home",  &config)); };

  let res =
    templates::main(uri, db, config.get_ref(), user)
      .await
      .map(|res| HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(res.into_string()))
      .map_err(|e| {
        eprintln!("Error: {:?}", e);
        HttpResponse::InternalServerError().finish()
      })?;

  println!("profiling: {:?}", std::time::Instant::now().duration_since(t0));
  Ok(res)
}

#[post("/rpc/{uri:.+}")]
async fn sv_rpc(uri: web::Path<String>, payload: web::Payload, db: DB, config: Config, session: Session) -> Result<HttpResponse, HttpResponse> {
  let t0 = std::time::Instant::now();
  let uri = util::strip_slashes(uri.to_string());

  let user = auth::get_user(db.clone(), &session).await;
  if user.is_none() && (uri != "/auth/login") { return Ok(HttpResponse::Unauthorized().finish()); };

  // parse into untyped
  let post_data = serde_json::from_slice::<JsonValue>(
    util::read_payload(payload, config.get_ref())
        .await?
        .as_ref()
  ).map_err(|e| e.to_string())?;

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
  std::env::set_var("RUST_BACKTRACE", "1");
  env_logger::init();

  let config = config::load();

  // r2d2 pool
  let manager = r2d2_sqlite::SqliteConnectionManager::file(&config.server.db_path);
  let db_pool = r2d2::Pool::new(manager).unwrap();

  cli::load(&config, &db_pool);

  let server = HttpServer::new({
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
  }});
  if config.server.bind_addr.starts_with("unix:/"){
    #[cfg(target_os = "linux")] {
      server.bind_uds(config.server.bind_addr.strip_prefix("unix:").unwrap())?
        .run().await
    }
    #[cfg(not(target_os = "linux"))]
      panic!("Unix sockets are not available for this target");
  } else {
    server.bind(config.server.bind_addr)?
      .run().await
  }
}