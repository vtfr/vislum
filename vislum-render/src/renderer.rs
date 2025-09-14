use vislum_asset::InternalAssetEvent;

use crate::{MeshManager, RenderDevice, ShaderManager, material::MaterialManager};

pub struct Renderer {
    device: RenderDevice,
    mesh_manager: MeshManager,
    shader_module_manager: ShaderManager,
    // material_manager: MaterialManager,
}

impl Renderer {
    pub fn new(device: RenderDevice) -> Self {
        let mesh_manager = MeshManager::new(device.clone());
        let shader_manager = ShaderManager::new(device.clone());

        Self { 
            device,
            mesh_manager,
            shader_module_manager: shader_manager,
        }
    }

    /// Listens to asset events and updates the renderer resources.
    pub fn on_asset_events(&self, events: &[InternalAssetEvent]) {
        for _event in events {
            // TODO: Implement this.
        }
    }

    /// Creates a new mesh.
    pub fn create_mesh(&self, mesh: Mesh) -> Handle<Mesh> {
        self.mesh_manager.create(mesh)
    }

    /// Creates a new material.
    pub fn create_material(&self, material: Material) -> Handle<Material> {
        self.material_manager.create(material)
    }
}