{
  struct Row {
    id: u32,
    datetime: Option<i64>,
    views: u32,
    city: String,
    thumbnail: Option<String>
  }

  let page: i64 = data.get("page").unwrap_or(&"1".into()).parse()?;

  let (graffitis, mar_navigation) = web::block({
    let db = db.get()?;
    let config = config.clone();

    move || -> error::Result<_> {
      let total = db.query_row("select count(*) from graffiti", params![], |row| {
        Ok(row.get::<_, u32>(0)?)
      })?;

      let mar_navigation = mar_navigation(
        &config,
        "{}views/graffitis/page/{}",
        page,
        config.web.rows_per_page as i64,
        total as i64
      )?;

      let mut stmt = db.prepare("
        with sub1 as (
          select id,
                 datetime,
                 views
            from graffiti
           order by id desc
           limit :page * :limit, :limit
        )
        select sub1.id as `0`,
               sub1.datetime as `1`,
               sub1.views as `2`,
               a.city as `3`,
               b.hash as `4`
          from sub1
               left join location a on a.graffiti_id = sub1.id
               left join graffiti_image b on b.graffiti_id = sub1.id and
                                             b.`order` = 0"
      )?;
      let graffitis: Vec<Row> = stmt.query_map(
        params![page - 1, config.web.rows_per_page],
        |row| {
          Ok(Row {
            id: row.get(0)?,
            datetime: row.get(1)?,
            views: row.get(2)?,
            city: row.get(3)?,
            thumbnail: row.get(4)?,
          })
        })?.filter_map(Result::ok).collect();
      Ok((graffitis, mar_navigation))
    }
  }).await?;

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
        (mar_navigation)
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
        (mar_navigation)
      }
    }
  }
}