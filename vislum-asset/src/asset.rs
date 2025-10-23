use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use downcast_rs::DowncastSync;

use crate::loader::LoadError;
use crate::path::AssetPath;

slotmap::new_key_type! {
    /// A unique identifier for an asset.
    pub struct AssetId;
}
pub trait Asset: Send + Sync + DowncastSync {}

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
pub struct LoadAssetCompletionEvent {
    /// The ID of the asset that was loaded.
    pub id: AssetId,

    /// The path of the asset that was requested to be loaded.
    pub path: AssetPath,

    /// The resolved filesystem path of the asset that was loaded.
    pub filesystem_path: Option<PathBuf>,

    /// The result of the loading operation.
    pub result: Result<Arc<dyn Asset>, LoadError>,

    /// The dependencies of the asset.
    pub dependencies: HashSet<AssetPath>,
}

static_assertions::assert_impl_all!(InternalAssetEvent: Send, Sync);
