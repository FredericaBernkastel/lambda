use crate::{config, error, util};
use actix_session::{CookieSession, Session};
use actix_web::{
  error::BlockingError, get, guard, middleware, post, web, App, HttpResponse, HttpServer, Result,
};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use serde_json::Value as JsonValue;

pub mod auth;
mod mvc;

pub type DB = web::Data<Pool<SqliteConnectionManager>>;
pub type Config = web::Data<config::Config>;

/*
 * wrapper over actix_web::web::block
 * receive smart pointer on db pool, spawn task, and obtain connection, before passing to the inner closure
 */
pub async fn block<F, I, E>(db_pool: DB, f: F) -> Result<I, BlockingError<E>>
where
  F: FnOnce(PooledConnection<SqliteConnectionManager>) -> Result<I, E> + Send + 'static,
  I: Send + 'static,
  E: Send + std::fmt::Debug + std::convert::From<r2d2::Error> + 'static,
{
  web::block(move || f(db_pool.get()?)).await
}

#[get("/views/{uri:.+}")]
async fn sv_views(
  uri: web::Path<String>,
  db_pool: DB,
  config: Config,
  session: Session,
) -> Result<HttpResponse, error::Error> {
  let uri = util::strip_slashes(uri.to_string());

  let user = auth::get_user(db_pool.clone(), &session).await?;
  match user {
    Some(_) if uri == "/login" => return Ok(util::redirect("views/home", &config)),
    None if uri != "/login" => return Ok(util::redirect("views/login", &config)),
    _ => (),
  }

  mvc::model(uri, db_pool, config, user).await.map(|res| {
    HttpResponse::Ok()
      .content_type("text/html; charset=utf-8")
      .body(res.into_string())
  })
}

#[post("/rpc/{uri:.+}")]
async fn sv_rpc(
  uri: web::Path<String>,
  payload: web::Payload,
  db_pool: DB,
  config: Config,
  session: Session,
) -> Result<HttpResponse, error::Error> {
  let uri = util::strip_slashes(uri.to_string());

  let user = auth::get_user(db_pool.clone(), &session).await?;
  if user.is_none() && (uri != "/auth/login") {
    return Ok(HttpResponse::Unauthorized().finish());
  };

  // parse into untyped
  let post_data =
    serde_json::from_slice::<JsonValue>(util::read_payload(payload, &config).await?.as_ref())?;

  mvc::controller(uri, post_data, db_pool, config, user, session)
    .await
    .map(|res| HttpResponse::Ok().json(res))
}

#[actix_rt::main]
pub async fn init(
  config: config::Config,
  db_pool: Pool<SqliteConnectionManager>,
) -> error::Result<()> {
  let server = HttpServer::new({
    let config = config.clone();
    move || {
      App::new()
        .data(db_pool.to_owned())
        .data(config.to_owned())
        .wrap(middleware::Logger::default())
        .wrap(
          middleware::DefaultHeaders::new()
            .header("content-type", "text/plain; charset=utf-8")
            .content_type(),
        )
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
    }
  });
  let server = if config.server.bind_addr.starts_with("unix:/") {
    #[cfg(target_os = "linux")]
    {
      server.bind_uds(&config.server.bind_addr.strip_prefix("unix:")?)?
    }
    #[cfg(not(target_os = "linux"))]
    error_chain::bail!("Unix sockets are not available for this target");
  } else {
    server.bind(&config.server.bind_addr)?
  };
  println!("Lambda v{}\nListening on {}", env!("CARGO_PKG_VERSION"), config.server.bind_addr);
  server.run().await?;
  Ok(())
}
