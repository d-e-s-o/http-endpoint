// Copyright (C) 2020 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use hyper::Body;
use hyper::body::to_bytes;
use hyper::Client as HttpClient;
use hyper::http::request::Builder as HttpRequestBuilder;
use hyper::Request;
use hyper_tls::HttpsConnector;

use http_endpoint::Endpoint;

use url::Url;

const HTTP_BIN_BASE_URL: &str = "https://httpbin.org/";


fn request<E>(input: &E::Input) -> Result<Request<Body>, E::Error>
where
  E: Endpoint,
{
  let mut url = Url::parse(HTTP_BIN_BASE_URL).unwrap();
  url.set_path(&E::path(&input));
  url.set_query(E::query(&input).as_ref().map(AsRef::as_ref));

  let request = HttpRequestBuilder::new()
    .method(E::method())
    .uri(url.as_str())
    .body(E::body(input)?)?;

  Ok(request)
}

pub async fn issue<E>(input: &E::Input) -> Result<E::Output, E::Error>
where
  E: Endpoint,
{
  let client = HttpClient::builder().build(HttpsConnector::new());
  let request = request::<E>(input)?;
  let result = client.request(request).await?;
  let status = result.status();
  let bytes = to_bytes(result.into_body()).await?;
  let body = bytes.as_ref();
  E::evaluate(status, body)
}