use std::{
  fs::canonicalize,
  io::SeekFrom,
  ops::Range,
  path::{Path, PathBuf},
};

use anyhow::Result;
use bytes::BytesMut;
use tokio::{
  fs::File,
  io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};

use super::{FsInfo, FsMeta};

pub struct FsStore {
  dir: PathBuf,
}

pub const CHUNK_SIZE: usize = 64 * 1024; // 64kb

impl FsStore {
  pub fn new(dir_path: impl AsRef<Path>) -> Self {
    Self {
      dir: canonicalize(dir_path).unwrap(),
    }
  }

  pub fn dir(&self) -> &Path {
    &self.dir
  }

  pub fn list_fs_paths(&self) -> Vec<String> {
    get_all_files(&self.dir)
  }

  pub async fn list_fs_infos(&self) -> Result<Vec<FsInfo>> {
    let fs_paths = self.list_fs_paths();
    let mut fs_infos: Vec<FsInfo> = vec![];
    for fs_path in fs_paths {
      fs_infos.push(FsInfo {
        fs_path: fs_path.clone(),
        meta: self.get_fs_meta(&fs_path).await?,
      });
    }
    Ok(fs_infos)
  }

  /*
  pub async fn save_file(&self, file_path: impl AsRef<Path>, file: FsFile) -> Result<()> {
    let full_path = self.dir.join(file_path);
    let mut f = {
      if full_path.exists() {
        tokio::fs::OpenOptions::new()
          .write(true)
          .open(full_path)
          .await
          .unwrap()
      } else {
        let parent_dir = full_path.parent().unwrap();
        if !parent_dir.exists() {
          tokio::fs::create_dir_all(parent_dir).await?;
        }
        tokio::fs::File::create(full_path).await?
      }
    };
    f.write_all(&file.content).await?;
    // if let FsContent::Bytes(bytes) = file.content {
    //   f.write_all(&bytes).await?;
    // }
    Ok(())
  }
  */

  pub async fn get_file(&self, fs_path: impl AsRef<Path>) -> Result<File> {
    let full_path = self.dir.join(fs_path);
    if full_path.exists() {
      Ok(
        tokio::fs::OpenOptions::new()
          .write(true)
          .open(full_path)
          .await?,
      )
    } else {
      let parent_dir = full_path.parent().unwrap();
      if !parent_dir.exists() {
        tokio::fs::create_dir_all(parent_dir).await?;
      }
      Ok(tokio::fs::File::create(full_path).await?)
    }
  }

  pub async fn write_fs_chunk(&self, file: &mut File, offset: u64, fs_chunk: &[u8]) -> Result<()> {
    file.seek(SeekFrom::Start(offset)).await?;
    file.write_all(fs_chunk).await?;
    Ok(())
  }

  pub async fn get_fs_info(&self, fs_path: &str) -> Result<FsInfo> {
    Ok(FsInfo {
      fs_path: fs_path.to_string(),
      meta: self.get_fs_meta(fs_path).await?,
    })
  }

  pub async fn get_fs_meta(&self, fs_path: impl AsRef<Path>) -> Result<FsMeta> {
    let path = self.dir.join(fs_path);
    let mut file = File::open(&path).await?;
    let mut buf = BytesMut::new();
    let mut ctx = md5::Context::new();
    let mut total_size = 0;
    loop {
      match file.read_buf(&mut buf).await? {
        0 => {
          break;
        }
        size => {
          total_size += size;
          ctx.consume(&buf);
          buf.clear();
          continue;
        }
      }
    }
    let digest = ctx.compute();
    Ok(FsMeta {
      md5: digest.0,
      size: total_size,
    })
  }

  pub async fn get_fs_chunk(
    &self,
    fs_path: impl AsRef<Path>,
    range: Range<u64>,
  ) -> Result<Vec<u8>> {
    let path = self.dir.join(fs_path);
    let mut file = File::open(&path).await?;

    if range.start > 0 {
      file.seek(SeekFrom::Start(range.start)).await?;
    }

    let (range_size, _) = range.size_hint();
    let mut buf = BytesMut::new();
    loop {
      match file.read_buf(&mut buf).await? {
        0 => {
          break;
        }
        _size => {
          if buf.len() >= range_size {
            break;
          }
          continue;
        }
      }
    }

    if buf.len() < range_size {
      return Ok(buf.to_vec());
    }

    Ok(buf[0..range_size].to_vec())
  }

  /*
  pub async fn get_file(&self, relative_path: impl AsRef<Path>) -> Result<FsFile> {
    let path = self.dir.join(relative_path);
    let mut file = File::open(&path).await?;
    let mut buf = BytesMut::new();
    let mut ctx = md5::Context::new();
    let mut total_size = 0;
    loop {
      match file.read_buf(&mut buf).await? {
        0 => {
          if total_size < CONTENT_BYTES_LIMIT {
            ctx.consume(&buf);
          }
          break;
        }
        size => {
          total_size += size;
          if total_size >= CONTENT_BYTES_LIMIT {
            ctx.consume(&buf);
            buf.clear();
          }
          continue;
        }
      }
    }
    let digest = ctx.compute();
    if total_size >= CONTENT_BYTES_LIMIT {
      Ok(FsFile {
        meta: FsMeta {
          md5: digest.0,
          size: total_size,
        },
        content: FsContent::Path(path.to_str().unwrap().to_string()),
      })
    } else {
      Ok(FsFile {
        meta: FsMeta {
          md5: digest.0,
          size: total_size,
        },
        content: FsContent::Bytes(buf.to_vec()),
      })
    }
  }
  */
}

fn recursive_scan_files<F>(dir: &Path, callback: &mut F) -> Result<()>
where
  F: FnMut(&Path),
{
  if dir.is_dir() {
    for entry in std::fs::read_dir(dir)? {
      let path = entry?.path();
      if path.is_dir() {
        recursive_scan_files(dir, callback)?;
      } else {
        callback(&path);
      }
    }
  }
  Ok(())
}

fn get_all_files(dir: &Path) -> Vec<String> {
  let mut files: Vec<String> = vec![];
  recursive_scan_files(dir, &mut |file| {
    if let Ok(fs_path) = file.strip_prefix(dir) {
      if let Some(f) = fs_path.to_str() {
        files.push(f.into());
      }
    }
  })
  .unwrap();
  files
}
