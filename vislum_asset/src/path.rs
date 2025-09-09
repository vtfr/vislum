use std::path::Path;

use atomicow::CowArc;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetPath<'a> {
    path: CowArc<'a, str>,
}

impl AssetPath<'_> {
    /// Creates a new asset path from a string.
    pub fn new(path: &str) -> AssetPath {
        AssetPath {
            path: CowArc::Borrowed(path),
        }
    }

    pub const fn new_static(path: &'static str) -> AssetPath<'static> {
        AssetPath {
            path: CowArc::Static(path)
        }
    }

    /// Creates a new asset path from a string.
    pub fn new_owned(path: &str) -> AssetPath<'static> {
        AssetPath {
            path: CowArc::new_owned_from_arc(path),
        }
    }

    /// Creates a new asset path from a path.
    pub fn from_path(path: &Path) -> AssetPath {
        AssetPath {
            path: CowArc::Borrowed(path.to_str().unwrap()),
        }
    }

    /// Returns the path as a reference.
    pub fn path(&self) -> &Path {
        Path::new(self.path.as_ref())
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

/// A bundled asset.
///
/// An asset that is bundled into the executable.
pub struct BundledAsset {
    path: AssetPath<'static>,
    data: &'static [u8],
}

impl BundledAsset {
    pub const fn new(path: AssetPath<'static>, data: &'static [u8]) -> Self {
        Self { path, data }
    }
}

#[macro_export]
macro_rules! embed {
    ($path:expr => $data:expr) => {
        $crate::BundledAsset::new($crate::AssetPath::new_static($path), $data)
    };
}
