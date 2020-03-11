html! {
  .page-authors {
    .container {
      .actions-wrapper {
        a href={ (root_url) "views/author/add" } {
          span.action-btn#add-author {
            "Add new author"
            svg {use xlink:href={ (root_url) "static/img/sprite.svg#plus" }{}}
          }
        }
      }
      (navigation(config))
      .table {
        .row.head {
          .col1 { "ID" }
          .col2 { "Image" }
          .col3 { "Name" }
          .col4 { "Age" }
          .col5 { "Graffiti" }
          .col6 { "Home city" }
          .col7 { "Views" }
        }
        @for i in (1..=20).rev() {
          .row {
            .col1 { (i) }
            .col2 { a.graffiti-img href={ (root_url) "views/author/" (i) } { img; } }
            .col3 { "Name Surname Lastname" }
            .col4 { "23" }
            .col5 { "1" }
            .col6 { "Madrid" }
            .col7 { "1" }
          }
        }
      }
      (navigation(config))
    }
  }
}