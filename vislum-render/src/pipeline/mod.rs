use std::{collections::HashSet, sync::Arc};

use vulkano::{device::Device};

use crate::pipeline::shader::ShaderCache;

pub mod shader;
pub mod graphics;
pub mod layout;

pub struct PipelineManager {
    device: Arc<Device>,
    shader_cache: Arc<ShaderCache>,
}

impl PipelineManager {
    pub fn new(shader_cache: Arc<ShaderCache>) -> Self {
        Self { shader_cache }
    }

    pub fn create_graphics_pipeline(
        &self,
        device: &Arc<Device>,
        descriptor: GraphicsPipelineDescriptor,
    ) -> Result<GraphicsPipeline, ()> {
        let create_info = vulkano::pipeline::graphics::GraphicsPipelineCreateInfo {
            flags: vulkano::pipeline::PipelineCreateFlags::empty(),
            stages: vec![],
            vertex_input_state: None,
            input_assembly_state: None,
            tessellation_state: None,
            viewport_state: None,
            rasterization_state: None,
            dynamic_state: [
                vulkano::pipeline::DynamicState::Viewport,
                vulkano::pipeline::DynamicState::Scissor,
                vulkano::pipeline::DynamicState::DepthBounds,
                vulkano::pipeline::DynamicState::StencilCompareMask,
                vulkano::pipeline::DynamicState::StencilWriteMask,
                vulkano::pipeline::DynamicState::StencilReference,
                vulkano::pipeline::DynamicState::CullMode,
                vulkano::pipeline::DynamicState::FrontFace,
                vulkano::pipeline::DynamicState::PrimitiveTopology,
                vulkano::pipeline::DynamicState::ViewportWithCount,
                vulkano::pipeline::DynamicState::ScissorWithCount,
                vulkano::pipeline::DynamicState::VertexInputBindingStride,
                vulkano::pipeline::DynamicState::DepthTestEnable,
                vulkano::pipeline::DynamicState::DepthWriteEnable,
                vulkano::pipeline::DynamicState::DepthCompareOp,
                vulkano::pipeline::DynamicState::DepthBoundsTestEnable,
                vulkano::pipeline::DynamicState::StencilTestEnable,
                vulkano::pipeline::DynamicState::StencilOp,
                vulkano::pipeline::DynamicState::RasterizerDiscardEnable,
            ].into(),
            layout: todo!(),
            ..Default::default()
        };

        vulkano::pipeline::GraphicsPipeline::new(device.clone(), create_info)
}

static_assertions::assert_impl_all!(PipelineManager: Send, Sync);