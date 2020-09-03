#![feature(proc_macro_hygiene)]
#![feature(try_trait)]
#![feature(type_ascription)]
#![feature(stmt_expr_attributes)]
#![feature(box_syntax)]

mod cli;
mod config;
mod error;
mod schema;
mod util;
mod web;

fn main() {
  (|| -> error::Result<_> {
    let cli_data = cli::load()?;

    let config = config::load(&cli_data.config_path)
      .map_err(|_| format!("Unable to load config file: {}", cli_data.config_path))?;
    for [k, v] in &config.server.env_vars {
      std::env::set_var(k, v);
    }

    env_logger::init();

    // r2d2 pool
    let manager = r2d2_sqlite::SqliteConnectionManager::file(&config.server.db_path);
    let db_pool = r2d2::Pool::new(manager)?;

    (*cli_data.callback)(&config, db_pool.clone())?;

    web::init(config, db_pool)?;

    Ok(())
  })()
  .unwrap();
}
