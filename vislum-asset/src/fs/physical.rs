use std::{path::PathBuf, time::Duration};

use crossbeam::channel::Sender;
use notify_debouncer_full::{
    DebouncedEvent, Debouncer, RecommendedCache,
    notify::{RecommendedWatcher, RecursiveMode},
};

use crate::{
    asset::InternalAssetEvent,
    fs::{File, Fs, ReadError},
    path::AssetUri,
};

/// A filesystem implementation for the editor.
pub struct PhysicalFs {
    root: PathBuf,
    watcher: Watcher,
}

impl PhysicalFs {
    pub fn new(root: PathBuf, event_tx: Sender<InternalAssetEvent>) -> Self {
        let watcher = Watcher::new(root.clone(), event_tx);

        Self { root, watcher }
    }
}

impl Fs for PhysicalFs {
    fn read(&self, path: &AssetUri) -> Result<File, ReadError> {
        // Convert AssetPath to filesystem path
        let physical_path = self.root.join(path.path());

        let bytes = std::fs::read(&physical_path)?;

        Ok(File::new_with_physical_path(physical_path, bytes))
    }
}

/// A filesystem implementation for the editor with embedded file watcher.
pub(crate) struct Watcher {
    #[allow(dead_code)]
    debouncer: Debouncer<RecommendedWatcher, RecommendedCache>,
}

impl Watcher {
    pub fn new(root_path: PathBuf, event_tx: Sender<InternalAssetEvent>) -> Self {
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
                        _root_path: &std::path::PathBuf,
                        _event_tx: &Sender<InternalAssetEvent>,
                    ) {
                        // Filter only modify events.
                        if !event.kind.is_modify() {
                            return;
                        }

                        // // Get the first path.
                        // let Some(path) = event.paths.first() else {
                        //     return;
                        // };

                        println!("event: {:?}", event);
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
