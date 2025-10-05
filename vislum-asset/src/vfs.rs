use std::sync::Arc;

use crate::{fs::Fs, path::AssetPath};

/// A virtual filesystem.
#[derive(Default, Clone)]
pub struct FileSystemRouter {
    entries: Vec<VirtualFileSystemEntry>,
}

impl FileSystemRouter {
    /// Adds a new virtual filesystem entry.
    /// 
    /// If an entry with the same root already exists, it will be replaced.
    pub fn add(&mut self, entry: VirtualFileSystemEntry) -> Option<VirtualFileSystemEntry> {
        let old_entry = self.entries.iter_mut()
            .find(|e| e.root == entry.root);

        if let Some(old_entry) = old_entry {
            Some(std::mem::replace(old_entry, entry))
        } else {
            self.entries.push(entry);
            None
        }
    }

    /// Resolves a path to a virtual asset path.
    pub fn resolve(&self, path: &AssetPath) -> Option<ResolvedVirtualAssetPath> {
        let entry = self.entries.iter()
            .find(|e| e.matches(path))?;

        // Computes the path relative to the entry root.
        let path = if entry.strip_prefix {
            path.strip_prefix(entry.root())?
        } else {
            path.to_owned()
        };

        Some(ResolvedVirtualAssetPath {
            path,
            virtual_fs: entry.clone(),
        })
    }

    /// Routes a path to the appropriate filesystem.
    pub fn route(&self, path: &AssetPath) -> Option<&dyn Fs> {
        let entry = self.entries.iter()
            .find(|e| e.matches(path))?;
        
        Some(entry.fs())
    }
}

/// A virtual filesystem entry.
#[derive(Clone)]
pub struct VirtualFileSystemEntry {
    /// The root prefix of the entry.
    /// 
    /// [`AssetPath`]s starting with this prefix will be resolved to this entry.
    pub root: AssetPath,

    /// Whether to strip the root prefix from the resolved path.
    pub strip_prefix: bool,

    /// The filesystem implementation for the entry.
    pub fs: Arc<dyn Fs>,
}

static_assertions::assert_impl_all!(VirtualFileSystemEntry: Send, Sync);

impl VirtualFileSystemEntry {
    /// Creates a new virtual filesystem entry.
    pub fn new(root: AssetPath, strip_prefix: bool, fs: Arc<dyn Fs>) -> Self {
        Self {
            root,
            strip_prefix,
            fs,
        }
    }

    /// Returns whether the entry matches the given path.
    pub fn matches(&self, path: &AssetPath) -> bool {
        path.path().starts_with(self.root.path())
    }

    /// Returns the root prefix of the entry.
    pub fn root(&self) -> &AssetPath {
        &self.root
    }

    /// Returns the filesystem implementation for the entry.
    pub fn fs(&self) -> &dyn Fs {
        &*self.fs
    }
}

/// A resolved virtual asset path.
pub struct ResolvedVirtualAssetPath {
    /// The path to the asset in the provided filesystem.
    pub path: AssetPath,

    pub virtual_fs: VirtualFileSystemEntry,
}