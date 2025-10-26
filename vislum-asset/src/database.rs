use slotmap::SlotMap;
use std::collections::HashSet;
use std::{collections::HashMap, sync::Arc};

use crate::asset::{Asset, AssetId};
use crate::path::AssetPath;

pub struct AssetDatabaseEntry {
    /// The path of the asset.
    path: AssetPath,

    /// The state of the asset.
    state: AssetState,

    /// The dependencies of the asset (by path, as they may not be loaded yet).
    dependencies: HashSet<AssetPath>,
}

impl AssetDatabaseEntry {
    /// Returns the path of the asset.
    pub fn path(&self) -> &AssetPath {
        &self.path
    }

    /// Returns the state of the asset.
    pub fn state(&self) -> &AssetState {
        &self.state
    }

    /// Returns whether the asset is loaded.
    pub fn loaded(&self) -> bool {
        matches!(self.state, AssetState::Loaded(_))
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
    /// The slotmap storing all assets by their ID.
    assets: SlotMap<AssetId, AssetDatabaseEntry>,

    /// Path to AssetId mapping for quick lookups.
    path_to_id: HashMap<AssetPath, AssetId>,
}

impl AssetDatabase {
    /// Returns an entry for the given path.
    pub fn get_id_by_path(&self, path: &AssetPath) -> Option<AssetId> {
        self.path_to_id.get(path).copied()
    }

    /// Returns an entry for the given AssetId.
    pub fn get(&self, id: AssetId) -> Option<&AssetDatabaseEntry> {
        self.assets.get(id)
    }

    /// Registers a new asset and returns its AssetId.
    pub fn add(&mut self, path: AssetPath) -> AssetId {
        // Check if asset already exists
        if let Some(&id) = self.path_to_id.get(&path) {
            return id;
        }

        // Create new asset entry
        let entry = AssetDatabaseEntry {
            path: path.clone(),
            state: AssetState::Loading,
            dependencies: HashSet::new(),
        };

        // Insert into slotmap and get ID
        let id = self.assets.insert(entry);

        // Update path mapping
        self.path_to_id.insert(path, id);

        id
    }

    /// Returns an iterator over all assets in the database.
    pub fn iter(&self) -> impl Iterator<Item = (AssetId, &AssetDatabaseEntry)> {
        self.assets.iter()
    }

    /// Updates an asset to the loaded state.
    pub fn set_asset_loaded(
        &mut self,
        id: AssetId,
        asset: Arc<dyn Asset>,
        dependencies: HashSet<AssetPath>,
    ) {
        if let Some(entry) = self.assets.get_mut(id) {
            entry.state = AssetState::Loaded(asset);
            entry.dependencies = dependencies;
        }
    }

    /// Updates an asset to the failed state.
    pub fn set_asset_failed(&mut self, id: AssetId, error: String) {
        if let Some(entry) = self.assets.get_mut(id) {
            entry.state = AssetState::Failed(error);
        }
    }

    /// Removes an asset from the database.
    pub fn remove_asset(&mut self, id: AssetId) {
        if let Some(entry) = self.assets.remove(id) {
            self.path_to_id.remove(entry.path());
        }
    }
}
