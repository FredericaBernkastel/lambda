{
  let data = db.query_row("select `value` from `main` where `key` = 1", params![], |row| {
    row.get::<_, String>(0)
  })?; // "Graffiti database"

  html! {
    .header {
      .container {
        .logo { (data) }
        .nav-menu {
          a href={ (root_url) "views/home" } { "Home" }
          a href={ (root_url) "views/graffitis" } { "Graffiti" }
          a href={ (root_url) "views/authors" } { "Authors" }
          a href="#" { "Tags" }
          a href="#" { "Help" }
        }
      }
    }
  }
}
