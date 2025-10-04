use vulkano::{device::Device, render_pass::RenderPass};

use crate::RenderContext;

pub struct PipelineManager {
    context: RenderContext,
}

impl PipelineManager {
    pub fn new(context: RenderContext) -> Self {
        Self { context }
    }

    pub fn create_pipeline(&self, ) -> Pipeline {
        vulkano::pipeline::GraphicsPipeline::new(
            self.context.device().clone(),
            None,
            vulkano::pipeline::graphics::GraphicsPipelineCreateInfo{
                flags: todo!(),
                stages: todo!(),
                vertex_input_state: todo!(),
                input_assembly_state: todo!(),
                tessellation_state: todo!(),
                viewport_state: todo!(),
                rasterization_state: todo!(),
                multisample_state: todo!(),
                depth_stencil_state: todo!(),
                color_blend_state: todo!(),
                dynamic_state: todo!(),
                layout: todo!(),
                subpass: todo!(),
                base_pipeline: todo!(),
                discard_rectangle_state: todo!(),
                fragment_shading_rate_state: todo!(),
                _ne: todo!(),
            });

        self.context.device
    }
}