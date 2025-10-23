use std::collections::HashSet;
use std::sync::Arc;

use thiserror::Error;

use crate::asset::Asset;
use crate::fs::{Bytes, ReadError};
use crate::path::AssetPath;
use crate::vfs::VirtualFileSystem;

/// A collection of asset loaders.
#[derive(Default)]
pub struct AssetLoadersBuilder {
    pub loaders: Vec<Arc<dyn ErasedAssetLoader>>,
}

impl AssetLoadersBuilder {
    /// Adds a loader to the collection.
    pub fn add<L>(&mut self, loader: L) -> &mut Self
    where
        L: ErasedAssetLoader + 'static,
    {
        self.loaders.push(Arc::new(loader));
        self
    }

    /// Builds the collection of loaders.
    pub fn build(self) -> AssetLoaders {
        AssetLoaders {
            loaders: Arc::from_iter(self.loaders),
        }
    }
}

#[derive(Clone)]
pub struct AssetLoaders {
    /// The loaders for the assets.
    pub loaders: Arc<[Arc<dyn ErasedAssetLoader>]>,
}

impl AssetLoaders {
    /// Finds a loader by extension.
    pub fn find_by_extension(&self, extension: &str) -> Option<Arc<dyn ErasedAssetLoader>> {
        self.loaders
            .iter()
            .find(|loader| loader.extensions().contains(&extension))
            .cloned()
    }
}

static_assertions::assert_impl_all!(AssetLoaders: Send, Sync);

/// The context for loading assets.
pub struct LoadContext {
    /// The path of the asset to load.
    pub path: AssetPath,

    /// The virtual filesystem for assets.
    pub virtual_fs: VirtualFileSystem,

    /// All the available asset loaders.
    pub loaders: AssetLoaders,

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

        // Get the loader reference
        let loader = self.loaders.find_by_extension(ext)
            .ok_or(LoadError::NoLoaderFound)?;
        
        let result = loader.load(self)?;
        
        Ok(result)
    }

    /// Reads an asset from the filesystem.
    pub fn read(&mut self, path: &AssetPath) -> Result<Bytes, LoadError> {
        let resolved_path = self.virtual_fs.resolve(path)
            .ok_or(LoadError::ReadError(ReadError::NotFound))?;

        let bytes = resolved_path.fs.read(&resolved_path.path)?;

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
