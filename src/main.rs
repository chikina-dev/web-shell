mod static_server;
mod ws_server;
mod lib;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use std::convert::Infallible;
use hyper::{Request, Response};
use hyper::body::{Bytes, Incoming};
use http_body_util::combinators::BoxBody;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 8080).into();
    let listener = TcpListener::bind(addr).await?;
    println!("Server running on http://127.0.0.1:8080");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let service = service_fn(handle_maneger);
        let http = http1::Builder::new()
            .serve_connection(io, service);

        if let Err(err) = http.await {
            eprintln!("Error serving connection: {}", err);
        }
    }
}

async fn handle_maneger(mut req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
    let path = req.uri().path();

    if path == "/ws" && hyper_tungstenite::is_upgrade_request(&req) {
        return ws_server::handle_socket(req).await;
    } else {
        return static_server::handle_request(path).await;
    }
}