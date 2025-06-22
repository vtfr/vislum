use std::any::Any;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{AssetPath, LoadError};

pub trait Asset: Any + Send + Sync {}

#[derive(Debug, Clone)]
pub struct AssetPathResolver {
    root: Arc<Path>,
}

impl AssetPathResolver {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: Arc::from(root.as_ref()),
        }
    }

    /// Resolves an asset path to an absolute path.
    pub fn resolve(&self, path: impl AsRef<Path>) -> PathBuf {
        self.root.join(path)
    }

    /// Unresolves an absolute path to a relative path.
    pub fn unresolve<'a>(&self, path: &'a Path) -> Option<AssetPath<'a>> {
        let stripped = path.strip_prefix(self.root.as_ref()).ok()?;

        Some(AssetPath::from_path(stripped))
    }

    pub fn root_path(&self) -> &Path {
        &self.root
    }
}

pub enum AssetEvent {
    Changed(AssetPath<'static>),
    Loaded(LoadedAsset),
}

pub struct LoadedAsset {
    pub path: AssetPath<'static>,
    pub result: Result<Arc<dyn Asset>, LoadError>,
    pub dependencies: HashSet<AssetPath<'static>>,
}
