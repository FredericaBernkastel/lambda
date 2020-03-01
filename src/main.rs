#![feature(proc_macro_hygiene)]
mod templates;
mod config;

use actix_web::{get, web, guard, App, HttpServer, HttpResponse, Result, middleware};

type DB = web::Data<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>>;
type Config = web::Data<config::Config>;

#[get("/views/{uri:.+}")]
async fn views(uri: web::Path<String>, db: DB, config: Config) -> Result<HttpResponse> {
  let t0 = std::time::Instant::now();
  let mut uri = uri.to_string();
  if uri.ends_with('/') { uri.pop(); }

  let res = web::block(move || {
    templates::main(uri, db, config.get_ref()).ok_or(())
  })
    .await
    .map(|res| HttpResponse::Ok()
      .content_type("text/html; charset=utf-8")
      .body(res.into_string()))
    .map_err(|_| HttpResponse::Forbidden().finish())?;
  println!("profiling: {:?}", std::time::Instant::now().duration_since(t0));
  Ok(res)
}

#[get("/test/{id}")]
async fn test(data: web::Path<u32>, db: DB) -> Result<HttpResponse> {
  // execute sync code in threadpool
  let res = web::block(move || {
    let db = db.get().unwrap();
    db.query_row("select `value` from `main` where `key` = $1", &[data.into_inner()], |row| {
      row.get::<_, String>(0)
    })
  })
    .await
    .map(|value| HttpResponse::Ok().body(value))
    .map_err(|_| HttpResponse::Forbidden())?;
  Ok(res)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  std::env::set_var("RUST_LOG", "actix_web=debug");
  env_logger::init();

  let config = config::load();

  // r2d2 pool
  let manager = r2d2_sqlite::SqliteConnectionManager::file("data/main.db");
  let db_pool = r2d2::Pool::new(manager).unwrap();


  HttpServer::new({
    let config = config.clone();
    move || {
      App::new()
        .data(db_pool.clone())
        .data(config.to_owned())
        .wrap(middleware::Logger::default())
        .wrap(middleware::DefaultHeaders::new().header("content-type", "text/plain; charset=utf-8").content_type())
        .service(views)
        .service(test)
        .service(actix_files::Files::new("/static", "./data/static").use_etag(true))

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
    .bind(config.server.bind_addr)?
    .run()
    .await
}