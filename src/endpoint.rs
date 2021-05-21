// Copyright (C) 2019-2021 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;
use std::error::Error;

use http::Error as HttpError;
use http::Method;
use http::StatusCode;

use serde::de::DeserializeOwned;
use serde_json::Error as JsonError;
use serde_json::from_slice;

use crate::Bytes;
use crate::Str;


/// A trait describing an HTTP endpoint.
///
/// An endpoint for our intents and purposes is basically a path and an
/// HTTP request method (e.g., GET or POST). The path will be combined
/// with an "authority" (scheme, host, and port) into a full URL. Query
/// parameters are supported as well.
/// An endpoint is used by the `Trader` who invokes the various methods.
pub trait Endpoint {
  /// The type of data being passed in as part of a request to this
  /// endpoint.
  type Input;
  /// The type of data being returned in the response from this
  /// endpoint.
  type Output: DeserializeOwned;
  /// The type of error this endpoint can report.
  type Error: Error + From<HttpError> + From<JsonError> + 'static;
  /// An error emitted by the API.
  type ApiError: Error + DeserializeOwned;

  /// Retrieve the base URL to use.
  ///
  /// By default no URL is provided for the endpoint, in which case it
  /// is the client's responsibility to supply one.
  fn base_url() -> Option<Str> {
    None
  }

  /// Retrieve the HTTP method to use.
  ///
  /// The default method being used is GET.
  fn method() -> Method {
    Method::GET
  }

  /// Inquire the path the request should go to.
  fn path(input: &Self::Input) -> Str;

  /// Inquire the query the request should use.
  ///
  /// By default no query is emitted.
  #[allow(unused)]
  fn query(input: &Self::Input) -> Option<Str> {
    None
  }

  /// Retrieve the request's body.
  ///
  /// By default this method creates an empty body.
  #[allow(unused)]
  fn body(input: &Self::Input) -> Result<Bytes, JsonError> {
    Ok(Cow::Borrowed(&[0; 0]))
  }

  /// Parse the body into the final result.
  ///
  /// By default this method directly parses the body as JSON.
  fn parse(body: &[u8]) -> Result<Self::Output, Self::Error> {
    from_slice::<Self::Output>(body).map_err(Self::Error::from)
  }

  /// Parse an API error.
  fn parse_err(body: &[u8]) -> Result<Self::ApiError, Vec<u8>> {
    from_slice::<Self::ApiError>(body).map_err(|_| body.to_vec())
  }

  /// Evaluate an HTTP status and body, converting it into an output or
  /// error, depending on the status.
  ///
  /// This method is not meant to be implemented manually. It will be
  /// auto-generated.
  #[doc(hidden)]
  fn evaluate(status: StatusCode, body: &[u8]) -> Result<Self::Output, Self::Error>;
}


/// A macro used for defining the properties for a request to a
/// particular HTTP endpoint.
#[macro_export]
macro_rules! EndpointDef {
  ( $(#[$docs:meta])* $pub:vis $name:ident($in:ty),
    // We just ignore any documentation for success cases: there is
    // nowhere we can put it.
    Ok => $out:ty, [$($(#[$ok_docs:meta])* $ok_status:ident,)*],
    Err => $err:ident, [$($(#[$err_docs:meta])* $err_status:ident => $variant:ident,)*],
    ApiErr => $api_err:ty,
    $($defs:tt)* ) => {

    $(#[$docs])*
    #[derive(Clone, Copy, Debug)]
    $pub struct $name;

    /// An enum representing the various errors this endpoint may
    /// encounter.
    #[allow(unused_qualifications)]
    #[derive(Debug)]
    $pub enum $err {
      $(
        $(#[$err_docs])*
        $variant(Result<$api_err, Vec<u8>>),
      )*
      /// An HTTP status not present in the endpoint's definition was
      /// encountered.
      UnexpectedStatus(::http::StatusCode, Result<$api_err, Vec<u8>>),
      /// An HTTP related error.
      Http(::http::Error),
      /// A JSON conversion error.
      Json(::serde_json::Error),
    }

    #[allow(unused_qualifications)]
    impl ::std::fmt::Display for $err {
      fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        fn format_message(message: &Result<$api_err, Vec<u8>>) -> String {
          match message {
            Ok(err) => err.to_string(),
            Err(body) => {
              match ::std::str::from_utf8(&body) {
                Ok(body) => format!("{}", body),
                Err(err) => format!("{:?}", body),
              }
            },
          }
        }

        match self {
          $(
            $err::$variant(message) => {
              let status = ::http::StatusCode::$err_status;
              let message = format_message(message);
              write!(fmt, "HTTP status {}: {}", status, message)
            },
          )*
          $err::UnexpectedStatus(status, message) => {
            let message = format_message(message);
            write!(fmt, "Unexpected HTTP status {}: {}", status, message)
          },
          $err::Http(err) => write!(fmt, "{}", err),
          $err::Json(err) => write!(fmt, "{}", err),
        }
      }
    }

    #[allow(unused_qualifications)]
    impl ::std::error::Error for $err {
      fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
        match self {
          $(
            $err::$variant(..) => None,
          )*
          $err::UnexpectedStatus(..) => None,
          $err::Http(err) => err.source(),
          $err::Json(err) => err.source(),
        }
      }
    }

    #[allow(unused_qualifications)]
    impl ::std::convert::From<::http::Error> for $err {
      fn from(src: ::http::Error) -> Self {
        $err::Http(src)
      }
    }

    #[allow(unused_qualifications)]
    impl ::std::convert::From<::serde_json::Error> for $err {
      fn from(src: ::serde_json::Error) -> Self {
        $err::Json(src)
      }
    }

    #[allow(unused_qualifications)]
    impl ::std::convert::From<$err> for ::http_endpoint::Error {
      fn from(src: $err) -> Self {
        match src {
          $(
            $err::$variant(result) => {
              let status = ::http::StatusCode::$err_status;
              match result {
                Ok(err) => {
                  ::http_endpoint::Error::HttpStatus(status, err.to_string().into_bytes())
                },
                Err(data) => ::http_endpoint::Error::HttpStatus(status, data),
              }
            },
          )*
          $err::UnexpectedStatus(status, result) => {
            match result {
              Ok(err) => {
                ::http_endpoint::Error::HttpStatus(status, err.to_string().into_bytes())
              },
              Err(data) => ::http_endpoint::Error::HttpStatus(status, data),
            }
          },
          $err::Http(err) => ::http_endpoint::Error::Http(err),
          $err::Json(err) => ::http_endpoint::Error::Json(err),
        }
      }
    }

    #[allow(unused_qualifications)]
    impl ::http_endpoint::Endpoint for $name {
      type Input = $in;
      type Output = $out;
      type Error = $err;
      type ApiError = $api_err;

      $($defs)*

      #[allow(unused_qualifications)]
      fn evaluate(
        status: ::http::StatusCode,
        body: &[u8],
      ) -> Result<$out, $err> {
        match status {
          $(
            ::http::StatusCode::$ok_status => {
              <$name as ::http_endpoint::Endpoint>::parse(&body)
            },
          )*
          status => {
            let res = <$name as ::http_endpoint::Endpoint>::parse_err(&body);
            match status {
              $(
                ::http::StatusCode::$err_status => {
                  Err($err::$variant(res))
                },
              )*
              _ => Err($err::UnexpectedStatus(status, res)),
            }
          },
        }
      }
    }
  };
}
