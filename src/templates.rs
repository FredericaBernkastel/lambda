use std::error::Error;
use maud::{DOCTYPE, html, Markup};
use rusqlite::params;
use crate::config::Config;
use crate::DB;

pub fn main(uri: String, db: DB, config: &Config) -> Result<Markup, Box<dyn Error>> {
  let db = db.get().unwrap();
  let root_url = &config.web.root_url;
  let page = match uri.as_str() {
    "home" => include!("templates/home.rs"),
    _ => return Err("route not found".into())
  };
  Ok(html! {
    (DOCTYPE)
    html lang="en" {
      head {
        meta http-equiv="Content-Type" content="text/html; charset=utf-8";
        meta name="viewport" content="width=device-width";

        link rel="stylesheet" href={ (root_url) "static/style.css" } type="text/css" media="screen";

        title { "nipaa =^_^=" }
      }
      body {
        ({include!("templates/header.rs")})
        p { (page) }
      }
    }
  })
}