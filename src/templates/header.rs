{
  html! {

    .popup-wrapper#error {
      .popup {
        p.title { "Error!" }
        .inner { 
          .message {  }
          .actions-wrapper { 
            span.action-btn#close { "Ok" }
          }
        }
      }
    }

    .popup-wrapper#warning {
      .popup {
        p.title { 
          svg {use xlink:href={ (root_url) "static/img/sprite.svg#exclamation-triangle" }{}} }
        .inner { 
          .message {  } 
          .actions-wrapper {
            span.action-btn.red#ok { "Ok" }
            span.action-btn#cancel { "Cancel" }
          }
        }
      }
    }

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
