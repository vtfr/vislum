use std::{
    path::{Path, PathBuf}, sync::Arc
};

use atomicow::CowArc;
use crossbeam::channel::{Receiver, Sender, TryIter};

pub mod physical;
pub mod watcher;

pub use physical::PhysicalFsWatcher;
pub use watcher::PhysicalFs;

#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("the file was not found: {0}")]
    NotFound(PathBuf),
    #[error("a physical I/O error occurred at {0}: {1}")]
    PhysicalIo(PathBuf, std::io::Error),
    #[error("unknown namespace: {0:?}")]
    UnknownNamespace(VirtualNamespace),
}

/// An iterator over the files in a directory.
pub type FsIterator = Box<dyn Iterator<Item = PathBuf>>;

/// A filesystem.
pub trait Fs {
    /// Reads a file from the filesystem.
    ///
    /// Returns the contents of the file as a byte array or a file error.
    fn read(&self, path: &Path) -> Result<Arc<[u8]>, FsError>;

    /// Writes a file to the filesystem.
    fn write(&self, path: &Path, data: &[u8]) -> Result<(), FsError>;

    /// Lists the files in a directory.
    ///
    /// If no path is provided, the root directory is listed.
    fn list(&self, path: Option<&Path>) -> FsIterator;
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VirtualNamespace {
    /// The library namespace.
    ///
    /// Stores shared assets that are available to all projects.
    Library,

    /// The project namespace.
    ///
    /// Stores assets that are specific to the project.
    Project,

    /// The shader cache namespace.
    ///
    /// Stores compiled shader modules.
    ShaderCache,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VirtualPath {
    namespace: VirtualNamespace,
    path: CowArc<'static, Path>,
}

impl VirtualPath {
    pub fn parse(path: &str) -> Self {
        let (namespace, path) = path.split_once("://").unwrap();
        let namespace = match namespace {
            "library" => VirtualNamespace::Library,
            "project" => VirtualNamespace::Project,
            "shader-cache" => VirtualNamespace::ShaderCache,
            _ => unreachable!(),
        };

        Self {
            namespace,
            path: CowArc::new_owned_from_arc(Path::new(path)),
        }
    }

    #[inline(always)]
    pub fn namespace(&self) -> VirtualNamespace {
        self.namespace
    }

    #[inline(always)]
    pub fn path(&self) -> &Path {
        &*self.path
    }
    
    pub fn new(namespace: VirtualNamespace, path: &Path) -> Self {
        Self { namespace, path: CowArc::new_owned_from_arc(path) }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FileEventType {
    Created,
    Modified,
    Removed,
}

pub struct FileEvent {
    pub path: VirtualPath,
    pub event_type: FileEventType,
}

/// A trait for watching changes to a filesystem.
pub trait Watcher {
    /// Watches a path for changes.
    fn new(
        namespace: VirtualNamespace,
        root: PathBuf,
        notify: Sender<FileEvent>,
    ) -> Box<dyn Watcher>
    where
        Self: Sized;
}

pub struct VirtualFs {
    entries: Vec<(VirtualNamespace, Box<dyn Fs>)>,
}

impl VirtualFs {
    pub fn new(
        entries: impl IntoIterator<Item = (VirtualNamespace, Box<dyn Fs>)>,
    ) -> Self {
        let entries = entries.into_iter().collect();

        Self { entries }
    }

    pub fn read(&self, path: VirtualPath) -> Result<Arc<[u8]>, FsError> {
        let fs = self.resolve(path.namespace())?;

        fs.read(&path.path)
    }

    pub fn write(&self, path: VirtualPath, data: &[u8]) -> Result<(), FsError> {
        let fs = self.resolve(path.namespace())?;

        fs.write(&path.path, data)
    }

    pub fn list(&self, path: VirtualPath) -> Result<FsIterator, FsError> {
        let fs = self.resolve(path.namespace())?;
        Ok(fs.list(None))
    }

    fn resolve(&self, namespace: VirtualNamespace) -> Result<&dyn Fs, FsError> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.0 == namespace)
            .ok_or(FsError::UnknownNamespace(namespace))?;

        Ok(&*entry.1)
    }

    pub fn events(&self) -> impl Iterator<Item = FileEvent> {
        todo!()
        // self.events.try_iter()
    }
}