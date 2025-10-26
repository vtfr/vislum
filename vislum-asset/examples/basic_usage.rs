use std::sync::Arc;
use vislum_asset::{
    asset::{Asset, AssetId},
    fs::editor::EditorFs,
    loader::{AssetLoader, AssetLoadersBuilder, LoadContext, LoadError},
    manager::AssetManager,
    path::AssetPath,
    vfs::VirtualFileSystemEntry,
};

pub struct FuckerLoader;

#[derive(Debug)]
pub struct Data {
    data: String,
}

impl Asset for Data {}

impl AssetLoader for FuckerLoader {
    type Asset = Data;

    fn extensions(&self) -> &'static [&'static str] {
        &["fuck"]
    }

    fn load(&self, _context: &mut LoadContext) -> Result<Self::Asset, LoadError> {
        Ok(Data {
            data: "fuck".to_string(),
        })
    }
}

fn main() {
    // Create project filesystem (for "project/..." paths)
    let project_fs = Arc::new(EditorFs::new(
        std::path::PathBuf::from("."),
        crossbeam::channel::unbounded().0, // No events for this example
    ));

    // Create vislum filesystem (for "vislum/..." paths)
    // let vislum_fs = Arc::new(MemoryFs::new());
    // Add some embedded assets to vislum_fs here...

    // Create asset loaders
    let mut loaders = AssetLoadersBuilder::default();
    loaders.add(FuckerLoader);
    let loaders = loaders.build();

    // Create asset manager
    let mut manager = AssetManager::new(loaders);

    // Add the virtual filesystem entries to the manager
    manager.add_virtual_fs(VirtualFileSystemEntry::new(
        AssetPath::new_owned("project"),
        true,
        project_fs.clone(),
    ));

    // Load an asset
    let asset_path = AssetPath::new_owned("project/fuck.fuck");
    let asset_id: AssetId = manager.load(asset_path);

    // Process events to handle loading completion
    while !manager.ready() {
        println!("Waiting for assets to load...");
        manager.process_events();
    }

    // Try to get the asset (it might still be loading)
    match manager.get::<Data>(asset_id) {
        Some(asset) => println!("Asset loaded successfully! {:?}", asset.data),
        None => println!("Asset not ready yet"),
    }
}
