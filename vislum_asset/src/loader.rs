use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use thiserror::Error;

use crate::{Asset, AssetPath, AssetPathResolver, Fs, FsError};

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("The asset was not found: {0}")]
    NotFound(AssetPath<'static>),

    #[error("The asset has a dependency that was not found: {0}")]
    DependencyNotFound(AssetPath<'static>),

    #[error("Dependency cycle detected")]
    DependencyCycle(Vec<AssetPath<'static>>),

    #[error("The asset is invalid: {0}")]
    Custom(String),
}

/// A context for loading an asset.
pub struct LoadContext {
    /// The resolver for asset paths.
    pub asset_path_resolver: AssetPathResolver,

    /// The filesystem.
    pub fs: Arc<dyn Fs>,

    /// The registry of loaders.
    pub loaders: Arc<LoaderRegistry>,

    /// The dependencies of the asset.
    pub dependencies: HashSet<AssetPath<'static>>,
}

impl LoadContext {
    /// Reads a file from the filesystem.
    pub fn read_file<'a>(&mut self, path: AssetPath<'a>) -> Result<Arc<[u8]>, LoadError> {
        let resolved_path = self.asset_path_resolver.resolve(path.path());

        let bytes = match self.fs.read(&resolved_path) {
            Ok(bytes) => bytes,
            Err(FsError::NotFound) => return Err(LoadError::NotFound(path.into_owned())),
        };

        // Add the dependency to the context.
        self.dependencies.insert(path.into_owned());

        Ok(bytes)
    }

    pub fn load(&mut self, path: AssetPath<'static>) -> Result<Arc<dyn Asset>, LoadError> {
        let ext = path
            .path()
            .extension()
            .ok_or_else(|| LoadError::NotFound(path.into_owned()))?
            .to_str()
            .ok_or_else(|| LoadError::NotFound(path.into_owned()))?;

        let Some(loader) = self.loaders.get(ext) else {
            return Err(LoadError::NotFound(path.into_owned()));
        };

        loader.load(self, path)
    }
}

pub trait AssetLoader: Send + Sync {
    type Asset: Asset;

    /// All the extensions this loader can load.
    fn extensions(&self) -> &'static [&'static str];

    /// Loads the asset.
    fn load(
        &self,
        ctx: &mut LoadContext,
        path: AssetPath<'static>,
    ) -> Result<Self::Asset, LoadError>;
}

pub trait ErasedAssetLoader: Send + Sync {
    fn extensions(&self) -> &'static [&'static str];

    fn load(
        &self,
        ctx: &mut LoadContext,
        path: AssetPath<'static>,
    ) -> Result<Arc<dyn Asset>, LoadError>;
}

impl<T: AssetLoader> ErasedAssetLoader for T {
    fn extensions(&self) -> &'static [&'static str] {
        T::extensions(self)
    }

    fn load(
        &self,
        ctx: &mut LoadContext,
        path: AssetPath<'static>,
    ) -> Result<Arc<dyn Asset>, LoadError> {
        let asset = T::load(self, ctx, path)?;
        Ok(Arc::new(asset))
    }
}

#[derive(Default)]
pub struct LoaderRegistry {
    loaders: HashMap<&'static str, Arc<dyn ErasedAssetLoader>>,
}

impl LoaderRegistry {
    pub fn add<L: ErasedAssetLoader + 'static>(&mut self, loader: L) {
        let loader = Arc::new(loader);
        for extension in loader.extensions() {
            self.loaders.insert(extension, loader.clone());
        }
    }

    pub fn get(&self, extension: &str) -> Option<Arc<dyn ErasedAssetLoader>> {
        self.loaders.get(extension).cloned()
    }
}
