use std::error::Error;
use maud::{DOCTYPE, html, Markup};
use rusqlite::params;
use path_tree::PathTree;
use crate::config::Config;
use crate::DB;
use std::collections::HashMap;

pub fn main(uri: String, db: DB, config: &Config) -> Result<Markup, Box<dyn Error>> {
  let db = db.get().unwrap();
  let root_url = &config.web.root_url;
  lazy_static! {
    static ref PATH_TREE: PathTree::<&'static str> = {
      let mut tmp = PathTree::<&str>::new();
      for path in vec![
        "/home",
        "/graffitis",
        "/graffiti/add",
        "/graffiti/:id",
        "/graffiti/:id/edit",
        "/authors",
        "/author/add",
        "/author/:id",
        "/author/:id/edit",
        "/tags",
        "/help"
      ] { tmp.insert(path, path); };
      tmp
    };
  };

  let page = match PATH_TREE.find(uri.as_str()) {
    Some((path, data)) => {
      let data: HashMap<_, _> = data.into_iter().collect();
      match *path {
        "/home" => include!("templates/home.rs"),
        "/graffitis" => include!("templates/graffitis.rs"),
        "/graffiti/add" => include!("templates/graffiti-add.rs"),// -------
        "/graffiti/:id" => include!("templates/graffiti.rs"),//           |
        "/graffiti/:id/edit" => include!("templates/graffiti-add.rs"),// --
        "/authors" => include!("templates/authors.rs"),
        "/author/add" => include!("templates/author-add.rs"),//------------
        "/author/:id" => include!("templates/author.rs"),//               |
        "/author/:id/edit" => include!("templates/author-add.rs"),// ------
        "/tags" => include!("templates/tags.rs"),
        "/help" => include!("templates/help.rs"),
        _=> unreachable!()
      }
    },
    None => return Err("route not found".into())
  };
  Ok(html! {
    (DOCTYPE)
    html lang="en" {
      head {
        meta http-equiv="Content-Type" content="text/html; charset=utf-8";
        meta name="viewport" content="width=device-width";

        link rel="stylesheet" href={ (root_url) "static/style.css" } type="text/css" media="screen";
        script type="text/javascript" src={ (root_url) "static/vendors.js" } {  }
        script type="text/javascript" src={ (root_url) "static/script.js" } {  }

        title { "nipaa =^_^=" }
      }
      body {
        ({include!("templates/header.rs")})
        p { (page) }
      }
    }
  })
}

fn navigation(config: &Config) -> Markup {
  html! {
    .navigation {
      .n_back { span {
        svg {use xlink:href={ (config.web.root_url) "static/img/sprite.svg#chevron-circle-left" } {  }} "prev"
      } }
      .navi_link {
        span { "1" }
        @for i in 2..11 {
          a href="#" { (i) }
        }
        span.nav_ext { "..." }
        a href="#" { "18" }
      }
      .n_next { a href="#" {
        "next" svg {use xlink:href={ (config.web.root_url) "static/img/sprite.svg#chevron-circle-right" } {  }}
      } }
    }
  }
}