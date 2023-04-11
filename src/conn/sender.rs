use anyhow::Result;
use tokio::{
  io::{AsyncWriteExt, BufWriter},
  net::tcp::WriteHalf,
};

pub struct Sender<'a> {
  writer: BufWriter<WriteHalf<'a>>,
}

impl<'a> Sender<'a> {
  pub fn from(wh: WriteHalf<'a>) -> Self {
    Self {
      writer: BufWriter::new(wh),
    }
  }

  pub async fn send(&mut self, data: &[u8]) -> Result<()> {
    self.writer.write_u32(data.len() as u32).await?;
    self.writer.write_all(data).await?;
    self.writer.flush().await?;
    Ok(())
  }
}

