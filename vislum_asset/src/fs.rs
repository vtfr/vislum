use crate::{AssetEvent, AssetPathResolver};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crossbeam::channel::Sender;
use notify_debouncer_full::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebouncedEvent, Debouncer, RecommendedCache};

#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("The file was not found")]
    NotFound,
}

/// A trait for filesystem operations.
pub trait Fs: Send + Sync {
    /// Reads a file from the filesystem.
    fn read(&self, path: &Path) -> Result<Arc<[u8]>, FsError>;
}

// /// A filesystem implementation for the desktop.
// pub struct DesktopFs;

/// A filesystem implementation for the desktop with embedded file watcher.
pub struct DesktopFsWatcher {
    #[allow(dead_code)]
    debouncer: Debouncer<RecommendedWatcher, RecommendedCache>,
}

impl DesktopFsWatcher {
    pub fn new(asset_path_resolver: AssetPathResolver, event_tx: Sender<AssetEvent>) -> Self {
        let mut debouncer = {
            let asset_path_resolver = asset_path_resolver.clone();

            notify_debouncer_full::new_debouncer(
                Duration::from_secs(1),
                None,
                move |event: Result<
                    Vec<DebouncedEvent>,
                    Vec<notify_debouncer_full::notify::Error>,
                >| {
                    match event {
                        Ok(events) => {
                            for event in events {
                                if event.kind.is_modify() {
                                    let Some(path) = event.paths.first() else {
                                        continue;
                                    };
                                    let Some(path) = asset_path_resolver.unresolve(path) else {
                                        continue;
                                    };

                                    let _ = event_tx.send(AssetEvent::Changed(path.into_owned()));
                                }
                            }
                        }
                        Err(errors) => {
                            dbg!(&errors);
                        }
                    }
                },
            )
            .unwrap()
        };

        debouncer
            .watch(asset_path_resolver.root_path(), RecursiveMode::Recursive)
            .unwrap();

        Self { debouncer }
    }
}

impl Fs for DesktopFsWatcher {
    fn read(&self, path: &Path) -> Result<Arc<[u8]>, FsError> {
        match std::fs::read(path) {
            Ok(data) => Ok(Arc::from(data)),
            Err(_) => Err(FsError::NotFound),
        }
    }
}
