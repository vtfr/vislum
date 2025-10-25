use std::sync::Arc;

use serde::{Deserialize, Serialize};
use vulkano::shader::ShaderStage;

use crate::resources::bindless::BindlessResourceType;

pub struct GraphicsPipeline {
    pipeline: Arc<()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderDescription {
    name: String,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexState {
    vertex_shader: ShaderDescription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentState {
    fragment_shader: ShaderDescription,
}

/// The type of a pipeline binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineBindingType {
    /// A bindless resource.
    BindlessResource(BindlessResourceType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsPipelineDescriptor {}
