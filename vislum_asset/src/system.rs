use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use crossbeam::channel::{Receiver, Sender};

use crate::{
    Asset, AssetEvent, AssetPath, AssetPathResolver, Fs, LoadContext, LoadError, LoadedAsset,
    LoaderRegistry,
};

/// A system for loading and managing assets.
pub struct AssetSystem {
    events_rx: Receiver<AssetEvent>,
    events_tx: Sender<AssetEvent>,
    loaders: Arc<LoaderRegistry>,
    asset_path_resolver: AssetPathResolver,
    fs: Arc<dyn Fs>,

    assets: HashMap<AssetPath<'static>, AssetState>,
    loading: HashSet<AssetPath<'static>>,
}

impl AssetSystem {
    pub fn new(
        root: impl AsRef<Path>,
        loaders: Arc<LoaderRegistry>,
        fs_factory: impl FnOnce(AssetPathResolver, Sender<AssetEvent>) -> Arc<dyn Fs>,
    ) -> Self {
        let (events_tx, events_rx) = crossbeam::channel::unbounded();
        let resolver = AssetPathResolver::new(root);
        let fs = fs_factory(resolver.clone(), events_tx.clone());

        Self {
            events_rx,
            events_tx,
            loaders,
            asset_path_resolver: resolver,
            fs,
            assets: Default::default(),
            loading: Default::default(),
        }
    }

    /// Returns true if the asset system is ready to retrieve all loaded assets.
    pub fn ready(&self) -> bool {
        self.loading.is_empty()
    }

    pub fn get<T: Asset>(&self, path: AssetPath<'_>) -> Option<Arc<T>> {
        let state = self.assets.get(&path)?;

        match &state.loading_state {
            LoadingState::Loaded(asset) => Arc::downcast::<T>(asset.clone()).ok(),
            _ => None,
        }
    }

    pub fn process_events(&mut self) {
        while let Ok(event) = self.events_rx.try_recv() {
            match event {
                AssetEvent::Loaded(LoadedAsset {
                    path,
                    result,
                    dependencies,
                }) => {
                    // Remove the asset from the loading set.
                    self.loading.remove(&path);

                    let state = self.assets.entry(path).or_insert_with(|| AssetState {
                        loading_state: LoadingState::Loading,
                        dependencies: HashSet::new(),
                    });

                    match result {
                        Ok(asset) => {
                            // Update the asset state to loaded.
                            state.loading_state = LoadingState::Loaded(asset);
                            state.dependencies = dependencies;
                        }
                        Err(e) => {
                            // Update the asset state to failed. Keep the dependencies as we might need them to
                            // retry loading.
                            state.loading_state = LoadingState::Failed(e);
                        }
                    }
                }
                AssetEvent::Changed(path_buf) => {
                    // Track all pending assets.
                    let mut pending = HashSet::new();

                    for (path, state) in self.assets.iter() {
                        if state.dependencies.contains(&path_buf) {
                            pending.insert(path.clone());
                        }
                    }

                    // Reload async.
                    for path in pending {
                        self.load_async(path);
                    }
                }
            }
        }
    }

    pub fn load_async<'a>(&mut self, path: AssetPath<'a>) {
        let path = path.into_owned();
        // Track this asset as loading.
        self.assets.insert(
            path.clone(),
            AssetState {
                loading_state: LoadingState::Loading,
                dependencies: HashSet::new(),
            },
        );

        self.loading.insert(path.clone());

        // Create a load context.
        let mut load_context = LoadContext {
            asset_path_resolver: self.asset_path_resolver.clone(),
            fs: self.fs.clone(),
            dependencies: HashSet::new(),
            loaders: self.loaders.clone(),
        };

        let events_tx = self.events_tx.clone();

        std::thread::spawn(move || {
            let result = load_context.load(path.clone());

            let _ = events_tx.send(AssetEvent::Loaded(LoadedAsset {
                path,
                result,
                dependencies: load_context.dependencies,
            }));
        });
    }
}

enum LoadingState {
    Loading,
    Loaded(Arc<dyn Asset>),
    Failed(LoadError),
}

struct AssetState {
    loading_state: LoadingState,
    dependencies: HashSet<AssetPath<'static>>,
}
