{
  let _id: u32 = data.get("id").ok_or("")?.parse()?;

  html! {
    .container {
      .page-author {
        .actions-wrapper {
          a href="#" {
            span.action-btn#edit {
              "Modify author"
              svg {use xlink:href={ (root_url) "static/img/sprite.svg#edit" }{}}
            }
          }
        }
        .row1 {
          .node113 {
            .node113_1 {
              a.link-prev href="#" { 
                svg { title { "Previous ID" } use xlink:href={ (root_url) "static/img/sprite.svg#angle-double-left" }{}}
              }
              .author-image {
                img;
              }
              a.link-next href="#" { 
                svg { title { "Next ID" } use xlink:href={ (root_url) "static/img/sprite.svg#angle-double-right" }{}}
              }
            }
            .node113_2.boxed {
              p.box-title { "Information" }
              .descr {
                .row { .l { "Name: " }        .r { "Authorname Surname Lastname" } }
                .row { .l { "Age: " }         .r { "23" } }
                .row { .l { "Height: " }      .r { "174cm" } }
                .row { .l { "Handedness: " }  .r { "right handed" } }
                .row { .l { "Home city: " }   .r { "Huesca" } }
                .row { .l { "Graffiti: " }    .r { "1" } }
              }
            }
            .node113_3.boxed {
              p.box-title { "Social networks" }
              .descr {
                a href="#" { "https://instagram.com/username" }
                a href="#" { "https://facebook.com/user.name" }
              }
            }
          }
          .node114 {
            .node114_1.boxed {
              p.box-title { "Notes" }
              .descr {
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean felis sem, placerat convallis 
                 eros quis, mollis venenatis dui. Pellentesque habitant morbi tristique senectus et netus et
                 malesuada fames ac turpis egestas. Nullam pulvinar ac nisl et eleifend. Aliquam commodo 
                 tristique mi, nec fringilla massa egestas vel. Morbi orci eros, ultricies id efficitur vel, 
                 interdum in arcu. Mauris ex dolor, sodales in massa et, tempus cursus urna. Pellentesque 
                 tristique sem hendrerit pretium dapibus."
              }
            }
            .node114_2.boxed {
              p.box-title { "Zones of activity" }
              .descr {
                .row { .l { "Countries: " } .r { "Spain(34), Portugal(3)" } }
                .row { .l { "Cities: " }    .r { "Zamora(21), Valladolid(7), León(6), Lisboa(3)" } }
              }
            }
            .node114_3 {
              .map { }
              p { "Activity map of author" }
            }
          }
          .node115 {
            .node115_1.boxed {
              p.box-title { "Previous companions" }
              .items {
                .item { a href="#" { "Author Name 1" } }
                .item { a href="#" { "Author Name 2" } }
                .item { a href="#" { "Author Name 3" } }
                .item { a href="#" { "Author Name 4" } }
                .item { a href="#" { "Author Name 5" } }
              }
            }
            .node115_2.boxed {
              p.box-title { "Relevant graffiti" }
              .items {
                .item {
                  p { "most recent" }
                  a.img href={ (root_url) "views/graffiti/1" } { img; }
                }
                .item {
                  p { "most viewed" }
                  a.img href={ (root_url) "views/graffiti/1" } { img; }
                }
              }
            }
          }
        }
      }
    }
  }
}