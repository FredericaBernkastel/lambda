use strum_macros::{Display, EnumIter};

#[derive(Debug)]
pub struct User {
  pub id: u32,
  pub login: String,
  pub password: String
}

pub struct Session {
  pub id: String,
  pub uid: u32,
  pub expires: u64
}

#[derive(Display, Debug, Copy, Clone, PartialEq, FromPrimitive, EnumIter)]
pub enum ShiftTime {
  Morning = 0,
  Afternoon = 1,
  Night = 2
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
  pub views: u32
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
  pub gps_lat: Option<f64>
}

