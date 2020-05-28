--
-- File generated with SQLiteStudio v3.2.1 on Thu May 28 22:16:29 2020
--
-- Text encoding used: System
--
PRAGMA foreign_keys = off;
BEGIN TRANSACTION;

-- Table: author
create table author (
  id              INTEGER primary key autoincrement,
  name            TEXT    not null,
  age             INTEGER,
  height          INTEGER,
  handedness      INTEGER,
  home_city       TEXT    not null
                          default (''),
  social_networks TEXT    not null
                          default (''),
  notes           TEXT    not null
                          default (''),
  views           INTEGER not null
                          default (0),
  last_viewed     INTEGER
);


-- Table: author_image
create table author_image (
  author_id INTEGER   references author (id) 
                      not null,
  hash      CHAR (64) not null,
  `order`   INTEGER   not null
);


-- Table: graffiti
create table graffiti (
  id           INTEGER primary key autoincrement
                       not null,
  complaint_id TEXT    not null
                       default (''),
  datetime     INTEGER,
  shift_time   INTEGER,
  intervening  TEXT    not null
                       default (''),
  companions   INTEGER not null
                       default (0),
  notes        TEXT    not null
                       default (''),
  views        INTEGER not null
                       default (0),
  last_viewed  INTEGER,
  author_count INTEGER not null
                       default (0) 
);


-- Table: graffiti_author
create table graffiti_author (
  graffiti_id INTEGER references graffiti (id) 
                      not null,
  author_id   INTEGER references author (id) 
                      not null,
  indubitable BOOLEAN not null
                      default (0) 
);


-- Table: graffiti_image
create table graffiti_image (
  graffiti_id INTEGER   references graffiti (id) 
                        not null,
  hash        CHAR (64) not null,
  `order`     INTEGER   not null
);


-- Table: graffiti_tag
create table graffiti_tag (
  graffiti_id INTEGER references graffiti (id) 
                      not null,
  tag_id      INTEGER references tag (id) 
                      not null
);


-- Table: location
create table location (
  graffiti_id INTEGER references graffiti (id) 
                      not null
                      unique,
  country     TEXT    not null
                      default (''),
  city        TEXT    not null
                      default (''),
  street      TEXT    not null
                      default (''),
  place       TEXT    not null
                      default (''),
  property    TEXT    not null
                      default (''),
  gps_long    REAL,
  gps_lat     REAL
);


-- Table: sessions
create table sessions (
  id      CHAR (64) primary key
                    not null,
  uid     INTEGER   references users (id) 
                    not null,
  expires INTEGER   not null
);


-- Table: tag
create table tag (
  id    INTEGER primary key autoincrement
                not null,
  name  TEXT    not null
                unique on conflict ignore,
  count INTEGER not null
                default (0) 
);


-- Table: tmp_store_image
create table tmp_store_image (
  id        CHAR (64) not null,
  timestamp INTEGER   not null
);


-- Table: users
create table users (
  id       INTEGER       primary key autoincrement
                         not null,
  login    VARCHAR (255) not null
                         unique,
  password CHAR (64)     not null
);


-- Index: author_image_author_id
create index author_image_author_id on author_image (
  author_id
);


-- Index: author_image_thumbnail
create unique index author_image_thumbnail on author_image (
  author_id,
  "order"
)
where `order` = 0;


-- Index: graffiti_author_author_id
create index graffiti_author_author_id on graffiti_author (
  author_id
);


-- Index: graffiti_author_graffiti_id
create index graffiti_author_graffiti_id on graffiti_author (
  graffiti_id
);


-- Index: graffiti_image_graffiti_id
create index graffiti_image_graffiti_id on graffiti_image (
  graffiti_id
);


-- Index: graffiti_image_thumbnail
create unique index graffiti_image_thumbnail on graffiti_image (
  graffiti_id,
  "order"
)
where `order` = 0;


-- Index: graffiti_tag_graffiti_id
create index graffiti_tag_graffiti_id on graffiti_tag (
  graffiti_id
);


-- Index: graffiti_tag_tag_id
create index graffiti_tag_tag_id on graffiti_tag (
  tag_id
);


-- Index: location_graffiti_id
create index location_graffiti_id on location (
  graffiti_id
);


-- Index: tag_name
create index tag_name on tag (
  name
);


-- Index: users_login
create index users_login on users (
  login
);


-- Trigger: graffiti_author_delete
create trigger graffiti_author_delete
        before delete
            on graffiti_author
      for each row
begin
  update graffiti
     set author_count = author_count - 1
   where graffiti.id = old.graffiti_id;
end;


-- Trigger: graffiti_author_insert
create trigger graffiti_author_insert
         after insert
            on graffiti_author
      for each row
begin
  update graffiti
     set author_count = author_count + 1
   where graffiti.id = new.graffiti_id;
end;


-- Trigger: graffiti_tag_delete
create trigger graffiti_tag_delete
         after delete
            on graffiti_tag
      for each row
begin
  update tag
     set count = count - 1
   where tag.id = old.tag_id;
end;


-- Trigger: graffiti_tag_insert
create trigger graffiti_tag_insert
         after insert
            on graffiti_tag
      for each row
begin
  update tag
     set count = count + 1
   where tag.id = new.tag_id;
end;


COMMIT TRANSACTION;
PRAGMA foreign_keys = on;
