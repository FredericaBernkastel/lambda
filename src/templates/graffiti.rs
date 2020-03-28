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
              img;
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