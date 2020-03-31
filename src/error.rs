// Copyright (C) 2020 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error as StdError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::str::from_utf8;

use http::Error as HttpError;
use http::StatusCode as HttpStatusCode;
use serde_json::Error as JsonError;


/// An error type that any endpoint related error can be converted into.
///
/// Please note that this error type necessarily looses some information
/// over dealing with the actual endpoint error type.
#[derive(Debug)]
pub enum Error {
  /// An HTTP related error.
  Http(HttpError),
  /// We encountered an HTTP that either represents a failure or is not
  /// supported.
  HttpStatus(HttpStatusCode, Vec<u8>),
  /// A JSON conversion error.
  Json(JsonError),
}

impl Display for Error {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
    match self {
      Error::Http(err) => write!(fmt, "{}", err),
      Error::HttpStatus(status, data) => {
        write!(fmt, "HTTP status: {}: ", status)?;
        match from_utf8(&data) {
          Ok(s) => fmt.write_str(s)?,
          Err(b) => write!(fmt, "{:?}", b)?,
        }
        Ok(())
      },
      Error::Json(err) => write!(fmt, "{}", err),
    }
  }
}

impl StdError for Error {
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    match self {
      Error::Http(err) => err.source(),
      Error::HttpStatus(..) => None,
      Error::Json(err) => err.source(),
    }
  }
}

impl From<HttpError> for Error {
  fn from(e: HttpError) -> Self {
    Error::Http(e)
  }
}

impl From<JsonError> for Error {
  fn from(e: JsonError) -> Self {
    Error::Json(e)
  }
}
