use std::path::Path;

use atomicow::CowArc;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetPath<'a> {
    path: CowArc<'a, Path>,
}

impl AssetPath<'_> {
    /// Creates a new asset path from a string.
    pub fn new(path: &str) -> AssetPath {
        AssetPath {
            path: CowArc::Borrowed(Path::new(path)),
        }
    }

    /// Creates a new asset path from a string.
    pub fn new_owned(path: &str) -> AssetPath<'static> {
        AssetPath {
            path: CowArc::new_owned_from_arc(Path::new(path)),
        }
    }

    /// Creates a new asset path from a path.
    pub fn from_path(path: &Path) -> AssetPath {
        AssetPath {
            path: CowArc::Borrowed(path),
        }
    }

    /// Returns the path as a reference.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Converts the path to an owned path.
    pub fn into_owned(&self) -> AssetPath<'static> {
        AssetPath {
            path: self.path.clone().into_owned(),
        }
    }
}

impl Display for AssetPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path().display())
    }
}
