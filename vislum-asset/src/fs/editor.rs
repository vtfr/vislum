use std::time::Duration;

use crossbeam::channel::Sender;
use notify_debouncer_full::{
    DebouncedEvent, Debouncer, RecommendedCache,
    notify::{RecommendedWatcher, RecursiveMode},
};

use crate::{
    asset::InternalAssetEvent, fs::{Bytes, Fs, ReadError}, path::AssetPath
};

/// A filesystem implementation for the editor.
pub struct EditorFs {
    root_path: std::path::PathBuf,
    watcher: Watcher,
}

impl EditorFs {
    pub fn new(root_path: std::path::PathBuf, event_tx: Sender<InternalAssetEvent>) -> Self {
        let watcher = Watcher::new(root_path.clone(), event_tx);

        Self {
            root_path,
            watcher,
        }
    }
}

impl Fs for EditorFs {
    fn read(&self, path: &AssetPath) -> Result<Bytes, ReadError> {
        // Convert AssetPath to filesystem path
        let fs_path = self.root_path.join(path.path());
        
        match std::fs::read(&fs_path) {
            Ok(data) => Ok(Bytes::new_owned(data)),
            Err(_) => Err(ReadError::NotFound),
        }
    }
}

/// A filesystem implementation for the editor with embedded file watcher.
pub(crate) struct Watcher {
    #[allow(dead_code)]
    debouncer: Debouncer<RecommendedWatcher, RecommendedCache>,
}

impl Watcher {
    pub fn new(
        root_path: std::path::PathBuf,
        event_tx: Sender<InternalAssetEvent>,
    ) -> Self {
        let mut debouncer = {
            let root_path = root_path.clone();

            notify_debouncer_full::new_debouncer(
                Duration::from_secs(1),
                None,
                move |event: Result<
                    Vec<DebouncedEvent>,
                    Vec<notify_debouncer_full::notify::Error>,
                >| {
                    #[inline(always)]
                    fn process_event(
                        event: DebouncedEvent,
                        root_path: &std::path::PathBuf,
                        event_tx: &Sender<InternalAssetEvent>,
                    ) {
                        // Filter only modify events.
                        if !event.kind.is_modify() {
                            return;
                        }

                        // Get the first path.
                        let Some(path) = event.paths.first() else {
                            return;
                        };

                        // Convert filesystem path to AssetPath
                        if let Ok(relative_path) = path.strip_prefix(root_path) {
                            let asset_path = AssetPath::new_owned(relative_path);
                            let _ = event_tx.send(InternalAssetEvent::Changed(asset_path));
                        }
                    }

                    match event {
                        Ok(events) => {
                            for event in events {
                                process_event(event, &root_path, &event_tx);
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
            .watch(&root_path, RecursiveMode::Recursive)
            .unwrap();

        Self { debouncer }
    }
}
