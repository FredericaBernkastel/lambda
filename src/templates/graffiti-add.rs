{

  struct Graffiti {
    id: String,
    complaint_id: String,
    date: String,
    time: String,
    shift_time: model::ShiftTime,
    intervening: String,
    notes: String
  }

  struct Location {
    country: String,
    city: String,
    street: String,
    place: String,
    property: String,
    gps: String,
  }

  let (graffiti, location) =

    if path == "/graffiti/:id/edit" {
      let id: u32 = data.get("id").ok_or("")?.parse()?;

      db.query_row("
      select
        a.complaint_id as '0', a.datetime as '1', a.shift_time as '2', a.intervening as '3', a.notes as '4',
        b.country as '5', b.city as '6', b.street as '7', b.place as '8', b.property as '9', b.gps_long as '10', b.gps_lat as '11'
      from `graffiti` a
      left join `location` b on b.`graffiti_id` = a.`id`
      where a.`id` = :id", params![id], |row| {
        Ok((
          Graffiti {
            id: id.to_string(),
            complaint_id: row.get(0)?,
            date: row.get::<_, Option<i64>>(1)?.map_or("".into(), |x| util::format_timestamp(x as u64, "%Y-%m-%d")),
            time: row.get::<_, Option<i64>>(1)?.map_or("".into(), |x| util::format_timestamp(x as u64, "%H:%M")),
            shift_time: model::ShiftTime::from_u8(row.get(2)?).unwrap_or(model::ShiftTime::Afternoon),
            intervening: row.get(3)?,
            notes: row.get(4)?,
          },
          Location {
            country: row.get(5)?,
            city: row.get(6)?,
            street: row.get(7)?,
            place: row.get(8)?,
            property: row.get(9)?,
            gps:  if let (Some(lat), Some(long)) = (row.get::<_, Option<f64>>(11)?, row.get::<_, Option<f64>>(10)?){
                    format!("{}, {}", lat, long)
                  } else { "".into() }
          }
        ))
      })?

    } else {
      (Graffiti {
        id: "#".to_string(),
        complaint_id: "".to_string(),
        date: "".to_string(),
        time: "".to_string(),
        shift_time: model::ShiftTime::Afternoon,
        intervening: "".to_string(),
        notes: "".to_string()
      },
      Location {
        country: "".to_string(),
        city: "".to_string(),
        street: "".to_string(),
        place: "".to_string(),
        property: "".to_string(),
        gps: "".to_string(),
      })
    };

  let mar_image = |src: Option<&str>| {
    let src = match src {
      Some(src) => src,
      None => "{src}"
    };

    html! {
      .image {
        img src=(src) {  }
        .controls {
          .sh {
            .shl { svg { title { "move left" }  use xlink:href={ (root_url) "static/img/sprite.svg#angle-left" }{}} }
            .shr { svg { title { "move right" } use xlink:href={ (root_url) "static/img/sprite.svg#angle-right" }{}} }
          }
          .del { svg { title { "delete" } use xlink:href={ (root_url) "static/img/sprite.svg#times-circle" }{}} }
        }
        .processing_overlay {
          svg { title { "uploading" } use xlink:href={ (root_url) "static/img/sprite.svg#spinner" }{}}
        }
      }
    }
  };

  html! {
    (include!("header.rs"))
    
    .page-graffiti-add {
      .container {
        .actions-wrapper {
          span.action-btn#save {
            "Save"
            svg {use xlink:href={ (root_url) "static/img/sprite.svg#save" }{}}
          }
        }
        .row1 {
          .node107 {
            .node107_1.boxed {
              p.box-title { "Information" }
              .descr {
                .row { .l { "ID: " }  .r { (graffiti.id) } }
                .row { .l { "Complaint ID: " }  .r { input#complaint_id type="text" placeholder="0000/0000" value=(graffiti.complaint_id); } }
                .row { .l { "Date: " }          .r { input#date type="text" placeholder="2018-01-01" value=(graffiti.date); } }
                .row { .l { "Time: " }          .r { input#time type="text" placeholder="00:00" value=(graffiti.time); } }
                .row { .l { "Shift: " }         .r { 
                  select#shift_time {
                    @for shift in model::ShiftTime::iter() {
                      @if graffiti.shift_time == shift { 
                        option value=({shift as u8}) selected="" { (shift.to_string()) }
                      } @else {
                        option value=({shift as u8}) { (shift.to_string()) }
                      }
                    }
                  }
                }}
                .row { .l { "Intervening: " }   .r { input#intervening type="text" value=(graffiti.intervening); } }
              }
            }
            .node107_2.boxed { 
              p.box-title { "GPS location" }
              input#gps type="text" placeholder="0.0, 0.0" value=(location.gps);
            }
          }
          .node108.boxed {
            p.box-title { "Author(s)" }
            .items { 
              .row.title { .l { "Author(s)" } .r { "Indubitable" } }
              @for _ in 1..7 {
                .row { .l { input type="text" {  } } .r { input type="checkbox"; } }
              }
            }
          }
          .node109.boxed {
            p.box-title { "Notes" }
            textarea#notes { (graffiti.notes) }
          }
          .node110.boxed {
            p.box-title { "Location" }
            .descr {
              .row { .l { "Country: " }  .r { input#country type="text" value=(location.country); } }
              .row { .l { "City: " }     .r { input#city type="text" value=(location.city); } }
              .row { .l { "Street: " }   .r { input#street type="text" value=(location.street); } }
              .row { .l { "Property: " } .r { input#property type="text" value=(location.property); } }
              .row { .l { "Place: " }    .r { input#place type="text" value=(location.place); } }
            }
          }
        }
        .row2 {
          .node111.boxed {
            p.box-title { "Images" }
            .img_upload_wrp {
              @for _ in 1..=4 {
                (mar_image(None))
              }
              .image.add title="Upload images" {
                svg {use xlink:href={ (root_url) "static/img/box-add.svg#box-add" }{}}
                div data-type="x-template" {
                  (mar_image(None))
                }
              }
              input type="file" id="openfiledlg" multiple="multiple" accept=".jpg" {  }
            }
          }
          .node112.boxed {
            .tags_wrp {
              span.label { "Tags:" }
              svg.add {use xlink:href={ (root_url) "static/img/sprite.svg#plus" }{}}
            }
          }
        }
      }
    }
  }
}