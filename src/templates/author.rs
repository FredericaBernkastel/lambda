{
  let id: u32 = data.get("id").ok_or("")?.parse()?;

  let author = db.query_row("
    select name as `0`,
           age as `1`,
           height as `2`,
           handedness as `3`,
           home_city as `4`,
           social_networks as `5`,
           notes as `6`,
           views as `7`
      from author
     where id = :id", params![id], |row| {
      Ok(model::Author {
        id: id,
        name: row.get(0)?,
        age: row.get(1)?,
        height: row.get(2)?,
        handedness: model::Handedness::from_u8(row.get(3)?),
        home_city: row.get(4)?,
        social_networks: row.get(5)?,
        notes: row.get(6)?,
        views: row.get(7)?
      })
    })?;

  let images: Vec<String> = db.prepare("
    select hash
      from author_image
     where author_id = :id
     order by `order` asc;
    ")?.query_map(params![id], |row| {
      Ok(row.get(0)?)
    })?.filter_map(Result::ok).collect();

  // update views, takes 5ms
  db.execute("
    update `author`
      set `views` = `views` + 1
      where `id` = :id", params![id])?;

  html! {
    (include!("header.rs"))
    
    .page-author {
      .container {
        .actions-wrapper {
          a.action-btn#edit href={ (root_url) "views/author/" (id) "/edit" } {
            "Modify author"
            svg {use xlink:href={ (root_url) "static/img/sprite.svg#edit" }{}}
          }
          span.action-btn.red#delete {
            "Delete"
            svg {use xlink:href={ (root_url) "static/img/sprite.svg#trash-alt" }{}}
          }
        }
        .row1 {
          .node113 {
            .node113_1 {
              a.link-prev href="#" { 
                svg { title { "Previous image" } use xlink:href={ (root_url) "static/img/sprite.svg#angle-double-left" }{}}
              }
              .author-image {
                @if let Some(image) = images.get(0) {
                  img data-id="0" src=(format!("{}static/img/author/{}/{}_p1.jpg", root_url, image.get(0..=1).unwrap_or(""), image));
                  .images data-type="x-template" {
                    (json::stringify(images))
                  }
                } @else {
                  .no-image {  }
                }
              }
              a.link-next href="#" { 
                svg { title { "Next image" } use xlink:href={ (root_url) "static/img/sprite.svg#angle-double-right" }{}}
              }
            }
            .node113_2.boxed {
              p.box-title { "Information" }
              .descr {
                .row { .l { "Name: " }        .r { (author.name) } }
                .row { .l { "Age: " }         .r { (author.age.map_or("".into(), |x| x.to_string())) } }
                .row { .l { "Height: " }      .r { (author.height.map_or("".into(), |x| format!("{}cm", x))) } }
                .row { .l { "Handedness: " }  .r { (author.handedness.map_or("".into(), |x| x.to_string())) } }
                .row { .l { "Home city: " }   .r { (author.home_city) } }
                .row { .l { "Graffiti: " }    .r { "0" } } // TODO: (aggregate)
              }
            }
            .node113_3.boxed {
              p.box-title { "Social networks" }
              .descr {
                @for line in author.social_networks.lines() {
                  a href=(line) target="_blank" { (line) }
                }
              }
            }
          }
          .node114 {
            .node114_1.boxed {
              p.box-title { "Notes" }
              .descr { (util::markup_br(author.notes)) }
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