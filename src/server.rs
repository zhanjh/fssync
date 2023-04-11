use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use tokio::net::{TcpListener, ToSocketAddrs};
use tracing::info;

use crate::conn;

#[async_trait]
pub trait Service: Send + Sync + 'static {
  async fn handle_frame(&self, addr: SocketAddr, frame: &[u8]) -> Vec<u8>;
}

pub async fn run_server<T: ToSocketAddrs, S: Service>(addr: T, service: Arc<S>) -> Result<()> {
  let listener = TcpListener::bind(addr).await?;

  loop {
    let (stream, remote_addr) = listener.accept().await?;
    info!("server accept connection from {remote_addr}");

    let s = service.clone();

    tokio::spawn(async move {
      let mut conn = conn::Connection::from(stream);
      let (mut sender, mut receiver) = conn.split();

      loop {
        match receiver.recv().await {
          Ok(Some(frame)) => {
            let resp = s.handle_frame(remote_addr, &frame).await;
            sender.send(&resp).await.unwrap();
          }
          Ok(None) => break,
          Err(_e) => break,
        }
      }
    });
  }
}
