{
  let data = db.query_row("select `value` from `main` where `key` = 1", params![], |row| {
    row.get::<_, String>(0)
  }).unwrap_or("".to_string()); // "Graffiti database"

  html! {
    .header {
      .container {
        .logo { (data) }
        .nav-menu {
          a href={ (root_url) "views/home" } { "Home" }
          a href="#" { "Graffiti" }
          a href="#" { "Authors" }
          a href="#" { "Tags" }
          a href="#" { "Help" }
        }
      }
    }
  }
}
