{
  struct Row {
    id: u32,
    thumbnail: Option<String>
  }

  let (graffitis_recent, graffitis_last_checked, authors_last_checked) = web::block({
    let db = db.get()?;

    move || -> error::Result<_> {
      let mut stmt = db.prepare("
        select a.id as `0`,
               b.hash as `1`
          from graffiti a
               left join graffiti_image b on b.graffiti_id = a.id and
                                             b.`order` = 0
         order by a.id desc
         limit 0, 8"
      )?;
      let graffitis_recent: Vec<Row> = stmt.query_map(params![], |row| {
        Ok(Row {
          id: row.get(0)?,
          thumbnail: row.get(1)?,
        })
      })?.filter_map(Result::ok).collect();


      let mut stmt = db.prepare("
        select a.id as `0`,
               b.hash as `1`
          from graffiti a
               left join graffiti_image b on b.graffiti_id = a.id and
                                             b.`order` = 0
         order by a.last_viewed desc
         limit 0, 4
      ")?;
      let graffitis_last_checked: Vec<Row> = stmt.query_map(params![], |row| {
        Ok(Row {
          id: row.get(0)?,
          thumbnail: row.get(1)?,
        })
      })?.filter_map(Result::ok).collect();


      let mut stmt = db.prepare("
        select id as `0`,
               name as `1`
          from author
         order by last_viewed desc
         limit 0, 6
      ")?;
      let authors_last_checked: Vec<(u32, String)> = stmt.query_map(params![], |row| {
        Ok((
          row.get(0)?,
          row.get(1)?,
        ))
      })?.filter_map(Result::ok).collect();

      Ok((graffitis_recent, graffitis_last_checked, authors_last_checked))
    }
  }).await?;

  html! {
    (include!("header.rs"))

    .page-home {
      .container {
        .node101 {
          .node103.boxed {
            p.box-title { "Most recent additions" }
            .images {
              @for graffiti in graffitis_recent {
                a href={ (root_url) "views/graffiti/" (graffiti.id) } {
                  .image {
                    @if let Some(thumbnail) = graffiti.thumbnail {
                      img src=(format!("{}static/img/graffiti/{}/{}_p1.jpg", root_url, thumbnail.get(0..=1).unwrap_or(""), thumbnail));
                    } @else {
                      .no-image {  }
                    }
                  }
                }
              }
            }
          }
          .node103.boxed {
            p.box-title { "Last checked graffiti" }
            .images {
              @for graffiti in graffitis_last_checked {
                a href={ (root_url) "views/graffiti/" (graffiti.id) } {
                  .image {
                    @if let Some(thumbnail) = graffiti.thumbnail {
                      img src=(format!("{}static/img/graffiti/{}/{}_p1.jpg", root_url, thumbnail.get(0..=1).unwrap_or(""), thumbnail));
                    } @else {
                      .no-image {  }
                    }
                  }
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
              @for (id, name) in authors_last_checked {
                a href={ (root_url) "views/author/" (id) } { (name) }
              }
            }
          }
        }
      }
    }
  }
}