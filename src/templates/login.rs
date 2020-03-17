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
        input#hash type="hidden" value="{hash}";
        p.si-error {  }
        button#submit 
          data-html="login <span class='icon'>fas fa-sign-in-alt</span>"
          data-spinner={"<svg><use xlink:href='" (root_url) "static/img/sprite.svg#spinner'></use></svg>"}
          data-action="admin/login" {
            "login" span.icon { svg {use xlink:href={ (root_url) "static/img/sprite.svg#sign-in-alt" }{}} }
          }
      }
    }
  }
}
