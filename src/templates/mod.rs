use maud::{DOCTYPE, html, Markup};

mod header;
mod home;

pub fn main(uri: String) -> Option<Markup> {
  let page = match uri.as_str() {
    "home" => home::tpl(),
    _ => return None
  };
  Some(html! {
    (DOCTYPE)
    html lang="en" {
      head {
        meta http-equiv="Content-Type" content="text/html; charset=utf-8";
        meta name="viewport" content="width=device-width";

        link rel="stylesheet" href="/static/style.css" type="text/css" media="screen";

        title { "nipaa =^_^=" }
      }
      body {
        (header::tpl())
        p { (page) }
      }
    }
  })
}