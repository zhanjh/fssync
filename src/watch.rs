use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, Result as NotifyResult, Watcher};

use tokio::{runtime::Runtime, sync::mpsc};
use tracing::{error, info};

use super::service::SomeService;

pub struct WatchService {
  service: Arc<SomeService>,
}

impl WatchService {
  pub fn new(service: Arc<SomeService>) -> Self {
    Self { service }
  }

  pub async fn run(&self) -> Result<()> {
    let (mut watcher, mut rx) = watcher()?;
    let dir = self.service.dir();
    watcher.watch(dir, notify::RecursiveMode::Recursive)?;
    while let Some(res) = rx.recv().await {
      match res {
        Ok(event) => {
          let paths = sub_paths(dir, &event.paths);
          if paths.is_empty() {
            continue;
          }
          let has_change = match event.kind {
            EventKind::Create(_) => {
              for file_path in &paths {
                self.service.file_added(file_path).await?;
              }
              true
            }
            EventKind::Modify(_) => {
              for file_path in &paths {
                self.service.file_modified(file_path).await?;
              }
              true
            }
            EventKind::Remove(_) => {
              for file_path in &paths {
                self.service.file_removed(file_path).await?;
              }
              true
            }
            _ => false,
          };

          if !has_change {
            continue;
          }
          info!("weak-peer watch, files changed {paths:?}");
        }
        Err(e) => {
          error!("{e}");
        }
      }
    }
    Ok(())
  }
}

fn watcher() -> Result<(RecommendedWatcher, mpsc::Receiver<NotifyResult<Event>>)> {
  let (tx, rx) = mpsc::channel(1);
  let runtime = Runtime::new()?;
  let watcher = RecommendedWatcher::new(
    move |res| {
      runtime.block_on(async {
        tx.send(res).await.unwrap();
      })
    },
    Config::default(),
  )?;
  Ok((watcher, rx))
}

fn sub_paths(dir: &Path, files: &[PathBuf]) -> Vec<String> {
  let mut paths = Vec::new();
  for file in files {
    if let Ok(sub_path) = file.strip_prefix(dir) {
      if !file.file_name().unwrap().to_str().unwrap().starts_with('.')
        && !file.to_str().unwrap().ends_with('~')
      {
        paths.push(sub_path.to_str().unwrap().to_string());
      }
    }
  }
  paths
}
