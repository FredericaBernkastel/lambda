{
  struct Row {
    id: u32,
    datetime: Option<i64>,
    views: u32,
    city: String,
    thumbnail: Option<String>
  }

  let mut stmt = db.prepare("
    select a.id as `0`,
           a.datetime as `1`,
           a.views as `2`,
           b.city as `3`,
           c.hash as `4`
      from graffiti a
           left join location b on b.graffiti_id = a.id
           left join graffiti_image c on c.graffiti_id = a.id and 
                                         c.`order` = 0
     order by a.id desc
     limit 0, 20"
  )?;
  let graffitis = stmt.query_map(params![], |row| {
    Ok(Row {
      id: row.get(0)?,
      datetime: row.get(1)?,
      views: row.get(2)?,
      city: row.get(3)?,
      thumbnail: row.get(4)?,
    })
  })?.filter_map(Result::ok);

  html! {
    (include!("header.rs"))
    
    .page-graffitis {
      .container {
        .actions-wrapper {
          a href={ (root_url) "views/graffiti/add" } {
            span.action-btn#add-graffiti {
              "Add new graffiti"
              svg {use xlink:href={ (root_url) "static/img/sprite.svg#plus" }{}}
            }
          }
        }
        (navigation(config))
        .table {
          .row.head {
            .col1 { "ID" }
            .col2 { "Graffiti" }
            .col3 { "City" }
            .col4 { "Date" }
            .col5 { "Views" }
          }
          @for graffiti in graffitis {
            .row {
              .col1 { (graffiti.id) }
              .col2 { 
                a.graffiti-img href={ (root_url) "views/graffiti/" (graffiti.id) } { 
                  @if let Some(thumbnail) = graffiti.thumbnail {
                    img src=(format!("{}static/img/graffiti/{}/{}_p2.jpg", root_url, thumbnail.get(0..=1).unwrap_or(""), thumbnail));
                  } @else {
                    .no-image {  }
                  }
                }
              }
              .col3 { (graffiti.city) }
              .col4 { (graffiti.datetime.map_or("".into(), |x| util::format_timestamp(x as u64, "%Y-%m-%d"))) }
              .col5 { (graffiti.views) }
            }
          }
        }
        (navigation(config))
      }
    }
  }
}