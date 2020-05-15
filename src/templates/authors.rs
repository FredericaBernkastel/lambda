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

  let page: i64 = data.get("page").unwrap_or(&"1".into()).parse()?;

  let (authors, mar_navigation) = web::block({
    let db = db.get()?;
    let config = config.clone();
    move || -> error::Result<_> {
      let total = db.query_row("select count(*) from author", params![], |row| {
        Ok(row.get::<_, u32>(0)?)
      })?;
      Ok((

        // authors
        db.prepare("
          with sub1 as (
            select id,
                   name,
                   age,
                   home_city,
                   views
              from author
             order by id desc
             limit :page * :limit, :limit
          )
          select sub1.id as `0`,
                 sub1.name as `1`,
                 sub1.age as `2`,
                 sub1.home_city as `3`,
                 sub1.views as `4`,
                 a.hash as `5`,
                 count(b.author_id) as `6`
            from sub1
                 left join author_image a on a.author_id = sub1.id and
                                             a.`order` = 0
                 left join graffiti_author b on b.author_id = sub1.id
           group by sub1.id
           order by sub1.id desc"
        )?.query_map(
          params![page - 1, config.web.rows_per_page],
          |row| {
            Ok(Row {
              id: row.get(0)?,
              name: row.get(1)?,
              age: row.get(2)?,
              home_city: row.get(3)?,
              views: row.get(4)?,
              thumbnail: row.get(5)?,
              graffiti: row.get(6)?,
            })
        })?.filter_map(Result::ok).collect(): Vec<Row>,

        // mar_navigation
        mar_navigation(
          &config,
          "{}views/authors/page/{}",
          page,
          config.web.rows_per_page as i64,
          total as i64
        )?
      ))
    }
  }).await?;

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
        (mar_navigation)
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
        (mar_navigation)
      }
    }
  }
}