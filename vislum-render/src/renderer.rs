use vislum_asset::asset::AssetEvent;

use crate::{
    Handle, Material, Mesh, MeshDescriptor, MeshManager, RenderDevice, ShaderManager, material::MaterialManager
};

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
}
