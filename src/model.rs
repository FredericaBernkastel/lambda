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