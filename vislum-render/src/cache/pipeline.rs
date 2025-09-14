use crate::{Handle, ShaderModule};

/// A high-level description of a render pipeline.
/// 
/// This is used to create a render pipeline.
pub struct RenderPipelineDescriptor {
    pub vertex_shader: Handle<ShaderModule>,
    pub fragment_shader: Handle<ShaderModule>,
}
