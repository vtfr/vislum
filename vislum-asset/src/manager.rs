use std::{collections::HashSet, path::Path, sync::{Arc, Mutex}};

use crossbeam::channel::{Receiver, Sender};
use thiserror::Error;

use crate::{
    asset::{Asset, InternalAssetEvent, LoadAssetCompletionEvent}, database::{AssetDatabase, AssetState}, fs::{Fs, memory::MemoryFs}, loader::{AssetLoaders, ErasedAssetLoader, LoadContext}, path::{AssetPath, VirtualFileSystem}, virfs::{VirtualFileSystem, VirtualFileSystemEntry}
};

/// The shared, mutable state of the asset manager.
#[derive(Default)]
pub struct AssetManagerShared {
    /// The database for the assets.
    database: AssetDatabase,

    /// The virtual filesystem for the assets.
    virtual_fs: VirtualFileSystem,
}

impl AssetManagerShared {
    /// Adds a virtual filesystem entry to the asset manager.
    pub fn add_virtual_fs(&mut self, virtual_fs: VirtualFileSystemEntry) {
        let replaced =self.virtual_fs.add(virtual_fs);
        
        // Untrack removed resources.
        if let Some(replaced) = replaced {
            self.database.set_asset_failed(replaced.root().clone(), "Virtual filesystem entry replaced".to_string());
        }
    }
}

/// The asset manager.
#[derive(Clone)]
pub struct AssetManager {
    /// The receiver for the internal events.
    internal_events_rx: Receiver<InternalAssetEvent>,
    
    /// The sender for the internal events.
    internal_events_tx: Sender<InternalAssetEvent>,
    
    /// The loaders for the assets.
    loaders: Arc<[Box<dyn ErasedAssetLoader>]>,
    
    /// The shared, mutable state of the asset manager.
    shared: Arc<Mutex<AssetManagerShared>>,
}

static_assertions::assert_impl_all!(AssetManager: Send, Sync);

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("The asset is not loaded. Retry in a couple frames.")]
    Loading,
    #[error("The asset is not found.")]
    NotFound,
    #[error("The asset failed to load. Retry in a couple frames.")]
    Failed,
    #[error("Incompatible asset type.")]
    IncompatibleType,
}

impl AssetManager {
    pub fn new_with_loaders(loaders: Arc<[Box<dyn ErasedAssetLoader>]>) -> Self {
        let (internal_events_tx, internal_events_rx) =
            crossbeam::channel::unbounded::<InternalAssetEvent>();

        Self {
            internal_events_rx,
            internal_events_tx,
            loaders,
            shared: Arc::new(Mutex::new(AssetManagerShared {
                database: Default::default(),
                virtual_fs: Default::default(),
            })),
        }
    }

    pub fn get<T: Asset>(&self, path: &AssetPath) -> Result<Arc<T>, AssetError> {
        let asset = self.get_untyped(path)?;
        match asset.clone().downcast_arc::<T>() {
            Ok(asset) => Ok(asset),
            Err(_) => Err(AssetError::IncompatibleType),
        }
    }

    pub fn get_untyped(&self, path: &AssetPath) -> Result<Arc<dyn Asset>, AssetError> {
        let shared = self.shared.lock().unwrap();
        let asset_entry = shared.database.get_entry(path)
            .ok_or(AssetError::NotFound)?;

        match asset_entry.state() {
            AssetState::Loaded(asset) => Ok(asset.clone()),
            AssetState::Loading => Err(AssetError::Loading),
            AssetState::Failed(_) => Err(AssetError::Failed),
        }
    }

    /// Loads an asset.
    /// 
    /// Asset loading is done in the background. Once an asset is loaded, 
    /// callers can retrieve it by calling [`AssetManager::get`].
    pub fn load(&mut self, path: AssetPath) {
        // If the asset is already being loaded, return.
        let mut shared = self.shared.lock().unwrap();
        // if shared.loading.contains(&path) {
        //     return;
        // }

        // Add the asset to the loading set.
        // shared.loading.insert(path.clone());

        let mut load_context = LoadContext {
            path: path.clone(),
            vislum_fs: shared.vislum_fs.clone(),
            loaders: shared.loaders.clone(),
            project_fs: shared.project_context.as_ref().map(|ctx| ctx.fs.clone()),
            dependencies: Default::default(),
        };
        
        let internal_events_tx = self.internal_events_tx.clone();

        // Spawn a thread to load the asset.
        std::thread::spawn(move || {
            let result = load_context.load(&path);

            let _ = internal_events_tx.send(InternalAssetEvent::Loaded(LoadAssetCompletionEvent {
                path,
                result,
                dependencies: load_context.dependencies,
            }));
        });
    }

    /// Processes the AssetManager events.
    /// 
    /// Returns a list of changed assets, and the type of change.
    pub fn process_events(&mut self) {
        let mut changed_paths = HashSet::new();
        
        for event in self.internal_events_rx.try_iter() {
            match event {
                InternalAssetEvent::Created(_asset_path) => {},
                InternalAssetEvent::Changed(asset_path) => {
                    // Collect the changed assets.
                    changed_paths.insert(asset_path);
                },
                InternalAssetEvent::Loaded(loaded_asset_event) => {
                    // Remove the asset from the loading set.
                    let mut shared = self.shared.lock().unwrap();

                    // Set the asset in the database.
                    match loaded_asset_event.result {
                        Ok(asset) => {
                            shared.database.set_asset_loaded(loaded_asset_event.path, asset, loaded_asset_event.dependencies);
                        },
                        Err(_) => {
                            shared.database.set_asset_failed(loaded_asset_event.path, "Failed to load asset".to_string());
                        },
                    }
                },
            }
        }
    }
}
