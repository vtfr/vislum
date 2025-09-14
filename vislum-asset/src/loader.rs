use std::collections::HashSet;
use std::sync::Arc;

use thiserror::Error;

use crate::asset::Asset;
use crate::fs::memory::MemoryFs;
use crate::fs::{Bytes, Fs, ReadError};
use crate::path::{AssetNamespace, AssetPath};

/// The context for loading assets.
pub struct LoadContext {
    /// The path of the asset to load.
    pub path: AssetPath,

    /// The filesystem for the vislum assets.
    pub vislum_fs: Arc<MemoryFs>,

    /// All the available asset loaders.
    pub loaders: Arc<AssetLoaders>,

    /// The filesystem for the project.
    pub project_fs: Option<Arc<dyn Fs>>,

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

        let loader = self.loaders.resolve(ext).ok_or(LoadError::NoLoaderFound)?;

        loader.load(self)
    }

    /// Reads an asset from the filesystem.
    pub fn read(&mut self, path: &AssetPath) -> Result<Bytes, LoadError> {
        let bytes = match path.namespace() {
            AssetNamespace::Vislum => self.vislum_fs.read(path)?,
            AssetNamespace::Project => match self.project_fs.as_ref() {
                Some(project_fs) => project_fs.read(path)?,
                None => return Err(LoadError::ProjectNotLoaded),
            },
        };

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

pub struct AssetLoaders {
    loaders: Vec<Arc<dyn ErasedAssetLoader>>,
}

impl AssetLoaders {
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
        }
    }

    /// Adds a loader to the loaders.
    pub fn add<L>(&mut self, loader: L)
    where
        L: ErasedAssetLoader + 'static,
    {
        self.loaders.push(Arc::new(loader));
    }

    /// Resolves a loader for the given extension.
    pub fn resolve(&self, extension: &str) -> Option<Arc<dyn ErasedAssetLoader>> {
        self.loaders
            .iter()
            .find(|loader| loader.extensions().contains(&extension))
            .cloned()
    }
}
