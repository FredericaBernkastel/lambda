use actix_web::{get, post, web, guard, App, HttpServer, HttpResponse, Result, middleware};
use actix_session::{CookieSession, Session};
use serde_json::Value as JsonValue;
use error_chain::bail;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use crate::{
  error,
  config,
  auth,
  util,
  rpc,
  templates,
};

pub type DB = web::Data<Pool<SqliteConnectionManager>>;
pub type Config = web::Data<config::Config>;

#[get("/views/{uri:.+}")]
async fn sv_views(uri: web::Path<String>, db: DB, config: Config, session: Session) -> Result<HttpResponse, error::Error> {
  let t0 = std::time::Instant::now();
  let uri = util::strip_slashes(uri.to_string());

  let user = auth::get_user(db.clone(), &session).await?;
  if user.is_none() && (uri != "/login") { return Ok(util::redirect("views/login", &config)); };
  if user.is_some() && (uri == "/login") { return Ok(util::redirect("views/home",  &config)); };

  let res =
    templates::main(uri, db, config, user)
      .await
      .map(|res| HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(res.into_string()))?;

  println!("profiling: {:?}", std::time::Instant::now().duration_since(t0));
  Ok(res)
}

#[post("/rpc/{uri:.+}")]
async fn sv_rpc(uri: web::Path<String>, payload: web::Payload, db: DB, config: Config, session: Session) -> Result<HttpResponse, error::Error> {
  let t0 = std::time::Instant::now();
  let uri = util::strip_slashes(uri.to_string());

  let user = auth::get_user(db.clone(), &session).await?;
  if user.is_none() && (uri != "/auth/login") { return Ok(HttpResponse::Unauthorized().finish()); };

  // parse into untyped
  let post_data = serde_json::from_slice::<JsonValue>(
    util::read_payload(payload, config.get_ref())
      .await?
      .as_ref()
  )?;

  let res = rpc::main(uri, post_data, db, config.get_ref(), user, session)
    .await
    .map(|res| HttpResponse::Ok().json(res));

  println!("profiling: {:?}", std::time::Instant::now().duration_since(t0));
  res
}

#[actix_rt::main]
pub async fn init(config: config::Config, db_pool: Pool<SqliteConnectionManager>) -> error::Result<()> {

  let server = HttpServer::new({
    let config = config.clone();
    move || {
      App::new()
        .data(db_pool.to_owned())
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
      server.bind_uds(config.server.bind_addr.strip_prefix("unix:")?)?
        .run().await?;
    }
    #[cfg(not(target_os = "linux"))]
    bail!("Unix sockets are not available for this target");
  } else {
    server.bind(config.server.bind_addr)?
      .run().await?;
  }
  Ok(())
}