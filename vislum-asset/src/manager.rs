use std::{collections::HashSet, path::Path, sync::{Arc, Mutex}};

use crossbeam::channel::{Receiver, Sender};
use thiserror::Error;

use crate::{
    asset::{Asset, InternalAssetEvent, LoadedAssetEvent}, database::{AssetDatabase, AssetState}, fs::{Fs, memory::MemoryFs}, loader::{AssetLoaders, LoadContext}, path::{AssetPath, ProjectAssetResolver}
};

struct ProjectContext {
    fs: Arc<dyn Fs>,
}

struct AssetManagerShared {
    /// The context for the current project.
    project_context: Option<ProjectContext>,

    /// The filesystem for the vislum assets.
    vislum_fs: Arc<MemoryFs>,

    /// The loaders for the assets.
    loaders: Arc<AssetLoaders>,

    /// The database for the assets.
    database: AssetDatabase,

    /// The assets that are currently being loaded.
    loading: HashSet<AssetPath>,
}

/// The asset manager.
pub struct AssetManager {
    internal_events_rx: Receiver<InternalAssetEvent>,
    internal_events_tx: Sender<InternalAssetEvent>,
    shared: Arc<Mutex<AssetManagerShared>>,
}

static_assertions::assert_impl_all!(AssetManager: Send, Sync);

#[derive(Error, Debug)]
pub enum GetError {
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
    pub fn new_with_loaders(loaders: AssetLoaders) -> Self {
        let (internal_events_tx, internal_events_rx) =
            crossbeam::channel::unbounded::<InternalAssetEvent>();

        let vislum_fs = Arc::new(MemoryFs::new());

        Self {
            internal_events_rx,
            internal_events_tx,
            shared: Arc::new(Mutex::new(AssetManagerShared {
                project_context: None,
                vislum_fs,
                loaders: Arc::new(loaders),
                database: Default::default(),
                loading: Default::default(),
            })),
        }
    }

    pub fn get<T: Asset>(&self, path: &AssetPath) -> Result<Arc<T>, GetError> {
        let asset = self.get_untyped(path)?;
        match asset.clone().downcast_arc::<T>() {
            Ok(asset) => Ok(asset),
            Err(_) => Err(GetError::IncompatibleType),
        }
    }

    pub fn get_untyped(&self, path: &AssetPath) -> Result<Arc<dyn Asset>, GetError> {
        let shared = self.shared.lock().unwrap();
        let asset_entry = shared.database.get_entry(path)
            .ok_or(GetError::NotFound)?;

        match asset_entry.state() {
            AssetState::Loaded(asset) => Ok(asset.clone()),
            AssetState::Loading => Err(GetError::Loading),
            AssetState::Failed(_) => Err(GetError::Failed),
        }
    }

    /// Loads an asset.
    /// 
    /// Asset loading is done in the background. Once an asset is loaded, 
    /// callers can retrieve it by calling [`AssetManager::get`].
    pub fn load(&mut self, path: AssetPath) {
        // If the asset is already being loaded, return.
        let mut shared = self.shared.lock().unwrap();
        if shared.loading.contains(&path) {
            return;
        }

        // Add the asset to the loading set.
        shared.loading.insert(path.clone());

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

            let _ = internal_events_tx.send(InternalAssetEvent::Loaded(LoadedAssetEvent {
                path,
                result,
                dependencies: load_context.dependencies,
            }));
        });
    }

    /// Opens a project.
    pub fn open_project(&mut self, project_path: &Path) {
        todo!()
        // // Close the current project, if open.
        // self.close_project();

        // let asset_path_resolver = ProjectAssetResolver::new(project_path);
        // let fs = Arc::new(EditorFs::new(asset_path_resolver.clone()));

        // let project_context = ProjectContext { fs };

        // self.project_context = Some(project_context);
    }

    /// Closes the current project.
    pub fn close_project(&mut self) {
        let mut shared = self.shared.lock().unwrap();
        shared.project_context.take();
    }

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
                    shared.loading.remove(&loaded_asset_event.path);

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
