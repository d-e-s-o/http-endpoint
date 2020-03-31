// Copyright (C) 2020 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

mod common;

use http::Method;
use http::StatusCode;

use http_endpoint::EndpointDef;
use http_endpoint::Str;

use test_env_log::test;

use common::issue;
use common::Error;


EndpointDef! {
  GetStatus(u16),
  Ok => (), [
    /* 204 */ NO_CONTENT,
  ],
  Err => GetError, [
    /* 404 */ NOT_FOUND => NotFound,
  ],
  ApiErr => String,

  fn path(status: &Self::Input) -> Str {
    format!("/status/{}", status).into()
  }

  fn parse(_: &[u8]) -> Result<Self::Output, Self::Error> {
    Ok(())
  }
}


EndpointDef! {
  PostStatus(u16),
  Ok => (), [
    /* 200 */ OK,
  ],
  Err => PostError, [
    /* 401 */ UNAUTHORIZED => Unauthorized,
  ],
  ApiErr => String,

  fn method() -> Method {
    Method::POST
  }

  fn path(status: &Self::Input) -> Str {
    format!("/status/{}", status).into()
  }

  fn parse(_: &[u8]) -> Result<Self::Output, Self::Error> {
    Ok(())
  }
}


/// Test handling of the success status code for GET requests.
#[test(tokio::test)]
async fn get_success_status() {
  issue::<GetStatus>(&204).await.unwrap()
}

/// Test handling of an expected error status code for GET requests.
#[test(tokio::test)]
async fn get_expected_error_status() {
  let err = issue::<GetStatus>(&404).await.unwrap_err();
  match err {
    Error::EndpointError(GetError::NotFound(err)) => {
      // The response isn't actually a JSON encoded string. We expect to
      // get back an empty body.
      assert_eq!(err.unwrap_err(), Vec::<u8>::new())
    },
    _ => panic!("unexpected error: {:?}", err),
  };
}

/// Test handling of an unexpected error status code for GET requests.
#[test(tokio::test)]
async fn get_unexpected_error_status() {
  let err = issue::<GetStatus>(&403).await.unwrap_err();
  match err {
    Error::EndpointError(GetError::UnexpectedStatus(StatusCode::FORBIDDEN, ..)) => (),
    _ => panic!("unexpected error: {:?}", err),
  };
}

/// Test handling of the success status code for POST requests.
#[test(tokio::test)]
async fn post_success_status() {
  issue::<PostStatus>(&200).await.unwrap()
}

/// Test handling of an expected error status code for POST requests.
#[test(tokio::test)]
async fn post_expected_error_status() {
  let err = issue::<PostStatus>(&401).await.unwrap_err();
  match err {
    Error::EndpointError(PostError::Unauthorized(..)) => (),
    _ => panic!("unexpected error: {:?}", err),
  };
}
