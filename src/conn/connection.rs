use tokio::net::TcpStream;

// use crate::connection::{Receiver, Sender};
use super::receiver::Receiver;
use super::sender::Sender;

pub struct Connection {
  stream: TcpStream,
}

impl Connection {
  pub fn from(stream: TcpStream) -> Self {
    Self { stream }
  }

  pub fn split(&mut self) -> (Sender, Receiver) {
    let (rs, ws) = self.stream.split();
    (Sender::from(ws), Receiver::from(rs))
  }
}
