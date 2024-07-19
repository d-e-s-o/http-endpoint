// Copyright (C) 2020-2024 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Debug;

use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes as HyperBytes;
use hyper::http::request::Builder as HttpRequestBuilder;
use hyper::Error as HyperError;
use hyper::Request;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client as HttpClient;
use hyper_util::client::legacy::Error as HyperUtilError;
use hyper_util::rt::TokioExecutor;

use http_endpoint::Bytes;
use http_endpoint::Endpoint;

use serde::Deserialize;

use thiserror::Error;

use url::Url;

const HTTP_BIN_BASE_URL: &str = "https://httpbin.org/";


#[derive(Debug, Deserialize, Error, PartialEq)]
#[error("an unspecified error was encountered")]
pub struct NoError;


#[allow(dead_code)]
#[derive(Debug)]
pub enum Error<E> {
  Endpoint(E),
  Hyper(HyperError),
  HyperUtil(HyperUtilError),
}


fn request<E>(input: &E::Input) -> Result<Request<Full<HyperBytes>>, E::Error>
where
  E: Endpoint,
{
  let mut url = Url::parse(HTTP_BIN_BASE_URL).unwrap();
  url.set_path(&E::path(input));
  url.set_query(E::query(input)?.as_ref().map(AsRef::as_ref));

  let body = match E::body(input)? {
    None => HyperBytes::new(),
    Some(Bytes::Borrowed(slice)) => HyperBytes::from(slice),
    Some(Bytes::Owned(vec)) => HyperBytes::from(vec),
  };

  let headers = E::headers(input)?;
  let mut request = HttpRequestBuilder::new()
    .method(E::method())
    .uri(url.as_str())
    .body(Full::new(body))?;

  if let Some(headers) = headers {
    request.headers_mut().extend(headers);
  }
  Ok(request)
}

pub async fn issue<E>(input: &E::Input) -> Result<E::Output, Error<E::Error>>
where
  E: Endpoint,
{
  let client = HttpClient::builder(TokioExecutor::new()).build(HttpsConnector::new());
  let request = request::<E>(input).map_err(Error::Endpoint)?;
  let result = client.request(request).await.map_err(Error::HyperUtil)?;
  let status = result.status();
  let bytes = BodyExt::collect(result)
    .await
    .map_err(Error::Hyper)?
    .to_bytes();
  let body = bytes.as_ref();
  let output = E::evaluate(status, body).map_err(Error::Endpoint)?;
  Ok(output)
}
