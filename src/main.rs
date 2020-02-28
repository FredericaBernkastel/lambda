#![feature(proc_macro_hygiene)]
mod templates;
use actix_web::{get, web, guard, App, HttpServer, HttpResponse, Result, middleware};
type DB = web::Data<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>>;

#[get("/views/{uri:.+}")]
async fn views(uri: web::Path<String>) -> HttpResponse {
  let mut uri = uri.to_string();
  if uri.ends_with('/') { uri.pop(); }

  match templates::main(uri) {
    Some(res) => HttpResponse::Ok()
      .content_type("text/html; charset=utf-8")
      .body(res.into_string()),
    None => HttpResponse::Forbidden().finish()
  }
}

#[get("/test/{id}")]
async fn test(data: web::Path<u32>, db: DB) -> Result<HttpResponse> {
  // execute sync code in threadpool
  let res = web::block(move || {
    let conn = db.get().unwrap();
    conn.query_row("select `value` from `main` where `key` = $1", &[data.into_inner()], |row| {
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

  // r2d2 pool
  let manager = r2d2_sqlite::SqliteConnectionManager::file("data/main.db");
  let db_pool = r2d2::Pool::new(manager).unwrap();

  HttpServer::new(move|| {
    App::new()
      .data(db_pool.clone())
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
  })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}