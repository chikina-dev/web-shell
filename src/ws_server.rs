use crate::lib::server;
use crate::lib::pty;

use std::convert::Infallible;
use hyper::{Request, Response};
use hyper::body::{Bytes, Incoming};
use http_body_util::combinators::BoxBody;

pub async fn handle_socket(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
    println!("WebSocket upgrade request received");
    let (response, websocket) = hyper_tungstenite::upgrade(req, None).unwrap();
    tokio::spawn(async move {
        println!("Waiting for WebSocket connection...");
        match websocket.await {
            Ok(ws) => {
                println!("WebSocket connection established");
                if let Err(e) = pty::pty_session(ws).await {
                    eprintln!("Error in WebSocket session: {}", e);
                }
            },
            Err(e) => {
                eprintln!("WebSocket upgrade failed: {}", e);
            }
        }
    });
    return Ok(response.map(|_| server::empty_body()));
}