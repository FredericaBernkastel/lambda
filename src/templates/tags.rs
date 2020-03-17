html! {
  (include!("header.rs"))
  
  .page-tags {
    .container {
      .node118 {
        .edit {
          span.action-btn#edit {
            "Modify"
            svg {use xlink:href={ (root_url) "static/img/sprite.svg#edit" }{}}
          }
          input type="text" id="from" placeholder="e.g. \"authorized\"" {  }
          span.arrow { "=>" }
          input type="text" id="to" placeholder="e.g. \"mischief\"" {  }
        }
        .delete {
          .action-btn.red#delete {
            "Delete"
            svg {use xlink:href={ (root_url) "static/img/sprite.svg#trash-alt" }{}}
          }
          input type="text" placeholder="e.g. \"motto\"" {  }
        }
      }
      .node119 {
        p { b { "List of graffiti tags" } }
        .tags {
          @for _ in 1..=10 {
            a href="#" { "vandalism" }
            a href="#" { "political" }
            a href="#" { "motto" }
          }
        }
      }
    }
  }
}