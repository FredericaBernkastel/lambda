use std::process::exit;
use crate::auth;

pub fn load(config: &crate::config::Config, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>){

  let matches = clap_app!(lambda =>
    (@subcommand register =>
      (@arg user: -u --user +takes_value)
      (@arg password: -p --password +takes_value)
    )
    (@subcommand help => )
  ).help(
    r#"USAGE (cli): <command> <opts>

Commmands:
register          register new user
  -u, --user        user login
  -p, --password    user password

help        print help message
"#)
  .get_matches();

  let db = db.get().unwrap();

  (|| -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {

      /*** register ***/
      ("register", Some(command)) => {
        let login = value_t!(command, "user", String)?;
        let password = value_t!(command, "password", String)?;

        if db.prepare("select `id` from `users` where `login` = :login")?
          .exists(params![login])? {
          return Err("error: user already exists".into());
        }

        let hash = auth::password_hash(password, config);

        db.prepare("insert into `users` (`login`, `password`) values (:login, :password)")?
          .insert(params![login, hash])?;

        exit(0);
      },

      _ => ()
    };
    Ok(())
  })()
    .map_err(|e| {
      eprintln!("{}", e);
      exit(0);
    }).ok();
}