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
          data-html={"login<span class='icon'><svg><use xlink:href='" (root_url) "static/img/sprite.svg#sign-in-alt'></use></svg></span>"}
          data-spinner={"<svg class='fa-spinner'><use xlink:href='" (root_url) "static/img/sprite.svg#spinner'></use></svg>"}
          data-action="login" {
            "login" span.icon { svg {use xlink:href={ (root_url) "static/img/sprite.svg#sign-in-alt" }{}} }
          }
      }
    }
  }
}
