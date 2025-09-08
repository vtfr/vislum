use encase::ShaderType;
use vislum_math::{Vector3, Vector4};
use vislum_system::Resource;

use crate::cache::bind_group::BindGroup;
use crate::cache::storage::{Handle, IntoResourceId, ResourceId, RenderResourceStorage};
use crate::cache::storage::Uniform;
use crate::cache::types::RenderDevice;

#[derive(ShaderType)]
pub struct MaterialUniformData {
    color: [f32; 3],
}

// /// A color or texture.
// pub enum ColorOrTexture {
//     Color(Vector4),
//     Texture(Handle<()>),
// }

// /// A base material.
// pub struct PbrProperties {
//     pub albedo: ColorOrTexture,
//     pub roughness: ColorOrTexture,
//     pub normal: ColorOrTexture,
// }

pub struct RenderMaterial {
    pub color: Vector3,
    // /// The base material.
    // ///
    // /// All materials are PBR materials.
    // pub pbr: PbrProperties,

    // The uniform data for the material.
    // pub uniform: Uniform<MaterialUniformData>,

    // The bind group for the material.
    // 
    // Contains both the uniform data and the texture data, along with
    // the samplers.
    // pub bind_group: BindGroup,
}

pub struct MaterialDescriptor {
    pub color: Vector3,
}

#[derive(Resource)]
pub struct MaterialManager {
    device: RenderDevice,
    materials: RenderResourceStorage<RenderMaterial>,
}

impl MaterialManager {
    pub fn new(device: RenderDevice) -> Self {
        Self { device, materials: RenderResourceStorage::new() }
    }

    /// Creates a new material.
    pub fn create(&mut self, descriptor: MaterialDescriptor) -> Handle<RenderMaterial> {
        let material = RenderMaterial {
            color: descriptor.color,
        };

        self.materials.insert(material)
    }

    /// Gets a material by its ID.
    pub fn get(&self, id: impl IntoResourceId<RenderMaterial>) -> Option<&RenderMaterial> {
        let id = id.into_resource_id();
        self.materials.get(id)
    }
}
