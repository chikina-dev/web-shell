use std::convert::Infallible;
use hyper::Response;
use hyper::body::Bytes;
use tokio::fs::read;
use http_body_util::combinators::BoxBody;
use crate::lib::server;

pub async fn handle_request(path: &str) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {

    let file = match path {
        "/" | "/index.html" => Some(("static/index.html", "text/html")),
        // "/index.css" => Some(("static/index.css", "text/css")),
        // "/index.js" => Some(("static/index.js", "application/javascript")),
        _ => None,
    };

    if let Some((file_path, content_type)) = file {
        match read(file_path).await {
            Ok(content) => {
                let response = Response::builder()
                    .header("Content-Type", content_type)
                    .body(server::full_body(content))
                    .unwrap();
                Ok(response)
                
            },
            Err(_) => Ok(server::not_found()),
        }
    } else {
        Ok(server::not_found())
    }
}