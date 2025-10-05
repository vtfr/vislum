use std::{collections::HashSet, sync::{Arc, Mutex}};

use crossbeam::channel::{Receiver, Sender};
use thiserror::Error;

use crate::{
    asset::{Asset, AssetId, InternalAssetEvent, LoadAssetCompletionEvent}, 
    database::{AssetDatabase, AssetState}, 
    loader::{AssetLoaders, LoadContext}, 
    path::AssetPath, 
    vfs::{FileSystemRouter, VirtualFileSystemEntry}
};

/// The shared, mutable state of the asset manager.
#[derive(Default)]
pub struct AssetManagerShared {
    /// The database for the assets.
    database: AssetDatabase,

    /// The virtual filesystem for the assets.
    virtual_fs: FileSystemRouter,
}

impl AssetManagerShared {
    /// Adds a virtual filesystem entry to the asset manager.
    pub fn add_virtual_fs(&mut self, virtual_fs: VirtualFileSystemEntry) {
        let replaced =self.virtual_fs.add(virtual_fs);
        
        // Untrack removed resources.
        if let Some(replaced) = replaced {
            // Find and remove assets that were using the replaced filesystem
            // This is a simplified approach - in practice, you might want to track which assets use which filesystem
            // For now, we'll just log that a filesystem was replaced
            log::warn!("Virtual filesystem entry replaced: {}", replaced.root());
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
    loaders: Arc<AssetLoaders>,
    
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
    pub fn new_with_loaders(loaders: Arc<AssetLoaders>) -> Self {
        let (internal_events_tx, internal_events_rx) =
            crossbeam::channel::unbounded::<InternalAssetEvent>();

        Self {
            internal_events_rx,
            internal_events_tx,
            loaders,
            shared: Arc::new(Mutex::new(AssetManagerShared::default())),
        }
    }

    pub fn get<T: Asset>(&self, id: AssetId) -> Result<Arc<T>, AssetError> {
        let asset = self.get_untyped(id)?;
        match asset.clone().downcast_arc::<T>() {
            Ok(asset) => Ok(asset),
            Err(_) => Err(AssetError::IncompatibleType),
        }
    }

    pub fn get_untyped(&self, id: AssetId) -> Result<Arc<dyn Asset>, AssetError> {
        let shared = self.shared.lock().unwrap();
        let asset_entry = shared.database.get_entry_by_id(id)
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
    /// Returns the AssetId for the asset being loaded.
    pub fn load(&mut self, path: AssetPath) -> AssetId {
        let mut shared = self.shared.lock().unwrap();
        
        // Check if the asset is already being loaded or loaded
        if let Some(entry) = shared.database.get_entry_by_path(&path) {
            match entry.state() {
                AssetState::Loading | AssetState::Loaded(_) => {
                    // Return existing ID
                    return shared.database.get_id_by_path(&path).unwrap();
                },
                AssetState::Failed(_) => {
                    // Retry loading if it previously failed
                }
            }
        }

        // Register the asset and get its ID
        let asset_id = shared.database.register_asset(path.clone());

        let mut load_context = LoadContext {
            path: path.clone(),
            virtual_fs: shared.virtual_fs.clone(),
            loaders: self.loaders.clone(),
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

        asset_id
    }

    /// Adds a virtual filesystem entry to the asset manager.
    pub fn add_virtual_fs(&mut self, virtual_fs: VirtualFileSystemEntry) {
        let mut shared = self.shared.lock().unwrap();
        shared.add_virtual_fs(virtual_fs);
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
                    let mut shared = self.shared.lock().unwrap();

                    // Get the AssetId for this path
                    if let Some(asset_id) = shared.database.get_id_by_path(&loaded_asset_event.path) {
                        // Set the asset in the database.
                        match loaded_asset_event.result {
                            Ok(asset) => {
                                shared.database.set_asset_loaded(asset_id, asset, loaded_asset_event.dependencies);
                            },
                            Err(_) => {
                                shared.database.set_asset_failed(asset_id, "Failed to load asset".to_string());
                            },
                        }
                    }
                },
            }
        }
    }
}
