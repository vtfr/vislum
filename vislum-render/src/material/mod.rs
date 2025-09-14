use vislum_system::Resource;

use crate::{Handle, RenderDevice, ResourceStorage, ShaderModule};

/// A high-level description of a material.
/// 
/// This is used to create a material through the [`MaterialManager`].
// #[derive(Serialize, Deserialize)]
pub struct MaterialDescriptor {
    /// The name of the material.
    pub name: String,

    /// The vertex shader for the material.
    pub vertex_shader: Handle<ShaderModule>,

    /// The fragment shader for the material.
    pub fragment_shader: Handle<ShaderModule>,
}

pub struct Material {
    pub name: String,
    pub vertex_shader: Handle<ShaderModule>,
    pub fragment_shader: Handle<ShaderModule>,
}

pub enum CreateMaterialError {
    VertexShaderNotFound,
    FragmentShaderNotFound,
    InvalidVertexShader,
}

#[derive(Resource)]
pub struct MaterialManager {
    device: RenderDevice,
    storage: ResourceStorage<Material>,
}

// impl MaterialManager {
//     pub fn new(device: RenderDevice) -> Self {
//         Self { 
//             device, 
//             storage: Default::default(),
//         }
//     }

//     pub fn create(
//         &self, 
//         descriptor: MaterialDescriptor,
//         shader_module_manager: &ShaderModuleManager,
//     ) -> Result<Handle<Material>, CreateMaterialError> {
//         let vertex_shader = shader_module_manager.get(descriptor.vertex_shader);
//         let fragment_shader = shader_module_manager.get(descriptor.fragment_shader);

//         Ok(self.storage.insert(Material {
//             name: descriptor.name,
//             vertex_shader,
//             fragment_shader,
//         }))
//     }
// }

