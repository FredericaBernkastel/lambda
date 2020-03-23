html! {
  .page-login {
    .login {
      p.title { "Graffiti database" }
      .form {
        .input-wrapper {
          .icon { svg {use xlink:href={ (root_url) "static/img/sprite.svg#user" }{}} }
          input#login type="text" placeholder="username";
        }
        .input-wrapper {
          .icon { svg {use xlink:href={ (root_url) "static/img/sprite.svg#key" }{}} }
          input#password type="password" placeholder="password";
        }
        p.si-error {  }
        button#submit 
          data-html=((html! { 
            "login" span.icon { svg {use xlink:href={ (root_url) "static/img/sprite.svg#sign-in-alt" }{}} } 
          }).into_string())
          data-spinner=((html! { 
            svg.fa-spinner {use xlink:href={ (root_url) "static/img/sprite.svg#spinner" }{}}
          }).into_string())
          {
            "login" span.icon { svg {use xlink:href={ (root_url) "static/img/sprite.svg#sign-in-alt" }{}} }
          }
      }
    }
  }
}
