use futures_util::SinkExt;
use futures_util::StreamExt;
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use nix::sys::wait::waitpid;
use nix::pty::*;
use nix::unistd::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::ffi::CString;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs::File;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub async fn pty_session(mut ws: WebSocketStream<TokioIo<Upgraded>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("PTY session started");
    let OpenptyResult { master, slave } = openpty(None, None)?;

    match unsafe { fork()? } {
        ForkResult::Child => {
            setsid()?;

            let shell = CString::new("/bin/bash")?;
            execvp(&shell, &[shell.clone()])?;

            std::process::exit(1);
        }
        ForkResult::Parent { child } => {
            close(slave);
            let mut master_async = Arc::new(Mutex::new(File::from_std(unsafe { std::fs::File::from_raw_fd(master.as_raw_fd()) })));

            let (mut ws_write, mut ws_read) = ws.split();

            let reader = {
                let master_async = Arc::clone(&master_async);
                tokio::spawn(async move {
                    while let Some(msg) = ws_read.next().await {
                        let mut master_async = master_async.lock().await;
                        match msg {
                            Ok(Message::Text(cmd)) => {
                                if master_async.write_all(cmd.as_bytes()).await.is_err() {
                                    break;
                                }
                            }
                            Ok(Message::Binary(bin)) => {
                                if master_async.write_all(&bin).await.is_err() {
                                    break;
                                }
                            }
                            _ => break,
                        }
                    }
                })
            };

            let writer = {
                let master_async = Arc::clone(&master_async);
                tokio::spawn(async move {
                    let mut buf = [0u8; 1024];
                    loop {
                        let mut master_async = master_async.lock().await;
                        let n = match master_async.read(&mut buf).await {
                            Ok(0) => break,
                            Ok(n) => n,
                            Err(_) => break,
                        };
                        if ws_write.send(Message::Text(String::from_utf8_lossy(&buf[..n]).to_string().into())).await.is_err() {
                            break;
                        }
                    }
                })
            };

            tokio::try_join!(reader, writer);
            waitpid(child, None)?;
        }
    }

    Ok(())
}