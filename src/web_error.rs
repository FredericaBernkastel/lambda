#[derive(Debug)]
pub enum WebError {
  Success,
  InternalError { d: String },
  InvalidLogin,
  InvalidRequest,
}

impl Into<u8> for WebError {
  fn into(self) -> u8 {
    match self {
      WebError::Success => 0,
      WebError::InternalError { .. } => 100,
      WebError::InvalidLogin => 101,
      WebError::InvalidRequest => 102,
    }
  }
}

impl std::fmt::Display for WebError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl std::error::Error for WebError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(self)
  }
}

impl From<String> for WebError {
  fn from(error: String) -> Self {
    return WebError::InternalError { d: error };
  }
}

impl From<rusqlite::Error> for WebError {
  fn from(error: rusqlite::Error) -> Self {
    return WebError::InternalError { d: format!("{:?}", error) };
  }
}

impl<T> From<actix_web::error::BlockingError<T>> for WebError
  where T: std::fmt::Debug {
  fn from(error: actix_web::error::BlockingError<T>) -> Self {
    return WebError::InternalError { d: format!("{:?}", error) };
  }
}

impl From<std::io::Error> for WebError {
  fn from(error: std::io::Error) -> Self {
    return WebError::InternalError { d: format!("{:?}", error) };
  }
}

impl From<&str> for WebError {
  fn from(error: &str) -> Self {
    return WebError::InternalError { d: error.to_string() };
  }
}

impl From<image::error::ImageError> for WebError {
  fn from(error: image::error::ImageError) -> Self {
    return WebError::InternalError { d: format!("{:?}", error) };
  }
}

impl From<std::str::Utf8Error> for WebError {
  fn from(error: std::str::Utf8Error) -> Self {
    return WebError::InternalError { d: format!("{:?}", error) };
  }
}

impl From<serde_json::Error> for WebError {
  fn from(error: serde_json::Error) -> Self {
    return WebError::InternalError { d: format!("{:?}", error) };
  }
}

impl From<base64::DecodeError> for WebError {
  fn from(error: base64::DecodeError) -> Self {
    return WebError::InternalError { d: format!("{:?}", error) };
  }
}

impl From<std::option::NoneError> for WebError {
  fn from(error: std::option::NoneError) -> Self {
    return WebError::InternalError { d: format!("{:?}", error) };
  }
}

impl From<std::num::ParseIntError> for WebError {
  fn from(error: std::num::ParseIntError) -> Self {
    return WebError::InternalError { d: format!("{:?}", error) };
  }
}

impl From<r2d2::Error> for WebError {
  fn from(error: r2d2::Error) -> Self {
    return WebError::InternalError { d: format!("{:?}", error) };
  }
}