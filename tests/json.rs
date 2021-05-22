// Copyright (C) 2020-2021 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

mod common;

use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::str::FromStr as _;

use http::header::HeaderMap;
use http::header::HeaderName;
use http::header::HeaderValue;
use http::Method;

use http_endpoint::Bytes;
use http_endpoint::EndpointDef;
use http_endpoint::Str;

use serde::Deserialize;
use serde::Serialize;
use serde_json::from_slice;
use serde_json::to_string as to_json;
use serde_json::Error as JsonError;

use test_env_log::test;

use thiserror::Error as ThisError;

use common::issue;
use common::Error;
use common::NoError;


/// Dummy data used for testing JSON deserialization.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Person {
  name: String,
  age: u8,
}

/// A response from httpbin.
///
/// There are various fields in the response, but the data we sent is
/// available in the "json" field.
#[derive(Debug, Deserialize)]
struct Data<T> {
  #[serde(rename = "json")]
  data: T,
}

impl<T> Display for Data<T>
where
  T: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{}", self.data)
  }
}

impl<T> StdError for Data<T> where T: StdError {}


EndpointDef! {
  PostPerson(Str),
  Ok => Data<Person>, [
    /* 200 */ OK,
  ],
  Err => PostError, [],
  ConversionErr => JsonError,
  ApiErr => NoError,

  fn method() -> Method {
    Method::POST
  }

  fn path(_: &Self::Input) -> Str {
    "/anything".into()
  }

  fn body(input: &Self::Input) -> Result<Option<Bytes>, Self::ConversionError> {
    Ok(Some(input.to_string().into_bytes().into()))
  }

  fn parse(body: &[u8]) -> Result<Self::Output, Self::ConversionError> {
    from_slice::<Self::Output>(body)
  }

  fn parse_err(body: &[u8]) -> Result<Self::ApiError, Vec<u8>> {
    assert_eq!(body, &[0; 0]);
    Ok(NoError)
  }
}


/// Dummy data used for testing JSON deserialization.
#[derive(Debug, Deserialize, PartialEq, Serialize, ThisError)]
#[error("{message} ({code})")]
struct ApiError {
  message: String,
  code: u8,
}


EndpointDef! {
  PostApiError(Str),
  Ok => (), [],
  Err => PostApiErrorError, [
    /* 200 */ OK => Ok,
  ],
  ConversionErr => JsonError,
  ApiErr => Data<ApiError>,

  fn method() -> Method {
    Method::POST
  }

  fn path(_: &Self::Input) -> Str {
    "/anything".into()
  }

  fn body(input: &Self::Input) -> Result<Option<Bytes>, Self::ConversionError> {
    Ok(Some(input.to_string().into_bytes().into()))
  }

  fn parse(body: &[u8]) -> Result<Self::Output, Self::ConversionError> {
    from_slice::<Self::Output>(body)
  }

  fn parse_err(body: &[u8]) -> Result<Self::ApiError, Vec<u8>> {
    from_slice::<Self::ApiError>(body).map_err(|_| body.to_vec())
  }
}


#[derive(Debug, Deserialize)]
struct Headers {
  #[serde(rename = "headers")]
  headers: HashMap<String, String>,
}


EndpointDef! {
  GetHeaders(Vec<(&'static str, &'static str)>),
  Ok => Headers, [
    /* 200 */ OK,
  ],
  Err => GetError, [],
  ConversionErr => JsonError,
  ApiErr => NoError,

  fn path(_: &Self::Input) -> Str {
    "/headers".into()
  }

  fn headers(input: &Self::Input) -> Result<Option<HeaderMap>, Self::ConversionError> {
    let mut headers = HeaderMap::with_capacity(input.len());
    for (key, value) in input {
      headers.append(HeaderName::from_str(key).unwrap(), HeaderValue::from_str(value).unwrap());
    }
    Ok(Some(headers))
  }

  fn parse(body: &[u8]) -> Result<Self::Output, Self::ConversionError> {
    from_slice::<Self::Output>(body)
  }

  fn parse_err(body: &[u8]) -> Result<Self::ApiError, Vec<u8>> {
    assert_eq!(body, &[0; 0]);
    Ok(NoError)
  }
}


/// Test decoding of a JSON formatted response.
#[test(tokio::test)]
async fn decode_json() {
  let person = Person {
    name: "Peter".to_string(),
    age: 37,
  };
  let json = to_json(&person).unwrap();
  let result = issue::<PostPerson>(&json.into()).await.unwrap();
  assert_eq!(result.data, person);
}

/// Test the case that we can't decode the response.
#[test(tokio::test)]
async fn decode_json_error() {
  let json = r#"{ foobar: invalid" }"#;
  let err = issue::<PostPerson>(&json.into()).await.unwrap_err();
  match err {
    Error::EndpointError(PostError::Conversion(err)) => {
      // httpbin auto-fills the "json" field, but if the JSON is valid
      // there will be nothing. Hence, the error is about a "null" being
      // encountered.
      assert!(
        err
          .to_string()
          .starts_with("invalid type: null, expected struct Person"),
        "{}",
        err,
      )
    },
    _ => panic!("unexpected error: {:?}", err),
  }
}

/// Test decoding of an API error.
#[test(tokio::test)]
async fn decode_api_error() {
  let api_error = ApiError {
    message: "that's a failure".to_string(),
    code: 42,
  };
  let json = to_json(&api_error).unwrap();
  let err = issue::<PostApiError>(&json.into()).await.unwrap_err();
  match err {
    Error::EndpointError(PostApiErrorError::Ok(err)) => assert_eq!(err.unwrap().data, api_error),
    _ => panic!("unexpected error: {:?}", err),
  }
}

/// Check that request headers are honored properly.
#[test(tokio::test)]
async fn request_headers() {
  let headers = vec![("Foobar", "foobaz"), ("Another", "header")];
  let response = issue::<GetHeaders>(&headers).await.unwrap();
  assert_eq!(response.headers.get("Another").unwrap(), "header");
  assert_eq!(response.headers.get("Foobar").unwrap(), "foobaz");
  assert_eq!(response.headers.get("Foobaz"), None);
}
