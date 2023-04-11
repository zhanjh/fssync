use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::FsChunk;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct FsContainer {
  clock: u64,
  chunks: Vec<FsChunk>,
}

impl FsContainer {
  pub fn new(clock: u64) -> Self {
    Self {
      clock,
      chunks: Default::default(),
    }
  }

  pub fn set_chunk(&mut self, index: usize, chunk: FsChunk) -> Result<()> {
    if self.chunks.len() <= index + 1 {
      self.chunks.resize_with(index + 1, Default::default);
    }
    self.chunks[index] = chunk;

    Ok(())
  }

  pub fn get_chunk(&self, index: usize) -> Result<Option<FsChunk>> {
    Ok(self.chunks.get(index).cloned())
  }

  pub fn get_clock(&self) -> u64 {
    self.clock
  }

  pub fn set_clock(&mut self, clock: u64) {
    self.clock = clock;
  }
}
