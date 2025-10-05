use std::collections::HashSet;
use std::{collections::HashMap, sync::Arc};

use crate::asset::Asset;
use crate::path::AssetPath;

#[derive(Default)]
pub struct AssetDatabaseEntry {
    /// The state of the asset.
    state: AssetState,

    /// The dependencies of the asset.
    dependencies: HashSet<AssetPath>,
}

impl AssetDatabaseEntry {
    /// Returns the state of the asset.
    pub fn state(&self) -> &AssetState {
        &self.state
    }

    /// Returns the dependencies of the asset.
    pub fn dependencies(&self) -> &HashSet<AssetPath> {
        &self.dependencies
    }
}

#[derive(Default)]
pub enum AssetState {
    #[default]
    Loading,
    Loaded(Arc<dyn Asset>),
    Failed(String),
}

#[derive(Default)]
pub struct AssetDatabase {
    database: HashMap<AssetPath, AssetDatabaseEntry>,
}

impl AssetDatabase {
    /// Returns an entry for the given path.
    pub fn get_entry(&self, path: &AssetPath) -> Option<&AssetDatabaseEntry> {
        self.database.get(path)
    }

    /// Updates an asset to the loading state.
    /// 
    /// Creates a new asset entry if it does not exist.
    pub fn set_asset_loading(&mut self, path: AssetPath) {
        self.database.entry(path)
            .and_modify(|entry| {
                entry.state = AssetState::Loading;
            })
            .or_default();
    }

    /// Updates an asset to the loaded state.
    pub fn set_asset_loaded(&mut self, path: AssetPath, asset: Arc<dyn Asset>, dependencies: HashSet<AssetPath>) {
        let entry = self.database.entry(path)
            .or_default();

        entry.state = AssetState::Loaded(asset);
        entry.dependencies = dependencies;
    }

    /// Updates an asset to the failed state.
    pub fn set_asset_failed(&mut self, path: AssetPath, error: String) {
        let entry = self.database.entry(path)
            .or_default();

        entry.state = AssetState::Failed(error);
    }
}
