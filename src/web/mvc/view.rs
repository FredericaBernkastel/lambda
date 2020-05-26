use super::model::{self, Model as View};
use crate::{
  error::{ErrorKind, Result},
  schema, util,
};
use error_chain::bail;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use runtime_fmt::{rt_format, rt_format_args};
use serde_json::{json, Value as JsonValue};
use std::collections::VecDeque;
use strum::IntoEnumIterator;

impl View {
  pub fn v_root(&self, body: Markup, js_glob: JsonValue) -> Result<Markup> {
    Ok(html! {
      (DOCTYPE)
      html lang="en" {
        head {
          meta http-equiv="Content-Type" content="text/html; charset=utf-8";
          meta name="viewport" content="width=device-width";

          link rel="stylesheet" href={ (self.root_url) "static/vendors.css" } type="text/css" media="screen";
          link rel="stylesheet" href={ (self.root_url) "static/style.css" } type="text/css" media="screen";
          script type="text/javascript" src={ (self.root_url) "static/vendors.js" } {  }
          script type="text/javascript" src={ (self.root_url) "static/script.js" } {  }

          title { "Graffiti database" }

          script type="text/javascript" {
            "var __glob = " (PreEscaped(js_glob.to_string())) ";"
          }
        }
        body {
          (body)
        }
      }
    })
  }

  pub fn v_login(&self) -> Result<Markup> {
    Ok(html! {
      .page-login {
        .login {
          p.title { "Graffiti database" }
          .form {
            .input-wrapper {
              .icon { (self.svg_sprite("user", "", "")) }
              input#login type="text" placeholder="username";
            }
            .input-wrapper {
              .icon { (self.svg_sprite("key", "", "")) }
              input#password type="password" placeholder="password";
            }
            p.si-error {  }
            button#submit
              data-html=((html! {
                "login" span.icon { (self.svg_sprite("sign-in-alt", "", "")) }
              }).into_string())
              data-spinner=((html! {
                (self.svg_sprite("spinner", "fa-spinner", ""))
              }).into_string())
              {
                "login" span.icon { (self.svg_sprite("sign-in-alt", "", "")) }
              }
          }
        }
      }
    })
  }

