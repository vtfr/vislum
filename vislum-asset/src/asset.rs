use std::collections::HashSet;
use std::sync::Arc;

use downcast_rs::{DowncastSync};
use slotmap::DefaultKey;

use crate::loader::LoadError;
use crate::path::AssetPath;

/// A unique identifier for an asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetId(DefaultKey);

impl AssetId {
    /// Creates a new AssetId from a slotmap key.
    pub fn new(key: DefaultKey) -> Self {
        Self(key)
    }

    /// Returns the underlying slotmap key.
    pub fn key(self) -> DefaultKey {
        self.0
    }
}

pub trait Asset: Send + Sync + DowncastSync { }

downcast_rs::impl_downcast!(sync Asset);
pub enum InternalAssetEvent {
    /// An asset has been created.
    Created(AssetPath),
    /// An asset has been changed.
    Changed(AssetPath),
    /// An asset has been loaded.
    Loaded(LoadAssetCompletionEvent),
}

static_assertions::assert_impl_all!(InternalAssetEvent: Send, Sync);

/// Internal event for when an asset has finished loading.
pub(crate) struct LoadAssetCompletionEvent {
    /// The path of the asset that was requested to be loaded.
    pub path: AssetPath,

    /// The result of the loading operation.
    pub result: Result<Arc<dyn Asset>, LoadError>,

    /// The dependencies of the asset.
    pub dependencies: HashSet<AssetPath>,
}

static_assertions::assert_impl_all!(InternalAssetEvent: Send, Sync);
