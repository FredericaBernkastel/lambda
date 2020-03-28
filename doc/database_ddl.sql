--
-- File generated with SQLiteStudio v3.1.1 on Sun Mar 29 03:05:51 2020
--
-- Text encoding used: System
--
PRAGMA foreign_keys = off;
BEGIN TRANSACTION;

-- Table: author
CREATE TABLE author (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL,
    age             INTEGER,
    height          INTEGER,
    handedness      INTEGER,
    home_city       TEXT    NOT NULL
                            DEFAULT (''),
    social_networks TEXT    NOT NULL
                            DEFAULT (''),
    notes           TEXT    NOT NULL
                            DEFAULT (''),
    views           INTEGER NOT NULL
                            DEFAULT (0) 
);


-- Table: graffiti
CREATE TABLE graffiti (
    id           INTEGER PRIMARY KEY AUTOINCREMENT
                         NOT NULL,
    complaint_id TEXT    NOT NULL
                         DEFAULT (''),
    datetime     INTEGER,
    shift_time   INTEGER,
    intervening  TEXT    NOT NULL
                         DEFAULT (''),
    companions   INTEGER NOT NULL
                         DEFAULT (0),
    notes        TEXT    NOT NULL
                         DEFAULT (''),
    views        INTEGER NOT NULL
                         DEFAULT (0) 
);


-- Table: graffiti_image
CREATE TABLE graffiti_image (
    graffiti_id INTEGER   REFERENCES graffiti (id) 
                          NOT NULL,
    hash        CHAR (64) NOT NULL,
    [order]     INTEGER   NOT NULL
);


-- Table: location
CREATE TABLE location (
    graffiti_id INTEGER REFERENCES graffiti (id) 
                        NOT NULL
                        UNIQUE,
    country     TEXT    NOT NULL
                        DEFAULT (''),
    city        TEXT    NOT NULL
                        DEFAULT (''),
    street      TEXT    NOT NULL
                        DEFAULT (''),
    place       TEXT    NOT NULL
                        DEFAULT (''),
    property    TEXT    NOT NULL
                        DEFAULT (''),
    gps_long    REAL,
    gps_lat     REAL
);


-- Table: sessions
CREATE TABLE sessions (
    id      CHAR (64) PRIMARY KEY
                      NOT NULL,
    uid     INTEGER   REFERENCES users (id) 
                      NOT NULL,
    expires INTEGER   NOT NULL
);


-- Table: tmp_store_image
CREATE TABLE tmp_store_image (
    id        CHAR (64) NOT NULL,
    timestamp INTEGER   NOT NULL
);


-- Table: users
CREATE TABLE users (
    id       INTEGER       PRIMARY KEY AUTOINCREMENT
                           NOT NULL,
    login    VARCHAR (255) NOT NULL
                           UNIQUE,
    password CHAR (64)     NOT NULL
);


-- Index: graffiti_image_graffiti_id
CREATE INDEX graffiti_image_graffiti_id ON graffiti_image (
    graffiti_id
);


-- Index: location_graffiti_id
CREATE INDEX location_graffiti_id ON location (
    graffiti_id
);


-- Index: users_login
CREATE INDEX users_login ON users (
    login
);


COMMIT TRANSACTION;
PRAGMA foreign_keys = on;
