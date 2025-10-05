use std::collections::HashSet;
use std::sync::Arc;

use thiserror::Error;

use crate::asset::Asset;
use crate::fs::{Bytes, ReadError};
use crate::path::AssetPath;
use crate::vfs::FileSystemRouter;

/// A collection of asset loaders.
pub struct AssetLoaders {
    pub loaders: Vec<Box<dyn ErasedAssetLoader>>,
}

impl AssetLoaders {
    /// Creates a new collection of asset loaders.
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
        }
    }

    /// Adds a loader to the collection.
    pub fn add<L>(&mut self, loader: L)
    where
        L: ErasedAssetLoader + 'static,
    {
        self.loaders.push(Box::new(loader));
    }

    /// Converts to a slice of loaders.
    pub fn as_slice(&self) -> &[Box<dyn ErasedAssetLoader>] {
        &self.loaders
    }

    /// Finds a loader by extension.
    pub fn find_by_extension(&self, extension: &str) -> Option<&dyn ErasedAssetLoader> {
        self.loaders
            .iter()
            .find(|loader| loader.extensions().contains(&extension))
            .map(|loader| &**loader)
    }
}

/// The context for loading assets.
pub struct LoadContext {
    /// The path of the asset to load.
    pub path: AssetPath,

    /// The virtual filesystem for assets.
    pub virtual_fs: FileSystemRouter,

    /// All the available asset loaders.
    pub loaders: Arc<AssetLoaders>,

    /// All collected dependencies.
    pub dependencies: HashSet<AssetPath>,
}

impl LoadContext {
    pub fn load(&mut self, path: &AssetPath) -> Result<Arc<dyn Asset>, LoadError> {
        let ext = path
            .path()
            .extension()
            .ok_or(LoadError::NoLoaderFound)?
            .to_str()
            .unwrap();

        // Find the loader index by extension
        let loader_index = self.loaders
            .loaders
            .iter()
            .position(|loader| loader.extensions().contains(&ext))
            .ok_or(LoadError::NoLoaderFound)?;

        // Get the loader reference
        let loader = &*self.loaders.loaders[loader_index];
        
        // Create a temporary context to avoid borrowing conflicts
        let mut temp_context = LoadContext {
            path: self.path.clone(),
            virtual_fs: self.virtual_fs.clone(),
            loaders: Arc::new(AssetLoaders {
                loaders: vec![], // Empty for temp context
            }),
            dependencies: HashSet::new(),
        };

        let result = loader.load(&mut temp_context)?;
        
        // Merge dependencies back
        self.dependencies.extend(temp_context.dependencies);
        
        Ok(result)
    }

    /// Reads an asset from the filesystem.
    pub fn read(&mut self, path: &AssetPath) -> Result<Bytes, LoadError> {
        let fs = self.virtual_fs.route(path)
            .ok_or(LoadError::ReadError(ReadError::NotFound))?;
        
        let bytes = fs.read(path)?;

        // If the path is not the same as the path of the asset to load,
        // then track it as a dependency.
        if path != &self.path {
            self.dependencies.insert(path.clone());
        }

        Ok(bytes)
    }
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Read error: {0}")]
    ReadError(#[from] ReadError),
    #[error(
        "The project is not loaded. Please open a project first before loading project assets."
    )]
    ProjectNotLoaded,
    #[error("No loader found for the given path")]
    NoLoaderFound,
}

pub trait AssetLoader: Send + Sync {
    type Asset: Asset;

    /// Returns the extensions that this loader can load.
    fn extensions(&self) -> &'static [&'static str];

    /// Loads an asset from the filesystem.
    fn load(&self, context: &mut LoadContext) -> Result<Self::Asset, LoadError>;
}

pub trait ErasedAssetLoader: Send + Sync {
    fn extensions(&self) -> &'static [&'static str];

    fn load(&self, context: &mut LoadContext) -> Result<Arc<dyn Asset>, LoadError>;
}

impl<L> ErasedAssetLoader for L
where
    L: AssetLoader,
{
    fn extensions(&self) -> &'static [&'static str] {
        L::extensions(self)
    }

    fn load(&self, context: &mut LoadContext) -> Result<Arc<dyn Asset>, LoadError> {
        match L::load(self, context) {
            Ok(asset) => Ok(Arc::new(asset)),
            Err(e) => Err(e),
        }
    }
}

impl AssetLoaders {
    /// Resolves a loader for the given extension.
    pub fn resolve(&self, extension: &str) -> Option<&dyn ErasedAssetLoader> {
        self.loaders
            .iter()
            .find(|loader| loader.extensions().contains(&extension))
            .map(|loader| &**loader)
    }
}
