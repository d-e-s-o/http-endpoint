// Copyright (C) 2020-2021 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error as StdError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::str::from_utf8;

use http::Error as HttpError;
use http::StatusCode as HttpStatusCode;


/// An error type that any endpoint related error can be converted into.
///
/// Please note that this error type necessarily looses some information
/// over dealing with the actual endpoint error type.
#[derive(Debug)]
pub enum Error<B>
where
  B: StdError,
{
  /// An HTTP related error.
  Http(HttpError),
  /// We encountered an HTTP that either represents a failure or is not
  /// supported.
  HttpStatus(HttpStatusCode, Vec<u8>),
  /// Some kind of conversion error was encountered.
  Conversion(B),
}

impl<B> Display for Error<B>
where
  B: StdError,
{
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
      Error::Conversion(err) => write!(fmt, "{}", err),
    }
  }
}

impl<B> StdError for Error<B>
where
  B: StdError,
{
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    match self {
      Error::Http(err) => err.source(),
      Error::HttpStatus(..) => None,
      Error::Conversion(err) => err.source(),
    }
  }
}

impl<B> From<HttpError> for Error<B>
where
  B: StdError,
{
  fn from(e: HttpError) -> Self {
    Error::Http(e)
  }
}
