use anyhow::{anyhow, Result};
use bytes::{Buf, Bytes, BytesMut};
use tokio::net::tcp::ReadHalf;

fn extract_frame(buf: &mut BytesMut, frame_size: usize) -> Result<Option<Bytes>> {
  let frame = Bytes::copy_from_slice(&buf[0..frame_size]);
  buf.advance(frame_size);
  Ok(Some(frame))
}

pub struct Receiver<'a> {
  reader: ReadHalf<'a>,
  buffer: BytesMut,
}

impl<'a> Receiver<'a> {
  pub fn from(reader: ReadHalf<'a>) -> Self {
    Self {
      reader,
      buffer: BytesMut::with_capacity(1024),
    }
  }

  pub async fn recv(&mut self) -> Result<Option<Bytes>> {
    let mut frame_size = 0;
    let mut loop_count = 0;

    loop {
      if loop_count > 1000 {
        return Err(anyhow!("too much loop"));
      }
      loop_count += 1;

      if frame_size > 0 && self.buffer.remaining() >= frame_size {
        return extract_frame(&mut self.buffer, frame_size);
        // let frame = Bytes::copy_from_slice(&self.buffer[0..frame_size]);
        // self.buffer.advance(frame_size);
        // return Ok(Some(frame));
      }

      self.reader.readable().await?;
      match self.reader.try_read_buf(&mut self.buffer) {
        Ok(0) => {
          if self.buffer.is_empty() {
            return Ok(None);
          } else {
            return Err(anyhow!("buffer is not empty"));
          }
        }
        Ok(_n) => {
          if frame_size == 0 {
            if self.buffer.len() < 4 {
              continue;
            }
            frame_size = self.buffer.get_u32() as usize;
          }
          if self.buffer.remaining() < frame_size {
            continue;
          }

          return extract_frame(&mut self.buffer, frame_size);
          // let frame = Bytes::copy_from_slice(&self.buffer[0..frame_size]);
          // self.buffer.advance(frame_size);
          // return Ok(Some(frame));
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
          // println!("would block");
          // self.ready().await?;
          continue;
        }
        Err(e) => return Err(e.into()),
      }
    }

    // Ok(None)
  }
}

