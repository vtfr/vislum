use vislum_system::System;

use crate::{mesh::RenderMesh, resource::ResourceId, shader_module::{ShaderModule, ShaderModuleManager}, types::RenderDevice};

#[derive(System)]
pub struct RenderSystem {
    device: RenderDevice,
    shader_module_system: ShaderModuleManager,
}
