use maud::{html, Markup};

pub fn tpl() -> Markup {
  html! {
    .container page="home" {
      .node100 {
        .node101 {
          .node103.boxed {
            p.box-title { "Most recent additions" }
            .images {
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
            }
          }
          .node103.boxed {
            p.box-title { "Last checked graffiti" }
            .images {
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
              div {
                .image {  }
                p.title { "Graffiti image" }
              }
            }
          }
        }
        .node102 {
          .node104.boxed {
            p.box-title { "Recent activity map" }
            .map {  }
            p { "Map location" }
          }
          .node105.boxed {
            p.box-title { "Last checked authors" }
            .authors {
              a href="#" { "AuthrorName1" }
              a href="#" { "AuthrorName2" }
              a href="#" { "AuthrorName3" }
              a href="#" { "AuthrorName4" }
              a href="#" { "AuthrorName5" }
              a href="#" { "AuthrorName6" }
            }
          }
        }
      }
    }
  }
}