use serde_repr::Deserialize_repr;
use strum_macros::{Display, EnumIter};

#[derive(Debug)]
pub struct User {
  pub id: u32,
  pub login: String,
  pub password: String,
}

#[derive(Debug)]
pub struct Session {
  pub id: String,
  pub uid: u32,
  pub expires: u64,
}

#[repr(u8)]
#[derive(Display, Debug, Copy, Clone, PartialEq, Deserialize_repr, EnumIter)]
pub enum ShiftTime {
  #[strum(serialize = "Mañana")]
  Morning = 0,
  #[strum(serialize = "Tarde")]
  Afternoon = 1,
  #[strum(serialize = "Noche")]
  Night = 2,
}

#[derive(Debug)]
pub struct Graffiti {
  pub id: u32,
  pub complaint_id: String,
  pub datetime: Option<i64>,
  pub shift_time: Option<ShiftTime>,
  pub intervening: String,
  pub companions: u32,
  pub notes: String,
  pub views: u32,
}

#[derive(Debug)]
pub struct Location {
  pub graffiti_id: u32,
  pub country: String,
  pub city: String,
  pub street: String,
  pub place: String,
  pub property: String,
  pub gps_long: Option<f64>,
  pub gps_lat: Option<f64>,
}

#[repr(u8)]
#[derive(Display, Debug, Copy, Clone, PartialEq, Deserialize_repr, EnumIter)]
pub enum Handedness {
  #[strum(serialize = "Derecha")]
  RightHanded = 0,
  #[strum(serialize = "Izquierda")]
  LeftHanded = 1,
  #[strum(serialize = "Ambidiextro/a")]
  Ambidextrous = 2,
}

#[derive(Debug)]
pub struct Author {
  pub id: u32,
  pub name: String,
  pub age: Option<u32>,
  pub height: Option<u32>,
  pub handedness: Option<Handedness>,
  pub home_city: String,
  pub social_networks: String,
  pub notes: String,
  pub views: u32,
}
