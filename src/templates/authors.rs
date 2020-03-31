{
  struct Row {
    id: u32,
    name: String,
    age: Option<u32>,
    graffiti: u32,
    home_city: String,
    views: u32,
    thumbnail: Option<String>
  }

  let mut stmt = db.prepare("
    select a.id as `0`,
           a.name as `1`,
           a.age as `2`,
           a.home_city as `3`,
           a.views as `4`,
           b.hash as `5`
      from author a
           left join author_image b on b.author_id = a.id and 
                                       b.`order` = 0
     order by a.id desc
     limit 0, 20"
  )?;
  let authors = stmt.query_map(params![], |row| {
    Ok(Row {
      id: row.get(0)?,
      name: row.get(1)?,
      age: row.get(2)?,
      graffiti: 0, // TODO: (aggregate)
      home_city: row.get(3)?,
      views: row.get(4)?,
      thumbnail: row.get(5)?
    })
  })?.filter_map(Result::ok);

  html! {
    (include!("header.rs"))
    
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
          @for author in authors {
            .row {
              .col1 { (author.id) }
              .col2 { 
                a.graffiti-img href={ (root_url) "views/author/" (author.id) } { 
                  @if let Some(thumbnail) = author.thumbnail {
                    img src=(format!("{}static/img/author/{}/{}_p2.jpg", root_url, thumbnail.get(0..=1).unwrap_or(""), thumbnail));
                  } @else {
                    .no-image {  }
                  }
                } 
              }
              .col3 { (author.name) }
              .col4 { (author.age.map_or("".into(), |x| x.to_string())) }
              .col5 { (author.graffiti) }
              .col6 { (author.home_city) }
              .col7 { (author.views) }
            }
          }
        }
        (navigation(config))
      }
    }
  }
}