// Copyright (C) 2020 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

mod common;

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use http::Method;

use hyper::Body;

use http_endpoint::EndpointDef;
use http_endpoint::Str;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Error as JsonError;
use serde_json::to_string as to_json;

use test_env_log::test;

use common::issue;


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


EndpointDef! {
  PostPerson(Str),
  Ok => Data<Person>, [
    /* 200 */ OK,
  ],
  Err => PostError, [],
  ApiErr => String,

  fn method() -> Method {
    Method::POST
  }

  fn path(_: &Self::Input) -> Str {
    "/anything".into()
  }

  fn body(input: &Self::Input) -> Result<Body, JsonError> {
    Ok(Body::from(input.to_string()))
  }
}


/// Dummy data used for testing JSON deserialization.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ApiError {
  message: String,
  code: u8,
}

impl Display for ApiError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{} ({})", self.message, self.code)
  }
}


EndpointDef! {
  PostApiError(Str),
  Ok => (), [],
  Err => PostApiErrorError, [
    /* 200 */ OK => Ok,
  ],
  ApiErr => Data<ApiError>,

  fn method() -> Method {
    Method::POST
  }

  fn path(_: &Self::Input) -> Str {
    "/anything".into()
  }

  fn body(input: &Self::Input) -> Result<Body, JsonError> {
    Ok(Body::from(input.to_string()))
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
    PostError::Json(err) => {
      // httpbin auto-fills the "json" field, but if the JSON is valid
      // there will be nothing. Hence, the error is about a "null" being
      // encountered.
      assert!(
        err
          .to_string()
          .starts_with("invalid type: null, expected struct Person"),
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
    PostApiErrorError::Ok(err) => assert_eq!(err.unwrap().data, api_error),
    _ => panic!("unexpected error: {:?}", err),
  }
}
