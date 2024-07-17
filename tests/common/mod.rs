// Copyright (C) 2020-2024 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;
use std::fmt::Debug;

use hyper::body::to_bytes;
use hyper::http::request::Builder as HttpRequestBuilder;
use hyper::Body;
use hyper::Client as HttpClient;
use hyper::Error as HyperError;
use hyper::Request;
use hyper_tls::HttpsConnector;

use http_endpoint::Endpoint;

use serde::Deserialize;

use thiserror::Error;

use url::Url;

const HTTP_BIN_BASE_URL: &str = "https://httpbin.org/";


#[derive(Debug, Deserialize, Error, PartialEq)]
#[error("an unspecified error was encountered")]
pub struct NoError;


#[derive(Debug)]
pub enum Error<E> {
  EndpointError(E),
  #[allow(dead_code)]
  HyperError(HyperError),
}


fn request<E>(input: &E::Input) -> Result<Request<Body>, E::Error>
where
  E: Endpoint,
{
  let mut url = Url::parse(HTTP_BIN_BASE_URL).unwrap();
  url.set_path(&E::path(input));
  url.set_query(E::query(input)?.as_ref().map(AsRef::as_ref));

  let headers = E::headers(input)?;
  let mut request = HttpRequestBuilder::new()
    .method(E::method())
    .uri(url.as_str())
    .body(Body::from(
      E::body(input)?.unwrap_or(Cow::Borrowed(&[0; 0])),
    ))?;

  if let Some(headers) = headers {
    request.headers_mut().extend(headers);
  }
  Ok(request)
}

pub async fn issue<E>(input: &E::Input) -> Result<E::Output, Error<E::Error>>
where
  E: Endpoint,
{
  let client = HttpClient::builder().build(HttpsConnector::new());
  let request = request::<E>(input).map_err(Error::EndpointError)?;
  let result = client.request(request).await.map_err(Error::HyperError)?;
  let status = result.status();
  let bytes = to_bytes(result.into_body())
    .await
    .map_err(Error::HyperError)?;
  let body = bytes.as_ref();
  let output = E::evaluate(status, body).map_err(Error::EndpointError)?;
  Ok(output)
}
