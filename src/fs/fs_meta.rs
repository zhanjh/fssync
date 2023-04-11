use serde::{Deserialize, Serialize};
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct FsMeta {
  pub size: usize,
  pub md5: [u8; 16],
}

impl FsMeta {
  pub fn new() -> Self {
    FsMeta {
      size: 0,
      md5: [0; 16],
    }
  }
}
