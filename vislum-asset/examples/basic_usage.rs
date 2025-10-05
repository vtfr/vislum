use std::sync::Arc;
use vislum_asset::{
    asset::AssetId,
    fs::memory::MemoryFs,
    fs::editor::EditorFs,
    loader::AssetLoaders,
    manager::AssetManager,
    path::AssetPath,
    vfs::VirtualFileSystemEntry,
};

fn main() {
    // Create project filesystem (for "project/..." paths)
    let project_fs = Arc::new(EditorFs::new(
        std::path::PathBuf::from("/path/to/project"),
        crossbeam::channel::unbounded().0, // No events for this example
    ));
    
    // Create vislum filesystem (for "vislum/..." paths) 
    let vislum_fs = Arc::new(MemoryFs::new());
    // Add some embedded assets to vislum_fs here...
    
    // Create asset loaders
    let loaders = AssetLoaders::new();
    // Add loaders here...
    
    // Create asset manager
    let mut manager = AssetManager::new_with_loaders(Arc::new(loaders));
    
    // Add the virtual filesystem entries to the manager
    manager.add_virtual_fs(VirtualFileSystemEntry::new(
        AssetPath::new_owned("project"),
        true,
        project_fs.clone(),
    ));
    
    manager.add_virtual_fs(VirtualFileSystemEntry::new(
        AssetPath::new_owned("vislum"),
        true,
        vislum_fs,
    ));
    
    // Load an asset
    let asset_path = AssetPath::new_owned("project/shaders/main.vert");
    let asset_id: AssetId = manager.load(asset_path);
    
    // Process events to handle loading completion
    manager.process_events();
    
    // Try to get the asset (it might still be loading)
    match manager.get_untyped(asset_id) {
        Ok(_asset) => println!("Asset loaded successfully!"),
        Err(e) => println!("Asset not ready yet: {:?}", e),
    }
}