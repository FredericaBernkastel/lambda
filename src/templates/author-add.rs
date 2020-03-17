{
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
                .row { .l { "Name: " }        .r { input type="text" {  } } }
                .row { .l { "Age: " }         .r { input type="number" {  } } }
                .row { .l { "Height (cm): " } .r { input type="number" {  } } }
                .row { .l { "Home city: " }   .r { input type="text" {  } } }
                .row { .l { "Handedness: " }  .r { 
                  select {
                    option value="0" { "right handed" }
                    option value="1" { "left handed" }
                    option value="2" { "ambidextrous" }
                  }
                }}
              }
            }
            .node116_2.boxed {
              p.box-title { "Social networks" br; span.small { "(one link per line)" } }
              textarea rows="4" {  }
            }
          }
          .node117 {
            .node117_1.boxed {
              p.box-title { "Notes" }
              textarea {  }
            }
            .node117_2.boxed {
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
          }
        }
      }
    }
  }
}