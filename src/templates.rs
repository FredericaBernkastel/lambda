use std::error::Error;
use maud::{DOCTYPE, html, Markup, PreEscaped};
use path_tree::PathTree;
use strum::IntoEnumIterator;
use num_traits::FromPrimitive;
use std::collections::HashMap;
use crate::{util, config::Config, model, DBConn as DB};

pub fn main(uri: String, db: DB, config: &Config, user: Option<model::User>) -> Result<Markup, Box<dyn Error>> {
  let root_url = config.web.root_url.as_str();
  let cors_h = util::gen_cors_hash(util::get_timestamp(), config);
  lazy_static! {
    static ref PATH_TREE: PathTree::<&'static str> = {
      let mut tmp = PathTree::<&str>::new();
      for path in vec![
        "/login",
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

  let __path_t;
  let page = match PATH_TREE.find(uri.as_str()) {
    Some((path, data)) => {
      let path = *path;
      let data: HashMap<_, _> = data.into_iter().collect();
      __path_t = (path, data.clone());
      if path == "/login" {
        include!("templates/login.rs")
      } else {
        let user = match user {
          Some(user) => user,
          None => return Err("unauthorized".into())
        };

        match path {
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
          _ => unreachable!()
        }
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

        script type="text/javascript" {
          "var __glob = " (PreEscaped((object!{
            path_t: __path_t.0,
            data: __path_t.1,
            root_url: root_url,
            rpc: format!("{}rpc/", root_url),
            cors_h: cors_h
          }).dump())) ";"
        }
      }
      body {
        (page)
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