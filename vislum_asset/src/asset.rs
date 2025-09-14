use std::collections::HashSet;
use std::sync::Arc;

use downcast_rs::{DowncastSync};

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