{
  struct Row {
    id: u32,
    thumbnail: Option<String>
  }

  let mut stmt = db.prepare("
    select a.id as `0`,
           b.hash as `1`
      from graffiti a
           left join graffiti_image b on b.graffiti_id = a.id and 
                                         b.`order` = 0
     order by a.id desc
     limit 0, 20"
  )?;

  let graffitis = stmt.query_map(params![], |row| {
    Ok(Row {
      id: row.get(0)?,
      thumbnail: row.get(1)?,
    })
  })?.filter_map(Result::ok);

  html! {
    (include!("header.rs"))
    
    .page-home {
      .container {
        .node101 {
          .node103.boxed {
            p.box-title { "Most recent additions" }
            .images {
              @for graffiti in graffitis {
                a href={ (root_url) "views/graffiti/" (graffiti.id) } {
                  .image {
                    @if let Some(thumbnail) = graffiti.thumbnail {
                      img src=(format!("{}static/img/graffiti/{}/{}_p1.jpg", root_url, thumbnail.get(0..=1).unwrap_or(""), thumbnail));
                    } @else {
                      .no-image {  }
                    }
                  }
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
                  .image {
                    .no-image {  }
                  }
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
}