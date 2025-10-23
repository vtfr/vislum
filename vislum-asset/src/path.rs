use atomicow::CowArc;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetPath(CowArc<'static, Path>);

static_assertions::assert_impl_all!(AssetPath: Send, Sync);

impl Serialize for AssetPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.to_str().unwrap())
    }
}

impl<'de> Deserialize<'de> for AssetPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Ok(AssetPath::new_owned(s))
    }
}

static_assertions::assert_impl_all!(AssetPath: Send, Sync);

impl AssetPath {
    /// Creates a new owned asset path.
    pub fn new_owned(path: impl AsRef<Path>) -> AssetPath {
        AssetPath(CowArc::new_owned_from_arc(path.as_ref()))
    }

    /// Creates a new asset path from a static path.
    pub const fn new_static(path: &'static Path) -> AssetPath {
        AssetPath(CowArc::Static(path))
    }

    /// Creates a new asset path from a borrowed path.
    pub const fn new_borrowed(path: &'static Path) -> AssetPath {
        AssetPath(CowArc::Borrowed(path))
    }

    // /// Converts the asset path to an owned path.
    // pub fn into_owned(self) -> AssetPath {
    //     AssetPath(self.0.into_owned())
    // }

    // /// Converts the asset path to an owned path.
    // pub fn to_owned(&self) -> AssetPath {
    //     AssetPath(self.0.clone_owned())
    // }

    /// Returns the path of the asset.
    pub fn path(&self) -> &Path {
        &*self.0
    }

    pub fn starts_with(&self, other: &AssetPath) -> bool {
        self.path().starts_with(other.path())
    }

    pub fn strip_prefix(&self, prefix: &AssetPath) -> Option<AssetPath> {
        self.path()
            .strip_prefix(prefix.path())
            .ok()
            .map(|p| AssetPath::new_owned(p))
    }
}

impl Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}
