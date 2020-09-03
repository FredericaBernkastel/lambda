use crate::{error::Result, web::auth, config::Config};
use clap::{clap_app, value_t};
use error_chain::bail;
use rusqlite::params;
use std::process::exit;

pub struct CLIRet {
  pub config_path: String,
  pub callback: Box<dyn Fn(&Config, r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>) -> Result<()>>
}

pub fn load() -> Result<CLIRet>
{
  let matches = clap_app!(lambda =>
    (@arg CONFIG: --config +takes_value)
    (@subcommand register =>
      (@arg user: -u --user +takes_value)
      (@arg password: -p --password +takes_value)
    )
    (@subcommand help => )
  )
  .help(
    r#"USAGE (cli): [vars] [<command> <opts>]
Variables:
  --config    sets a custom config file location

Commmands:
register          register new user
  -u, --user        user login
  -p, --password    user password

help        print help message
"#,
  )
  .get_matches();

  Ok(CLIRet {
    config_path: matches
      .value_of("CONFIG")
      .unwrap_or("data/config.toml")
      .to_owned(),
    callback: match matches.subcommand() {
      /*** register ***/
      ("register", Some(command)) => {
        let login = value_t!(command, "user", String)?;
        let password = value_t!(command, "password", String)?;

        box move |config, db| {
          let db = db.get()?;

          if db
            .prepare("select `id` from `users` where `login` = :login")?
            .exists(params![login])?
          {
            bail!("user already exists");
          }

          let hash = auth::password_hash(&password, config);

          db.prepare("insert into `users` (`login`, `password`) values (:login, :password)")?
            .insert(params![login, hash])?;

          exit(0);
        }
      },
      _ => box |_, _| Ok(()),
    }
  })
}
