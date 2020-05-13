use error_chain::error_chain;
use actix_web::{ HttpResponse, error::BlockingError };
use actix_http::Response;
use actix_http::http::StatusCode;

error_chain! {
  foreign_links {
    R2D2Error(r2d2::Error);
    RusqliteError(rusqlite::Error);
    ParseIntError(std::num::ParseIntError);
    SerdeJsonError(serde_json::error::Error);
    Base64DecodeError(base64::DecodeError);
    ImageError(image::error::ImageError);
    IoError(std::io::Error);
  }

  errors {
    ActixError
    NoneError(e: std::option::NoneError)

    InvalidLogin
    InvalidRequest
  }
}

impl<T> From<BlockingError<T>> for Error
  where T: std::fmt::Debug {
  fn from(e: BlockingError<T>) -> Self {
    let kind = match e {
      BlockingError::Error(_) => ErrorKind::ActixError,
      BlockingError::Canceled => ErrorKind::ActixError,
    };
    Error::from_kind(kind)
  }
}

impl From<std::option::NoneError> for Error {
  fn from(e: std::option::NoneError) -> Self {
    Error::from_kind(ErrorKind::NoneError(e))
  }
}

impl From<actix_http::error::PayloadError> for Error {
  fn from(_: actix_http::error::PayloadError) -> Self {
    Error::from_kind(ErrorKind::ActixError)
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
  fn status_code(&self) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
  }
  fn error_response(&self) -> Response {
    HttpResponse::InternalServerError().body(display(self))
  }
}