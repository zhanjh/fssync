use serde::{Deserialize, Serialize};

use super::FsMeta;

#[derive(Debug, Serialize, Deserialize)]
pub struct FsInfo {
  pub fs_path: String,
  pub meta: FsMeta,
}
