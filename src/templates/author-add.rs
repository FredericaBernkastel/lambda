{
  struct Author {
    name: String,
    age: String,
    height: String,
    handedness: model::Handedness,
    home_city: String,
    social_networks: String,
    notes: String,
  }

  let (author, images) = web::block({
    let db = db.get()?;

    move || -> error::Result<_> {
      if path == "/author/:id/edit" {
        let id: u32 = data.get("id")?.parse()?;
        
        Ok((
          // author
          db.query_row("
            select 
              `name` as '0', `age` as '1', `height` as '2', `handedness` as '3', `home_city` as '4', `social_networks` as '5', `notes` as '6'
            from `author`
            where `id` = :id", params![id], |row| {
              Ok(Author {
                name: row.get(0)?,
                age: row.get::<_, Option<u32>>(1)?.map_or("".into(), |x| x.to_string()),
                height: row.get::<_, Option<u32>>(2)?.map_or("".into(), |x| x.to_string()),
                handedness: model::Handedness::from_u8(row.get(3)?).unwrap_or(model::Handedness::RightHanded),
                home_city: row.get(4)?,
                social_networks: row.get(5)?,
                notes: row.get(6)?
              })
            }
          )?,

          // images
          db.prepare("
            select `hash` from `author_image`
            where `author_id` = :id
            order by `order` asc")?
            .query_map(params![id], |row| {
              Ok(row.get::<_, String>(0)?)
            })?.filter_map(Result::ok).collect()
        ))
      } else {
        Ok((
          // author
          Author {
            name: "".to_string(),
            age: "".to_string(),
            height: "".to_string(),
            handedness: model::Handedness::RightHanded,
            home_city: "".to_string(),
            social_networks: "".to_string(),
            notes: "".to_string(),
          },

          // images
          vec![]
        ))
      }
    }
  }).await?;

  html! {
    (include!("header.rs"))
    
    .page-author-add {
      .container {
        .actions-wrapper {
          a href="#" {
            span.action-btn#save {
              "Save"
              svg {use xlink:href={ (root_url) "static/img/sprite.svg#save" }{}}
            }
          }
        }
        .row1 {
          .node116 {
            .node116_1.boxed {
              p.box-title { "Information" }
              .descr {
                .row { .l { "Name: " }        .r { input#name type="text" value=(author.name); } }
                .row { .l { "Age: " }         .r { input#age type="number" value=(author.age); } }
                .row { .l { "Height (cm): " } .r { input#height type="number" value=(author.height); } }
                .row { .l { "Home city: " }   .r { input#home_city type="text" value=(author.home_city); } }
                .row { .l { "Handedness: " }  .r { 
                  select#handedness {
                    @for variant in model::Handedness::iter() {
                      @if author.handedness == variant { 
                        option value=({variant as u8}) selected="" { (variant.to_string()) }
                      } @else {
                        option value=({variant as u8}) { (variant.to_string()) }
                      }
                    }
                  }
                }}
              }
            }
            .node116_2.boxed {
              p.box-title { "Social networks" br; span.small { "(one link per line)" } }
              textarea#social_networks rows="4" { (author.social_networks) }
            }
          }
          .node117 {
            .node117_1.boxed {
              p.box-title { "Notes" }
              textarea#notes { (author.notes) }
            }
            .node117_2.boxed {
              p.box-title { "Images" }
              .img_upload_wrp {
                @for image in images {
                  (mar_image(Some(&image), "{}static/img/author/{}/{}_p1.jpg", config)?)
                }
                .image.add title="Upload images" {
                  svg {use xlink:href={ (root_url) "static/img/box-add.svg#box-add" }{}}
                  div data-type="x-template" data=(mar_image(None, "", config)?.into_string()) { }
                }
                input type="file" id="openfiledlg" multiple="multiple" accept=".jpg";
              }
            }
          }
        }
      }
    }
  }
}