  pub fn v_graffitis(
    &self,
    graffitis: Vec<model::graffitis_Graffiti>,
    mar_navigation: Markup,
  ) -> Result<Markup> {
    Ok(html! {
      (self.mar_header()?)

      .page-graffitis {
        .container {
          .actions-wrapper {
            a href={ (self.root_url) "views/graffiti/add" } {
              span.action-btn#add-graffiti {
                "Add new graffiti"
                (self.svg_sprite("plus", "", ""))
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
                  a.graffiti-img href={ (self.root_url) "views/graffiti/" (graffiti.id) } {
                    @if let Some(thumbnail) = graffiti.thumbnail {
                      img src=(format!("{}static/img/graffiti/{}/{}_p2.jpg", self.root_url, thumbnail.get(0..=1)?, thumbnail));
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
    })
  }

  pub fn v_graffiti_edit(
    &self,
    graffiti: model::graffiti_edit_Graffiti,
    location: model::graffiti_edit_Location,
    images: Vec<String>,
    authors: Vec<model::graffiti_Author>,
    tags: Vec<String>,
  ) -> Result<Markup> {
    let mar_author_row = |author: Option<model::graffiti_Author>| {
      html! {
        .row {
          .l {
            (self.svg_sprite("times", "delete", ""))
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

    Ok(html! {
      (self.mar_header()?)

      .page-graffiti-add {
        .container {
          .actions-wrapper {
            span.action-btn#save {
              "Save"
              (self.svg_sprite("save", "", ""))
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
                      @for shift in schema::ShiftTime::iter() {
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
                @for image in images {
                  (self.mar_image(Some(&image), "{}static/img/graffiti/{}/{}_p1.jpg")?)
                }
                .image.add title="Upload images" {
                  svg {use xlink:href={ (self.root_url) "static/img/box-add.svg#box-add" }{}}
                  div data-type="x-template" data=(self.mar_image(None, "")?.into_string()) { }
                }
                input type="file" id="openfiledlg" multiple="multiple" accept=".jpg";
              }
            }
            .node112.boxed {
              p.box-title { "Tags" }
              .tags_wrp {
                select.tags-input multiple="" autocomplete="off"  {
                  @for tag in tags {
                    option selected="" { (tag) }
                  }
                }
              }
            }
          }
        }
      }
    })
  }

  pub fn v_home(
    &self,
    graffitis_recent: Vec<model::home_Graffiti>,
    graffitis_last_checked: Vec<model::home_Graffiti>,
    authors_last_checked: Vec<(/* id: */ u32, /* name: */ String)>,
  ) -> Result<Markup> {
    let map_aggregate = graffitis_recent
      .iter()
      .filter_map(|x| {
        Some(json!({
         "id": x.id,
         "thumbnail": x.thumbnail.clone(),
         "coords": x.coords?
        }))
      })
      .collect(): JsonValue;

    Ok(html! {
      (self.mar_header()?)

      .page-home {
        .container {
          .node101 {
            .node103.boxed {
              p.box-title { "Most recent additions" }
              .images {
                @for graffiti in graffitis_recent {
                  a href={ (self.root_url) "views/graffiti/" (graffiti.id) } {
                    .image {
                      @if let Some(thumbnail) = graffiti.thumbnail {
                        img src=(format!("{}static/img/graffiti/{}/{}_p1.jpg", self.root_url, thumbnail.get(0..=1)?, thumbnail));
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
                  a href={ (self.root_url) "views/graffiti/" (graffiti.id) } {
                    .image {
                      @if let Some(thumbnail) = graffiti.thumbnail {
                        img src=(format!("{}static/img/graffiti/{}/{}_p1.jpg", self.root_url, thumbnail.get(0..=1)?, thumbnail));
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
              .map data=(json!(map_aggregate).to_string()) {}
            }
            .node105.boxed {
              p.box-title { "Last checked authors" }
              .authors {
                @for (id, name) in authors_last_checked {
                  a href={ (self.root_url) "views/author/" (id) } { (name) }
                }
              }
            }
          }
        }
      }
    })
  }

  pub fn v_graffiti(
    &self,
    graffiti: schema::Graffiti,
    location: schema::Location,
    images: Vec<String>,
    authors: Vec<model::graffiti_Author>,
    tags: Vec<String>,
  ) -> Result<Markup> {
    let (gps_json, gps_label) =
      if let (Some(lat), Some(long)) = (location.gps_lat, location.gps_long) {
        (
          json!([{
            "id": graffiti.id,
            "thumbnail": "",
            "coords": [lat, long]
          }])
          .to_string(),
          format!("[{}, {}]", lat, long),
        )
      } else {
        ("[]".into(), "".into())
      };

    Ok(html! {
      (self.mar_header()?)

      .page-graffiti {
        .container {
          .actions-wrapper {
            a.action-btn#edit href={ (self.root_url) "views/graffiti/" (graffiti.id) "/edit" } {
              "Modify graffiti"
              (self.svg_sprite("edit", "", ""))
            }
            span.action-btn.red#delete {
              "Delete"
              (self.svg_sprite("trash-alt", "", ""))
            }
          }
          .row1 {
            .node100.boxed {
              p.box-title { "Information" }
              .descr {
                .row { .l { "Graffiti ID: " } .r { (graffiti.id) } }
                .row { .l { "Complaint ID: " } .r { (graffiti.complaint_id) } }
                .row { .l { "Date: " } .r { (graffiti.datetime.map_or("".into(), |x| util::format_timestamp(x as u64, "%Y-%m-%d"))) } }
                .row { .l { "Time: " } .r { (graffiti.datetime.map_or("".into(), |x| util::format_timestamp(x as u64, "%H:%M"))) } }
                .row { .l { "Shift: " } .r { (graffiti.shift_time.map_or("".into(), |x| x.to_string())) } }
                .row { .l { "Intervening: " } .r { (graffiti.intervening) } }
              }
            }
            a.link-prev href="#" {
              (self.svg_sprite("angle-double-left", "", "Previous image"))
            }
            .node102 {
              .graffiti-image {
                @if let Some(image) = images.get(0) {
                  img data-id="0" src=(format!("{}static/img/graffiti/{}/{}_p1.jpg", self.root_url, image.get(0..=1)?, image));
                  .images data-type="x-template" {
                    (json!(images))
                  }
                } @else {
                  .no-image {  }
                }
              }
              .tags {
                @for tag in tags {
                  a href="#" { (tag) }
                }
              }
            }
            a.link-next href="#" {
              (self.svg_sprite("angle-double-right", "", "Next image"))
            }
            .node101.boxed {
              p.box-title { "Author(s)" }
              .items {
                @for author in authors {
                  @let classname =
                    format!("item {}", if author.indubitable { "checked" } else { "" });
                  .(classname) {
                    (self.svg_sprite("check", "", "checked"))
                    a href={ (self.root_url) "views/author/" (author.id) } {
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
              .map data=(gps_json) {  }
              p {
                (gps_label)
              }
            }
          }
        }
      }
    })
  }

  pub fn v_authors(
    &self,
    authors: Vec<model::authors_Author>,
    mar_navigation: Markup,
  ) -> Result<Markup> {
    Ok(html! {
      (self.mar_header()?)

      .page-authors {
        .container {
          .actions-wrapper {
            a href={ (self.root_url) "views/author/add" } {
              span.action-btn#add-author {
                "Add new author"
                (self.svg_sprite("plus", "", ""))
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
                  a.graffiti-img href={ (self.root_url) "views/author/" (author.id) } {
                    @if let Some(thumbnail) = author.thumbnail {
                      img src=(format!("{}static/img/author/{}/{}_p2.jpg", self.root_url, thumbnail.get(0..=1)?, thumbnail));
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
    })
  }

  pub fn v_author_edit(
    &self,
    author: model::author_edit_Author,
    images: Vec<String>,
  ) -> Result<Markup> {
    Ok(html! {
      (self.mar_header()?)

      .page-author-add {
        .container {
          .actions-wrapper {
            a href="#" {
              span.action-btn#save {
                "Save"
                (self.svg_sprite("save", "", ""))
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
                      @for variant in schema::Handedness::iter() {
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
                    (self.mar_image(Some(&image), "{}static/img/author/{}/{}_p1.jpg")?)
                  }
                  .image.add title="Upload images" {
                    svg {use xlink:href={ (self.root_url) "static/img/box-add.svg#box-add" }{}}
                    div data-type="x-template" data=(self.mar_image(None, "")?.into_string()) { }
                  }
                  input type="file" id="openfiledlg" multiple="multiple" accept=".jpg";
                }
              }
            }
          }
        }
      }
    })
  }

  pub fn v_author(
    &self,
    author: schema::Author,
    images: Vec<String>,
    graffiti_count: u32,
    graffiti_recent: Option<(/* id */ u32, /* thumbnail */ Option<String>)>,
    graffiti_most_viewed: Option<(/* id */ u32, /* thumbnail */ Option<String>)>,
    aggregate_counties: Vec<(/* country */ String, /* count */ u32)>,
    aggregate_cities: Vec<(/* city */ String, /* count */ u32)>,
    aggregate_gps: Vec<model::home_Graffiti>,
  ) -> Result<Markup> {
    Ok(html! {
      (self.mar_header()?)

      .page-author {
        .container {
          .actions-wrapper {
            a.action-btn#edit href={ (self.root_url) "views/author/" (author.id) "/edit" } {
              "Modify author"
              (self.svg_sprite("edit", "", ""))
            }
            span.action-btn.red#delete {
              "Delete"
              (self.svg_sprite("trash-alt", "", ""))
            }
          }
          .row1 {
            .node113 {
              .node113_1 {
                a.link-prev href="#" {
                  (self.svg_sprite("angle-double-left", "", "Previous image"))
                }
                .author-image {
                  @if let Some(image) = images.get(0) {
                    img data-id="0" src=(format!("{}static/img/author/{}/{}_p1.jpg", self.root_url, image.get(0..=1)?, image));
                    .images data-type="x-template" {
                      (json!(images))
                    }
                  } @else {
                    .no-image {  }
                  }
                }
                a.link-next href="#" {
                  (self.svg_sprite("angle-double-right", "", "Next image"))
                }
              }
              .node113_2.boxed {
                p.box-title { "Information" }
                .descr {
                  .row { .l { "Name: " }        .r { (author.name) } }
                  .row { .l { "Age: " }         .r { (author.age.map_or("".into(), |x| x.to_string())) } }
                  .row { .l { "Height: " }      .r { (author.height.map_or("".into(), |x| format!("{}cm", x))) } }
                  .row { .l { "Handedness: " }  .r { (author.handedness.map_or("".into(), |x| x.to_string())) } }
                  .row { .l { "Home city: " }   .r { (author.home_city) } }
                  .row { .l { "Graffiti: " }    .r { (graffiti_count)} }
                }
              }
              .node113_3.boxed {
                p.box-title { "Social networks" }
                .descr {
                  @for line in author.social_networks.lines() {
                    a href=(line) target="_blank" { (line) }
                  }
                }
              }
            }
            .node114 {
              .node114_1.boxed {
                p.box-title { "Notes" }
                .descr { (util::markup_br(author.notes)) }
              }
              .node114_2.boxed {
                p.box-title { "Zones of activity" }
                .descr {
                  .row { .l { "Countries: " } .r {
                    @for (country, count) in aggregate_counties {
                      (format!("{} ({}), ", country, count))
                    }
                  } }
                  .row { .l { "Cities: " } .r {
                    @for (city, count) in aggregate_cities {
                      (format!("{} ({}), ", city, count))
                    }
                  } }
                }
              }
              .node114_3 {
                .map data=(json!(aggregate_gps).to_string()) { }
                p { "Activity map of author" }
              }
            }
            .node115 {
              .node115_1.boxed {
                p.box-title { "Previous companions" }
                .items {
                  .item { a href="#" { "Author Name 1" } }
                  .item { a href="#" { "Author Name 2" } }
                  .item { a href="#" { "Author Name 3" } }
                  .item { a href="#" { "Author Name 4" } }
                  .item { a href="#" { "Author Name 5" } }
                }
              }
              .node115_2.boxed {
                p.box-title { "Relevant graffiti" }
                .items {
                  .item {
                    p { "most recent" }
                    @if let Some((id, thumbnail)) = graffiti_recent {
                      a.img href={ (self.root_url) "views/graffiti/" (id) } {
                        @if let Some(image) = thumbnail {
                          img src=(format!("{}static/img/graffiti/{}/{}_p2.jpg", self.root_url, image.get(0..=1)?, image));
                        } @else {
                          .no-image {  }
                        }
                      }
                    } @else {
                      a.img href="#" { .no-image {  } }
                    }
                  }
                  .item {
                    p { "most viewed" }
                    @if let Some((id, thumbnail)) = graffiti_most_viewed {
                      a.img href={ (self.root_url) "views/graffiti/" (id) } {
                        @if let Some(image) = thumbnail {
                          img src=(format!("{}static/img/graffiti/{}/{}_p2.jpg", self.root_url, image.get(0..=1)?, image));
                        } @else {
                          .no-image {  }
                        }
                      }
                    } @else {
                      a.img href="#" { .no-image {  } }
                    }
                  }
                }
              }
            }
          }
        }
      }
    })
  }

  pub fn v_tags(&self) -> Result<Markup> {
    Ok(html! {
      (self.mar_header()?)

      .page-tags {
        .container {
          .node118 {
            .edit {
              span.action-btn#edit {
                "Modify"
                (self.svg_sprite("edit", "", ""))
              }
              input type="text" id="from" placeholder="e.g. \"authorized\"" {  }
              span.arrow { "=>" }
              input type="text" id="to" placeholder="e.g. \"mischief\"" {  }
            }
            .delete {
              .action-btn.red#delete {
                "Delete"
                (self.svg_sprite("trash-alt", "", ""))
              }
              input type="text" placeholder="e.g. \"motto\"" {  }
            }
          }
          .node119 {
            p { b { "List of graffiti tags" } }
            .tags {
              @for _ in 1..=10 {
                a href="#" { "vandalism" }
                a href="#" { "political" }
                a href="#" { "motto" }
              }
            }
          }
        }
      }
    })
  }

  pub fn v_help(&self) -> Result<Markup> {
    Ok(html! {
      (self.mar_header()?)

      .page-help {
        .container {
          .row1 {
            p {
              "Welcome to the Graffiti Database, this site was designed to provide a complete tool for
               the management of graffiti images and their authors, as well as a faciliator in the tasks
               of organizing and analyzing data."
            }
          }
          .row2 {
            div {
              p {
                "It is possible to check or add entries as well as search for them in the Graffiti and
                 Authors menu via the pertinent buttons and submenus."
              }
              a data-fancybox="" href={ (self.root_url) "static/img/help1.png" } {
                img src={ (self.root_url) "static/img/help1.png" };
              }
            }
            div {
              p {
                "Modification of existing entries is also possible inside the entry's page."
              }
              a data-fancybox="" href={ (self.root_url) "static/img/help2.png" } {
                img src={ (self.root_url) "static/img/help2.png" };
              }
            }
            div {
              p {
                "It is also possible to add, delete or modify any tags of the database in the tag menu,
                 these tags come to use in the graffiti, where they can be added or deleted at will and
                 will help with the search and organization of them."
              }
              a data-fancybox="" href={ (self.root_url) "static/img/help3.png" } {
                img src={ (self.root_url) "static/img/help3.png" };
              }
            }
          }
          .row3 {
            p {
              b { "Technical support: " } "email@example.com"
            }
          }
        }
      }
    })
  }

  fn svg_sprite(&self, id: &str, classname: &str, title: &str) -> Markup {
    html! {
      svg.(classname) {
        @if !title.is_empty() {
          title { (title) }
        }
        use xlink:href={ (self.root_url) "static/img/sprite.svg#" (id) }{  }
      }
    }
  }

  fn mar_header(&self) -> Result<Markup> {
    Ok(html! {
      .popup-wrapper#error {
        .popup {
          p.title { "Error!" }
          .inner {
            .message {  }
            .actions-wrapper {
              span.action-btn#close { "Ok" }
            }
          }
        }
      }

      .popup-wrapper#warning {
        .popup {
          p.title { (self.svg_sprite("exclamation-triangle", "", "")) }
          .inner {
            .message {  }
            .actions-wrapper {
              span.action-btn.red#ok { "Ok" }
              span.action-btn#cancel { "Cancel" }
            }
          }
        }
      }

      .header {
        .container {
          .logo { "Graffiti database" }
          .nav-menu {
            .pages {
              a href={ (self.root_url) "views/home" } { "Home" }
              a href={ (self.root_url) "views/graffitis" } { "Graffiti" }
              a href={ (self.root_url) "views/authors" } { "Authors" }
              a href={ (self.root_url) "views/tags" } { "Tags" }
              a href={ (self.root_url) "views/help" } { "Help" }
            }
            .user {
              (self.svg_sprite("user", "icon-user", ""))
              span.login { (self.user.as_ref()?.login) }
              (self.svg_sprite("sign-out-alt", "logout", "logout"))
            }
          }
        }
      }
    })
  }

  pub fn mar_navigation(
    &self,
    link_tpl: &str,
    current_page: i64,
    per_page: i64,
    total: i64,
  ) -> Result<Markup> {
    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    if current_page < 1 || current_page > total_pages {
      bail!(ErrorKind::InvalidRequest);
    }

    let radius = 4;
    let prev_page = match current_page - 1 {
      x if x > 0 => Some(x),
      _ => None,
    };
    let next_page = match current_page + 1 {
      x if x <= total_pages => Some(x),
      _ => None,
    };
    let mut pages = VecDeque::<Option<i64>>::new();
    (current_page - radius..=current_page + radius)
      .filter(|x| *x > 0 && *x <= total_pages)
      .for_each(|x| pages.push_back(Some(x)));
    match current_page - radius - 1 {
      1 => vec![Some(1)],
      x if x > 1 => vec![Some(1), None],
      _ => vec![],
    }
    .into_iter()
    .rev()
    .for_each(|x| pages.push_front(x));
    match -current_page - radius + total_pages {
      1 => vec![Some(total_pages)],
      x if x > 1 => vec![None, Some(total_pages)],
      _ => vec![],
    }
    .into_iter()
    .for_each(|x| pages.push_back(x));

    let link_fmt =
      |page| rt_format!(link_tpl, self.root_url, page).map_err(|_| "invalid format template");

    Ok(html! {
      .navigation {
        .n_back {
          @let svg = html!{
            (self.svg_sprite("chevron-circle-left", "", ""))
          };
          @match prev_page {
            Some(page) => a href=(link_fmt(page)?) { (svg) "prev" },
            None => span { (svg) "prev" }
          }
        }
        .navi_link {
          @for page in pages {
            @match page {
              Some(page) =>
                @if page != current_page {
                  a href=(link_fmt(page)?) { (page) }
                } @else {
                  span { (page) }
                },
              None => { span.nav_ext { "..." } }
            }
          }
        }
        .n_next {
          @let svg = html!{
            (self.svg_sprite("chevron-circle-right", "", ""))
          };
          @match next_page {
            Some(page) => a href=(link_fmt(page)?) { "next" (svg) },
            None => span { "next" (svg) }
          }
        }
      }
    })
  }

  fn mar_image(&self, hash: Option<&str>, path_template: &str) -> Result<Markup> {
    let src = match hash {
      Some(hash) => rt_format!(path_template, self.root_url, hash.get(0..=1)?, hash)
        .map_err(|_| "invalid format template")?,
      None => "{src}".into(),
    };

    Ok(html! {
      .image data-id=(hash.unwrap_or("")) {
        img src=(src) {  }
        .controls {
          .sh {
            .shl { (self.svg_sprite("angle-left", "", "move left")) }
            .shr { (self.svg_sprite("angle-righ", "", "move right")) }
          }
          .del { (self.svg_sprite("times-circle", "", "delete")) }
        }
        .processing_overlay {
          (self.svg_sprite("spinner", "", "uploading"))
        }
      }
    })
  }
}
