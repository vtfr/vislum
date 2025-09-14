use std::collections::HashSet;
use std::sync::Arc;

use downcast_rs::{DowncastSync};

use crate::fs::Bytes;
use crate::loader::LoadError;
use crate::path::AssetPath;

pub trait Asset: Send + Sync + DowncastSync { }

downcast_rs::impl_downcast!(sync Asset);
pub(crate) enum InternalAssetEvent {
    /// An asset has been created.
    Created(AssetPath),
    /// An asset has been changed.
    Changed(AssetPath),
    /// An asset has been loaded.
    Loaded(LoadedAssetEvent),
}

static_assertions::assert_impl_all!(InternalAssetEvent: Send, Sync);

pub(crate) struct LoadedAssetEvent {
    pub path: AssetPath,
    pub result: Result<Arc<dyn Asset>, LoadError>,
    pub dependencies: HashSet<AssetPath>,
}

static_assertions::assert_impl_all!(InternalAssetEvent: Send, Sync);

/// A trait for assets that are embedded in the project.
pub struct EmbeddedAsset {
    path: AssetPath,
    bytes: Bytes,
}

impl EmbeddedAsset {
    pub const fn new(path: &'static str, bytes: &'static [u8]) -> Self {
        Self {
            path: AssetPath::new_embedded(path),
            bytes: Bytes::new_static(bytes),
        }
    }
}

#[macro_export]
macro_rules! embedded_asset {
    ($path_str:expr => $bytes:expr) => {{
        $crate::asset::EmbeddedAsset::new($path_str, $bytes)
    }}
}

pub enum AssetEvent {
    /// An asset has been created.
    Created(AssetPath),
    /// An asset has been changed.
    Changed(AssetPath),
    /// An asset has been deleted.
    Deleted(AssetPath),
}