use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::StatusCode;
use hyper::Response;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use std::convert::Infallible;

pub fn empty_body() -> BoxBody<Bytes, Infallible> {
  full_body(Bytes::new())
}

pub fn not_found() -> Response<BoxBody<Bytes, Infallible>> {
  Response::builder()
      .status(StatusCode::NOT_FOUND)
      .body(full_body(Bytes::from("404 Not Found")))
      .unwrap()
}

pub fn full_body(bytes: impl Into<Bytes>) -> BoxBody<Bytes, Infallible> {
  Full::new(bytes.into())
      .map_err(|_| -> Infallible { unreachable!() })
      .boxed()
}