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

  struct Author {
    id: u32,
    indubitable: bool,
    name: String
  }

  let ((graffiti, location), images, authors) = web::block({
    let db = db.get().unwrap();
    
    move || -> Result<_, WebError> {
      if path == "/graffiti/:id/edit" {

        let id: u32 = data.get("id").ok_or("")?.parse()?;

        Ok((
          db.query_row("
            select a.complaint_id as `0`,
                   a.datetime as `1`,
                   a.shift_time as `2`,
                   a.intervening as `3`,
                   a.notes as `4`,
                   b.country as `5`,
                   b.city as `6`,
                   b.street as `7`,
                   b.place as `8`,
                   b.property as `9`,
                   b.gps_long as `10`,
                   b.gps_lat as `11`
              from graffiti a
                   left join location b on b.graffiti_id = a.id
             where a.id = :id", params![id], |row| {
            Ok((

              // graffiti
              Graffiti {
                id: id.to_string(),
                complaint_id: row.get(0)?,
                date: row.get::<_, Option<i64>>(1)?.map_or("".into(), |x| util::format_timestamp(x as u64, "%Y-%m-%d")),
                time: row.get::<_, Option<i64>>(1)?.map_or("".into(), |x| util::format_timestamp(x as u64, "%H:%M")),
                shift_time: model::ShiftTime::from_u8(row.get(2)?).unwrap_or(model::ShiftTime::Afternoon),
                intervening: row.get(3)?,
                notes: row.get(4)?,
              },

              // location
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
          })?,

          // images
          db.prepare("
            select hash
              from graffiti_image
             where graffiti_id = :id
             order by `order` asc")?
            .query_map(params![id], |row| {
              Ok(row.get::<_, String>(0)?)
            })?.filter_map(Result::ok).collect(),

          // authors
          db.prepare("
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
            })?.filter_map(Result::ok).collect()
        ))

      } else {
        Ok((
          (
            // graffiti
            Graffiti {
              id: "#".to_string(),
              complaint_id: "".to_string(),
              date: "".to_string(),
              time: "".to_string(),
              shift_time: model::ShiftTime::Afternoon,
              intervening: "".to_string(),
              notes: "".to_string()
            },

            // location
            Location {
              country: "".to_string(),
              city: "".to_string(),
              street: "".to_string(),
              place: "".to_string(),
              property: "".to_string(),
              gps: "".to_string(),
            }
          ),

          // images
          vec![],

          // authors
          vec![]
        ))
      }
    }
  }).await?;

  let mar_author_row = |author: Option<Author>| {
    html! {
      .row { 
        .l {
          svg.delete {use xlink:href={ (root_url) "static/img/sprite.svg#times" }{}}
          @if let Some(author) = &author {
            input type="text" readonly="" autocomplete="off" value=(author.name) data-id=(author.id); 
          } @else {
            input type="text" readonly="" autocomplete="off"; 
          }
        } 
        .r {
          @let checked = 
            {
              if let Some(author) = author {
                author.indubitable
              } else { false }
            };
          @if checked {
            input type="checkbox" checked=""; 
          } @else {
            input type="checkbox"; 
          }
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
              .row.title { 
                .l { "Author(s)" } 
                .r { "Indubitable" } 
              }
              @for author in authors {
                (mar_author_row(Some(author)))
              }
              (mar_author_row(None))
              div data-type="x-template" data=((mar_author_row(None)).into_string()) { }
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
              @for image in images.into_iter() {
                (mar_image(Some(&image), "{}static/img/graffiti/{}/{}_p1.jpg", config))
              }
              .image.add title="Upload images" {
                svg {use xlink:href={ (root_url) "static/img/box-add.svg#box-add" }{}}
                div data-type="x-template" data=(mar_image(None, "", config).into_string()) { }
              }
              input type="file" id="openfiledlg" multiple="multiple" accept=".jpg";
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