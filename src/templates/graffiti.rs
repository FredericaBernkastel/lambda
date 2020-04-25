{
  let id: u32 = data.get("id").ok_or("")?.parse()?;

  let (graffiti, location) = db.query_row("
    select
      a.id as '0', a.complaint_id as '1', a.datetime as '2', a.shift_time as '3', a.intervening as '4', a.companions as '5', a.notes as '6', a.views as '7',
      b.graffiti_id as '8', b.country as '9', b.city as '10', b.street as '11', b.place as '12', b.property as '13', b.gps_long as '14', b.gps_lat as '15'
    from `graffiti` a
    left join `location` b on b.`graffiti_id` = a.`id`
    where a.`id` = :id", params![id], |row| {
      Ok((
        model::Graffiti {
          id: row.get(0)?,
          complaint_id: row.get(1)?,
          datetime: row.get(2)?,
          shift_time: model::ShiftTime::from_u8(row.get(3)?),
          intervening: row.get(4)?,
          companions: row.get(5)?,
          notes: row.get(6)?,
          views: row.get(7)?
        },
        model::Location {
          graffiti_id: row.get(8)?,
          country: row.get(9)?,
          city: row.get(10)?,
          street: row.get(11)?,
          place: row.get(12)?,
          property: row.get(13)?,
          gps_long: row.get(14)?,
          gps_lat: row.get(15)?
        }
      ))
    })?;

  let images: Vec<String> = db.prepare("
    select hash
      from graffiti_image
     where graffiti_id = :id
     order by `order` asc;
    ")?.query_map(params![id], |row| {
      Ok(row.get(0)?)
    })?.filter_map(Result::ok).collect();


  struct Author {
    id: u32,
    indubitable: bool,
    name: String
  }
  let authors: Vec<Author> = db.prepare("
    select a.author_id,
           a.indubitable,
           b.name
      from graffiti_author a
           inner join author b on a.author_id = b.id
     where graffiti_id = :id")?
    .query_map(params![id], |row| {
      Ok(Author { 
        id: row.get(0)?,
        indubitable: row.get(1)?,
        name: row.get(2)?,
      })
    })?.filter_map(Result::ok).collect();

  // update views, takes 5ms
  db.execute("
    update `graffiti`
      set `views` = `views` + 1
      where `id` = :id", params![id])?;

  let gps = if let (Some(lat), Some(long)) = (location.gps_lat, location.gps_long){
    format!("{}, {}", lat, long)
  } else { "".into() };

  html! {
    (include!("header.rs"))
    
    .page-graffiti {
      .container {
        .actions-wrapper {
          a.action-btn#edit href={ (root_url) "views/graffiti/" (id) "/edit" } {
            "Modify graffiti"
            svg {use xlink:href={ (root_url) "static/img/sprite.svg#edit" }{}}
          }
          span.action-btn.red#delete {
            "Delete"
            svg {use xlink:href={ (root_url) "static/img/sprite.svg#trash-alt" }{}}
          }
        }
        .row1 {
          .node100.boxed {
            p.box-title { "Information" }
            .descr {
              .row { .l { "Graffiti ID: " } .r { (id) } }
              .row { .l { "Complaint ID: " } .r { (graffiti.complaint_id) } }
              .row { .l { "Date: " } .r { (graffiti.datetime.map_or("".into(), |x| util::format_timestamp(x as u64, "%Y-%m-%d"))) } }
              .row { .l { "Time: " } .r { (graffiti.datetime.map_or("".into(), |x| util::format_timestamp(x as u64, "%H:%M"))) } }
              .row { .l { "Shift: " } .r { (graffiti.shift_time.map_or("".into(), |x| x.to_string())) } }
              .row { .l { "Intervening: " } .r { (graffiti.intervening) } }
            }
          }
          a.link-prev href="#" { 
            svg { title { "Previous image" } use xlink:href={ (root_url) "static/img/sprite.svg#angle-double-left" }{}}
          }
          .node102 {
            .graffiti-image {
              @if let Some(image) = images.get(0) {
                img data-id="0" src=(format!("{}static/img/graffiti/{}/{}_p1.jpg", root_url, image.get(0..=1).unwrap_or(""), image));
                .images data-type="x-template" {
                  (json::stringify(images))
                }
              } @else {
                .no-image {  }
              }
            }
            .tags {
              a href="#" { "vandalism" }
              a href="#" { "political" }
              a href="#" { "motto" }
            }
          }
          a.link-next href="#" { 
            svg { title { "Next image" } use xlink:href={ (root_url) "static/img/sprite.svg#angle-double-right" }{}}
          }
          .node101.boxed {
            p.box-title { "Author(s)" }
            .items {
              @for author in authors {
                @let classname = 
                  format!("item {}", if author.indubitable { "checked" } else { "" });
                .(classname) { 
                  svg { title { "checked" } use xlink:href={ (root_url) "static/img/sprite.svg#check" }{}} 
                  a href={ (root_url) "views/author/" (author.id) } { 
                    (author.name) 
                  } 
                }
              }
            }
          }
        }
        .row2 {
          .node104.boxed {
            p.box-title { "Location" }
            .descr {
              .row { .l { "Country: " } .r { (location.country) } }
              .row { .l { "City: " } .r { (location.city) } }
              .row { .l { "Street: " } .r { (location.street) } }
              .row { .l { "Property: " } .r { (location.property) } }
              .row { .l { "Place: " } .r { (location.place) } }
            }
          }
          .node105.boxed {
            p.box-title { "Notes" }
            .descr { (util::markup_br(graffiti.notes)) }
          }
          .node106 {
            .map {  }
            p { 
              "map location" br;
              (gps)
            }
          }
        }
      }
    }
  }
}