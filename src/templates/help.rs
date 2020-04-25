html! { 
  (include!("header.rs"))
  
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
          a data-fancybox="" href={ (root_url) "static/img/help1.png" } {
            img src={ (root_url) "static/img/help1.png" };
          }
        }
        div {
          p {
            "Modification of existing entries is also possible inside the entry's page."
          }
          a data-fancybox="" href={ (root_url) "static/img/help2.png" } {
            img src={ (root_url) "static/img/help2.png" };
          }
        }
        div {
          p {
            "It is also possible to add, delete or modify any tags of the database in the tag menu,
             these tags come to use in the graffiti, where they can be added or deleted at will and
             will help with the search and organization of them."
          }
          a data-fancybox="" href={ (root_url) "static/img/help3.png" } {
            img src={ (root_url) "static/img/help3.png" };
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
}