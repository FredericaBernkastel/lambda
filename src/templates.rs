use maud::{DOCTYPE, html, Markup, PreEscaped};
use actix_web::web;
use serde_json::json;
use path_tree::PathTree;
use strum::IntoEnumIterator;
use num_traits::FromPrimitive;
use std::collections::{HashMap, VecDeque};
use lazy_static::lazy_static;
use runtime_fmt::{rt_format, rt_format_args};
use rusqlite::params;
use error_chain::bail;
use crate::{
  error,
  util,
  model,
  web::DB,
  web::Config
};

pub async fn main(uri: String, db: DB, config: Config, user: Option<model::User>) -> error::Result<Markup> {
  let root_url = config.web.root_url.as_str();
  let cors_h = util::gen_cors_hash(util::get_timestamp(), &config);
  lazy_static! {
    static ref PATH_TREE: PathTree::<&'static str> = {
      let mut tmp = PathTree::<&str>::new();
      for path in vec![
        "/login",
        "/home",
        "/graffitis",
        "/graffitis/page/:page",
        "/graffiti/add",
        "/graffiti/:id",
        "/graffiti/:id/edit",
        "/authors",
        "/authors/page/:page",
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
      let data: HashMap<_, _> = data
        .into_iter()
        .map(|(arg, value)| (arg.to_string(), value.to_string())).
        collect();
      __path_t = (path, data.clone());

      match path {
        "/login" => include!("templates/login.rs"),
        _ => {
          let user = match user {
            Some(user) => user,
            None => bail!("unauthorized")
          };
          match path {
            "/home" => include!("templates/home.rs"),

            "/graffitis" => include!("templates/graffitis.rs"),// -------------
            "/graffitis/page/:page" => include!("templates/graffitis.rs"),// --

            "/graffiti/add" => include!("templates/graffiti-add.rs"),// -------
            "/graffiti/:id" => include!("templates/graffiti.rs"),//           |
            "/graffiti/:id/edit" => include!("templates/graffiti-add.rs"),// --

            "/authors" => include!("templates/authors.rs"),// -----------------
            "/authors/page/:page" => include!("templates/authors.rs"),// ------

            "/author/add" => include!("templates/author-add.rs"),//------------
            "/author/:id" => include!("templates/author.rs"),//               |
            "/author/:id/edit" => include!("templates/author-add.rs"),// ------

            "/tags" => include!("templates/tags.rs"),
            "/help" => include!("templates/help.rs"),
            _ => unreachable!()
          }
        }
      }
    },
    None => bail!("route not found")
  };

  let js_glob = json!({
    "path_t": __path_t.0,
    "data": __path_t.1,
    "root_url": root_url,
    "rpc": format!("{}rpc/", root_url),
    "cors_h": cors_h
  });

  Ok(html! {
    (DOCTYPE)
    html lang="en" {
      head {
        meta http-equiv="Content-Type" content="text/html; charset=utf-8";
        meta name="viewport" content="width=device-width";

        link rel="stylesheet" href={ (root_url) "static/vendors.css" } type="text/css" media="screen";
        link rel="stylesheet" href={ (root_url) "static/style.css" } type="text/css" media="screen";
        script type="text/javascript" src={ (root_url) "static/vendors.js" } {  }
        script type="text/javascript" src={ (root_url) "static/script.js" } {  }

        title { "Graffiti database" }

        script type="text/javascript" {
          "var __glob = " (PreEscaped(js_glob.to_string())) ";"
        }
      }
      body {
        (page)
      }
    }
  })
}

fn mar_navigation(config: &Config, link_tpl: &str, current_page: i64, per_page: i64, total: i64) -> error::Result<Markup> {
  let total_pages = (total as f64 / per_page as f64).ceil() as i64;

  if current_page < 1 || current_page > total_pages {
    bail!(error::ErrorKind::InvalidRequest);
  }

  let radius = 4;
  let prev_page = match current_page - 1 {
    x if x > 0 => Some(x),
    _ => None
  };
  let next_page = match current_page + 1 {
    x if x <= total_pages => Some(x),
    _ => None
  };
  let mut pages = VecDeque::<Option<i64>>::new();
  (current_page - radius ..= current_page + radius)
    .filter(|x| *x > 0 && *x <= total_pages)
    .for_each(|x| pages.push_back(Some(x)));
  match current_page - radius - 1 {
    1 => vec![Some(1)],
    x if x > 1 => vec![Some(1), None],
    _ => vec![]
  }
    .into_iter()
    .rev()
    .for_each(|x| pages.push_front(x));
  match -current_page - radius + total_pages {
    1 => vec![Some(total_pages)],
    x if x > 1 => vec![None, Some(total_pages)],
    _ => vec![]
  }
    .into_iter()
    .for_each(|x| pages.push_back(x));

  let link_fmt = |page| rt_format!(link_tpl, config.web.root_url, page)
    .map_err(|_| "invalid format template");

  Ok(html! {
    .navigation {
      .n_back {
        @let svg = html!{ svg {use xlink:href={ (config.web.root_url) "static/img/sprite.svg#chevron-circle-left" } {  }} };
        @match prev_page {
          Some(page) => a href=(link_fmt(page)?) { (svg) "prev" },
          None => span { (svg) "prev" }
        }
      }
      .navi_link {
        @for page in pages {
          @match page {
            Some(page) =>
              @if page != current_page {
                a href=(link_fmt(page)?) { (page) }
              } @else {
                span { (page) }
              },
            None => { span.nav_ext { "..." } }
          }
        }
      }
      .n_next {
        @let svg = html!{ svg {use xlink:href={ (config.web.root_url) "static/img/sprite.svg#chevron-circle-right" } {  }} };
        @match next_page {
          Some(page) => a href=(link_fmt(page)?) { "next" (svg) },
          None => span { "next" (svg) }
        }
      }
    }
  })
}

fn mar_image(hash: Option<&str>, path_template: &str, config: &Config) -> error::Result<Markup> {
  let root_url = &config.web.root_url;
  let src = match hash {
    Some(hash) => rt_format!(path_template, root_url, hash.get(0..=1)?, hash)
      .map_err(|_| "invalid format template")?,
    None => "{src}".into()
  };

  Ok(html! {
    .image data-id=(hash.unwrap_or("")) {
      img src=(src) {  }
      .controls {
        .sh {
          .shl { svg { title { "move left" }  use xlink:href={ (root_url) "static/img/sprite.svg#angle-left" }{}} }
          .shr { svg { title { "move right" } use xlink:href={ (root_url) "static/img/sprite.svg#angle-right" }{}} }
        }
        .del { svg { title { "delete" } use xlink:href={ (root_url) "static/img/sprite.svg#times-circle" }{}} }
      }
      .processing_overlay {
        svg { title { "uploading" } use xlink:href={ (root_url) "static/img/sprite.svg#spinner" }{}}
      }
    }
  })
}