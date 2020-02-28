use maud::{html, Markup};

pub fn tpl() -> Markup {
  html! {
    .header {
      .container {
        .logo { "Graffiti database" }
        .nav-menu {
          a href="#" { "Home" }
          a href="#" { "Graffiti" }
          a href="#" { "Authors" }
          a href="#" { "Tags" }
          a href="#" { "Help" }
        }
      }
    }
  }
}