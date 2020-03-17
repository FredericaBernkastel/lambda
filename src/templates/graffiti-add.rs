{

  /*enum ShiftTime {
    morning = 0,
    afternoon = 1,
    night = 2
  };

  struct Location {

  };

  struct Data {
    id: u32,
    complaint_id: String,
    date: String,
    shift: ShiftTime,
    intervening: String,
    notes: String,
    authors: Vec<String, bool>
  };

  if *path == "/graffiti/:id/edit" {

  }*/

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
          a href="#" {
            span.action-btn#save {
              "Save"
              svg {use xlink:href={ (root_url) "static/img/sprite.svg#save" }{}}
            }
          }
        }
        .row1 {
          .node107 {
            .node107_1.boxed {
              p.box-title { "Information" }
              .descr {
                .row { .l { "ID: " }  .r { "#" } }
                .row { .l { "Complaint ID: " }  .r { input type="text" placeholder="0000/0000" {  } } }
                .row { .l { "Date: " }          .r { input type="text" placeholder="2018-01-01" {  } } }
                .row { .l { "Time: " }          .r { input type="text" placeholder="00:00" {  } } }
                .row { .l { "Shift: " }         .r { input type="text" {  } } }
                .row { .l { "Intervening: " }   .r { input type="text" placeholder="0000/0000" {  } } }
              }
            }
            .node107_2.boxed { 
              p.box-title { "GPS location" }
              input type="text" placeholder="0.0, 0.0" {  }
            }
          }
          .node108.boxed {
            p.box-title { "Author(s)" }
            .items { 
              .row.title { .l { "Author(s)" } .r { "Indubitable" } }
              @for _ in 1..7 {
                .row { .l { input type="text" {  } } .r { input type="checkbox" {  } } }
              }
            }
          }
          .node109.boxed {
            p.box-title { "Notes" }
            textarea {  }
          }
          .node110.boxed {
            p.box-title { "Location" }
            .descr {
              .row { .l { "Country: " }  .r { input type="text" {  } } }
              .row { .l { "City: " }     .r { input type="text" {  } } }
              .row { .l { "Street: " }   .r { input type="text" {  } } }
              .row { .l { "Property: " } .r { input type="text" {  } } }
              .row { .l { "Place: " }    .r { input type="text" {  } } }
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