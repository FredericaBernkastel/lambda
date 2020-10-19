use error_chain::{error_chain, ChainedError};

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
    FromUtf8Error(std::string::FromUtf8Error);
    ChronoParseError(chrono::format::ParseError);
  }

  errors {
    NoneError(e: std::option::NoneError)

    InvalidLogin
    InvalidRequest
    RouteNotFound
  }
}

impl<T> From<actix_web::error::BlockingError<T>> for Error
where
  T: ChainedError,
{
  fn from(e: actix_web::error::BlockingError<T>) -> Self {
    match e {
      actix_web::error::BlockingError::Error(e) => Error::with_chain(e, ""),
      actix_web::error::BlockingError::Canceled => "request cancelled".into(),
    }
  }
}

impl From<std::option::NoneError> for Error {
  fn from(e: std::option::NoneError) -> Self {
    Error::from_kind(ErrorKind::NoneError(e))
  }
}

impl From<actix_http::error::PayloadError> for Error {
  fn from(e: actix_http::error::PayloadError) -> Self {
    e.to_string().into()
  }
}

pub fn display(error: &Error) -> String {
  match error.kind() {
    ErrorKind::RouteNotFound => "".to_string(),
    _ => {
      let mut msg = "Error:\n".to_string();
      error
        .iter()
        .enumerate()
        .for_each(|(index, error)| msg.push_str(&format!("└> {} - {}", index, error)));

      if let Some(backtrace) = error.backtrace() {
        msg.push_str(&format!("\n\n{:?}", backtrace));
      }
      eprintln!("{}", msg);
      msg
    }
  }
}

impl actix_http::error::ResponseError for Error {
  fn status_code(&self) -> actix_http::http::StatusCode {
    match self.kind() {
      ErrorKind::RouteNotFound => actix_http::http::StatusCode::NOT_FOUND,
      _ => actix_http::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
  fn error_response(&self) -> actix_http::Response {
    match self.kind() {
      ErrorKind::RouteNotFound => actix_web::HttpResponse::NotFound().body(display(self)),
      ErrorKind::InvalidRequest => actix_web::HttpResponse::BadRequest().body({
        #[cfg(debug_assertions)]
        {
          display(self)
        }
        #[cfg(not(debug_assertions))]
        {
          "".to_string()
        }
      }),
      _ => actix_web::HttpResponse::InternalServerError().body(display(self)),
    }
  }
}
