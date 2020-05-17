#![feature(proc_macro_hygiene)]
#![feature(str_strip)]
#![feature(try_trait)]
#![feature(type_ascription)]
#![feature(stmt_expr_attributes)]

mod cli;
mod config;
mod error;
mod schema;
mod util;
mod web;

fn main() {
  std::env::set_var("RUST_LOG", "actix_web=debug");
  std::env::set_var("RUST_BACKTRACE", "1");
  env_logger::init();

  (|| -> error::Result<_> {
    let config = config::load()?;

    // r2d2 pool
    let manager = r2d2_sqlite::SqliteConnectionManager::file(&config.server.db_path);
    let db_pool = r2d2::Pool::new(manager)?;

    cli::load(&config, db_pool.clone())?;

    web::init(config, db_pool)?;

    Ok(())
  })()
  .unwrap();
}
