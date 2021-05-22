// Copyright (C) 2020-2021 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

mod common;

use http::Method;
use http::StatusCode;

use http_endpoint::EndpointDef;
use http_endpoint::Str;

use test_env_log::test;

use common::issue;
use common::Error;
use common::NoError;


EndpointDef! {
  GetStatus(u16),
  Ok => (), [
    /* 204 */ NO_CONTENT,
  ],
  Err => GetError, [
    /* 404 */ NOT_FOUND => NotFound,
  ],
  ConversionErr => NoError,
  ApiErr => NoError,

  fn path(status: &Self::Input) -> Str {
    format!("/status/{}", status).into()
  }

  fn parse(_: &[u8]) -> Result<Self::Output, Self::ConversionError> {
    Ok(())
  }

  fn parse_err(body: &[u8]) -> Result<Self::ApiError, Vec<u8>> {
    assert_eq!(body, &[0; 0]);
    Ok(NoError)
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
  ConversionErr => NoError,
  ApiErr => NoError,

  fn method() -> Method {
    Method::POST
  }

  fn path(status: &Self::Input) -> Str {
    format!("/status/{}", status).into()
  }

  fn parse(_: &[u8]) -> Result<Self::Output, Self::ConversionError> {
    Ok(())
  }

  fn parse_err(body: &[u8]) -> Result<Self::ApiError, Vec<u8>> {
    assert_eq!(body, &[0; 0]);
    Ok(NoError)
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
      assert_eq!(err.unwrap(), NoError)
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
