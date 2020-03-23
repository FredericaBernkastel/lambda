{
  /*let data = db.query_row("select `value` from `main` where `key` = 1", params![], |row| {
    row.get::<_, String>(0)
  })?; */

  html! {
    .header {
      .container {
        .logo { "Graffiti database" }
        .nav-menu {
          .pages {
            a href={ (root_url) "views/home" } { "Home" }
            a href={ (root_url) "views/graffitis" } { "Graffiti" }
            a href={ (root_url) "views/authors" } { "Authors" }
            a href={ (root_url) "views/tags" } { "Tags" }
            a href={ (root_url) "views/help" } { "Help" }
          }
          .user {
            svg.icon-user {use xlink:href={ (root_url) "static/img/sprite.svg#user" }{}}
            span.login { (user.login) }
            svg.logout { title { "logout" } use xlink:href={ (root_url) "static/img/sprite.svg#sign-out-alt" }{}}
          }
        }
      }
    }
  }
}
