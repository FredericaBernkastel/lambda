html! {
  .container {
    .page-home {
      .node101 {
        .node103.boxed {
          p.box-title { "Most recent additions" }
          .images {
            @for i in 1..9 {
              a href={ (root_url) "views/graffiti/" (i) } {
                .image {}
                p.title { "Graffiti image" }
              }
            }
          }
        }
        .node103.boxed {
          p.box-title { "Last checked graffiti" }
          .images {
            @for i in 1..5 {
              a href={ (root_url) "views/graffiti/" (i) } {
                .image {}
                p.title { "Graffiti image" }
              }
            }
          }
        }
      }
      .node102 {
        .node104.boxed {
          p.box-title { "Recent activity map" }
          .map {}
          p { "Map location" }
        }
        .node105.boxed {
          p.box-title { "Last checked authors" }
          .authors {
            a href={ (root_url) "views/author/1" } { "AuthrorName1" }
            a href={ (root_url) "views/author/1" } { "AuthrorName2" }
            a href={ (root_url) "views/author/1" } { "AuthrorName3" }
            a href={ (root_url) "views/author/1" } { "AuthrorName4" }
            a href={ (root_url) "views/author/1" } { "AuthrorName5" }
            a href={ (root_url) "views/author/1" } { "AuthrorName6" }
          }
        }
      }
    }
  }
}