use atomicow::CowArc;
use thiserror::Error;
use std::{fmt::Display, path::{Path, PathBuf}, sync::Arc};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AssetNamespace {
    /// The namespace for Vislum embedded assets, 
    /// e.g. "vislum://shaders/default.wgsl".
    Vislum,
    /// The namespace for project defined assets, 
    /// e.g. "project://textures/texture.png".
    Project,
}

impl Display for AssetNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetNamespace::Vislum => write!(f, "vislum"),
            AssetNamespace::Project => write!(f, "project"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetPath {
    namespace: AssetNamespace,
    path: CowArc<'static, Path>,
}

static_assertions::assert_impl_all!(AssetPath: Send, Sync);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Error)]
#[error("Invalid asset path")]
pub struct AssetPathParseError;

impl AssetPath {
    /// Creates a new asset path.
    pub fn new(namespace: AssetNamespace, path: &Path) -> Self {
        Self {
            namespace,
            path: CowArc::new_owned_from_arc(path),
        }
    }

    /// Creates a new project asset path.
    pub fn new_project(path: &Path) -> Self {
        Self {
            namespace: AssetNamespace::Project,
            path: CowArc::new_owned_from_arc(path),
        }
    }

    pub fn try_parse(path: &str) -> Result<AssetPath, AssetPathParseError> {
        let (namespace, rest) = path.split_once("://")
            .ok_or(AssetPathParseError)?;

        let namespace = match namespace {
            "vislum" => AssetNamespace::Vislum, 
            "project" => AssetNamespace::Project,
            _ => return Err(AssetPathParseError)
        };

        let path = CowArc::new_owned_from_arc(Path::new(rest));

        Ok(AssetPath {
            namespace,
            path,
        })
    }

    /// Returns the namespace of the asset.
    pub fn namespace(&self) -> AssetNamespace {
        self.namespace
    }

    /// Returns the path of the asset.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}", self.namespace, self.path.display())
    }
}

/// A path resolver for project assets.
/// 
/// This path resolver is used to resolve paths for assets that are not bundled into the executable.
#[derive(Debug, Clone)]
pub struct ProjectAssetResolver {
    root: Arc<Path>,
}

#[derive(Debug, Error)]
#[error("Failed to unresolve path")]
pub struct ProjectAssetPathUnresolveError;

impl ProjectAssetResolver {
    /// Creates a new project asset path resolver.
    pub fn new(root: &Path) -> Self {
        Self { root: Arc::from(root) }
    }

    /// Creates a project asset path resolver that does not have a root.
    /// 
    /// Assets paths are resolved as is, without any prefix.
    pub fn none() -> Self {
        Self { root: Arc::from(Path::new("")) }
    }

    /// Resolves a path to a full path.
    pub fn resolve(&self, path: &AssetPath) -> PathBuf {
        let path = path.path();
        
        self.root.join(path)
    }

    /// Unresolves a path to an asset path.
    pub fn unresolve(&self, path: &Path) -> Result<AssetPath, ProjectAssetPathUnresolveError> {
        let path = path.strip_prefix(&self.root)
            .map_err(|_| ProjectAssetPathUnresolveError)?;

        Ok(AssetPath::new_project(path))
    }

    /// Returns the root of the project asset path resolver.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

static_assertions::assert_impl_all!(ProjectAssetResolver: Send, Sync);