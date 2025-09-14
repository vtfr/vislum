use std::path::Path;
use std::time::Duration;

use crossbeam::channel::Sender;
use notify_debouncer_full::{
    DebouncedEvent, Debouncer, RecommendedCache,
    notify::{RecommendedWatcher, RecursiveMode},
};

use crate::{
    asset::InternalAssetEvent, fs::{Bytes, Fs, ReadError}, path::{AssetPath, ProjectAssetResolver}
};

/// A filesystem implementation for the editor.
pub struct EditorFs {
    resolver: ProjectAssetResolver,
    watcher: Watcher,
}

impl EditorFs {
    pub fn new(resolver: ProjectAssetResolver, event_tx: Sender<InternalAssetEvent>) -> Self {
        let watcher = Watcher::new(resolver.clone(), event_tx);

        Self {
            resolver,
            watcher,
        }
    }
}

impl Fs for EditorFs {
    fn read(&self, path: &AssetPath) -> Result<Bytes, ReadError> {
        let path = self.resolver.resolve(&path);

        match std::fs::read(path) {
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
        project_asset_resolver: ProjectAssetResolver,
        event_tx: Sender<InternalAssetEvent>,
    ) -> Self {
        let mut debouncer = {
            let asset_path_resolver = project_asset_resolver.clone();

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
                        asset_path_resolver: &ProjectAssetResolver,
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

                        // Unresolve the path.
                        let path = match asset_path_resolver.unresolve(path) {
                            Ok(path) => path,
                            Err(error) => {
                                dbg!(&error);
                                return;
                            }
                        };

                        // Send the event.
                        let _ = event_tx.send(InternalAssetEvent::Changed(path));
                    }

                    match event {
                        Ok(events) => {
                            for event in events {
                                process_event(event, &asset_path_resolver, &event_tx);
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
            .watch(project_asset_resolver.root(), RecursiveMode::Recursive)
            .unwrap();

        Self { debouncer }
    }
}
