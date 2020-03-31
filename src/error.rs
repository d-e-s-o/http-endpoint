// Copyright (C) 2020 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error as StdError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use http::Error as HttpError;
use http::StatusCode as HttpStatusCode;
use hyper::Error as HyperError;
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
  HttpStatus(HttpStatusCode),
  /// An error reported by the `hyper` crate.
  Hyper(HyperError),
  /// A JSON conversion error.
  Json(JsonError),
}

impl Display for Error {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
    match self {
      Error::Http(err) => write!(fmt, "{}", err),
      Error::HttpStatus(status) => write!(fmt, "HTTP status: {}", status),
      Error::Hyper(err) => write!(fmt, "{}", err),
      Error::Json(err) => write!(fmt, "{}", err),
    }
  }
}

impl StdError for Error {
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    match self {
      Error::Http(err) => err.source(),
      Error::HttpStatus(..) => None,
      Error::Hyper(err) => err.source(),
      Error::Json(err) => err.source(),
    }
  }
}

impl From<HttpError> for Error {
  fn from(e: HttpError) -> Self {
    Error::Http(e)
  }
}

impl From<HttpStatusCode> for Error {
  fn from(e: HttpStatusCode) -> Self {
    Error::HttpStatus(e)
  }
}

impl From<HyperError> for Error {
  fn from(e: HyperError) -> Self {
    Error::Hyper(e)
  }
}

impl From<JsonError> for Error {
  fn from(e: JsonError) -> Self {
    Error::Json(e)
  }
}
