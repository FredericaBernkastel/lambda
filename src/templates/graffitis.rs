html! {
  .container {
    .page-graffiti {
      .actions-wrapper {
        a href="#" {
          span.action-btn#add-graffiti {
            "Add new graffiti"
            svg {use xlink:href={ (root_url) "static/img/sprite.svg#plus" } {  }}
          }
        }
      }
      .table {
        .row.head {
          .col1 { "ID" }
          .col2 { "Graffiti" }
          .col3 { "City" }
          .col4 { "Date" }
          .col5 { "Views" }
        }
        @for i in 1..21 {
          .row {
            .col1 { (i) }
            .col2 { a.graffiti-img href={ (root_url) "views/graffiti/" (i) } { img { } } }
            .col3 { "City name" }
            .col4 { "2018-01-01" }
            .col5 { "1" }
          }
        }
      }
      (navigation(config))
    }
  }
}