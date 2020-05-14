use error_chain::error_chain;

error_chain! {
  foreign_links {
    R2D2Error(r2d2::Error);
    RusqliteError(rusqlite::Error);
    ParseIntError(std::num::ParseIntError);
    SerdeJsonError(serde_json::error::Error);
    Base64DecodeError(base64::DecodeError);
    ImageError(image::error::ImageError);
    IoError(std::io::Error);
    TomlError(toml::de::Error);
    ClapError(clap::Error);
  }

  errors {
    GenericError(e: String)
    NoneError(e: std::option::NoneError)

    InvalidLogin
    InvalidRequest
  }
}

impl<T> From<actix_web::error::BlockingError<T>> for Error
  where T: std::fmt::Debug {
  fn from(e: actix_web::error::BlockingError<T>) -> Self {
    Error::from_kind(ErrorKind::GenericError(format!("{:?}", e)))
  }
}

impl From<std::option::NoneError> for Error {
  fn from(e: std::option::NoneError) -> Self {
    Error::from_kind(ErrorKind::NoneError(e))
  }
}

impl From<actix_http::error::PayloadError> for Error {
  fn from(e: actix_http::error::PayloadError) -> Self {
    Error::from_kind(ErrorKind::GenericError(format!("{:?}", e)))
  }
}

pub fn display(error: &Error) -> String {
  let mut msg = "Error:\n".to_string();
  error
    .iter()
    .enumerate()
    .for_each(|(index, error)|
      msg.push_str(format!("└> {} - {}", index, error).as_str())
    );

  if let Some(backtrace) = error.backtrace() {
    msg.push_str(format!("{:?}", backtrace).as_str());
  }
  eprintln!("{}", msg);
  msg
}


impl actix_http::error::ResponseError for Error {
  fn status_code(&self) -> actix_http::http::StatusCode {
    actix_http::http::StatusCode::INTERNAL_SERVER_ERROR
  }
  fn error_response(&self) -> actix_http::Response {
    actix_web::HttpResponse::InternalServerError().body(display(self))
  }
}