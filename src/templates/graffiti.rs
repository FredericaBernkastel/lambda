{
  let id: u32 = data.get("id").ok_or("")?.parse()?;

  html! {
    .page-graffiti {
      .container {
        .actions-wrapper {
          a href={ (root_url) "views/graffiti/" (id) "/edit" } {
            span.action-btn#edit {
              "Modify graffiti"
              svg {use xlink:href={ (root_url) "static/img/sprite.svg#edit" }{}}
            }
          }
        }
        .row1 {
          .node100.boxed {
            p.box-title { "Information" }
            .descr {
              .row { .l { "Graffiti ID: " } .r { (id) } }
              .row { .l { "Complaint ID: " } .r { "0000/0000" } }
              .row { .l { "Date: " } .r { "2018-01-01" } }
              .row { .l { "Time: " } .r { "00:00" } }
              .row { .l { "Shift: " } .r { "Night" } }
              .row { .l { "Intervening: " } .r { "Firemen" } }
            }
          }
          a.link-prev href="#" { 
            svg { title { "Previous ID" } use xlink:href={ (root_url) "static/img/sprite.svg#angle-double-left" }{}}
          }
          .node102 {
            .graffiti-image {
              img;
            }
            .tags {
              a href="#" { "vandalism" }
              a href="#" { "political" }
              a href="#" { "motto" }
            }
          }
          a.link-next href="#" { 
            svg { title { "Next ID" } use xlink:href={ (root_url) "static/img/sprite.svg#angle-double-right" }{}}
          }
          .node101.boxed {
            p.box-title { "Author(s)" }
            .items {
              .item { svg { title { "checked" } use xlink:href={ (root_url) "static/img/sprite.svg#check" }{}} a href={ (root_url) "views/author/1" } { "Author Name 1" } }
              .item.checked { svg { title { "checked" } use xlink:href={ (root_url) "static/img/sprite.svg#check" }{}} a href={ (root_url) "views/author/1" } { "Author Name 2" } }
              .item { svg { title { "checked" } use xlink:href={ (root_url) "static/img/sprite.svg#check" }{}} a href={ (root_url) "views/author/1" } { "Author Name 3" } }
              .item.checked { svg { title { "checked" } use xlink:href={ (root_url) "static/img/sprite.svg#check" }{}} a href={ (root_url) "views/author/1" } { "Author Name 4" } }
              .item { svg { title { "checked" } use xlink:href={ (root_url) "static/img/sprite.svg#check" }{}} a href={ (root_url) "views/author/1" } { "Author Name 5" } }
            }
          }
        }
        .row2 {
          .node104.boxed {
            p.box-title { "Location" }
            .descr {
              .row { .l { "Country: " } .r { "Spain" } }
              .row { .l { "City: " } .r { "Zamora" } }
              .row { .l { "Street: " } .r { "Organization responsible for reporting the location" } }
              .row { .l { "Property: " } .r { "Stadium" } }
              .row { .l { "Place: " } .r { "Lorem ipsum" } }
            }
          }
          .node105.boxed {
            p.box-title { "Notes" }
            .descr {
              "Lorem ipsum dolor sit amet, consectetur adipiscing elit, 
              sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. 
              Ut enim ad minim veniam"
            }
          }
          .node106 {
            .map {  }
            p { "map location" }
          }
        }
      }
    }
  }
}