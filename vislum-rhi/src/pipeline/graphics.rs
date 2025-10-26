use std::{ffi::CString, sync::Arc};

use ash::vk;

use crate::{
    AshHandle, VkHandle,
    descriptor::DescriptorSetLayout,
    device::device::Device,
    image::ImageFormat,
};

use super::ShaderModule;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexAttribute {
    pub location: u32,
    pub format: vk::Format,
    pub offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VertexBufferLayout {
    pub stride: u32,
    pub input_rate: vk::VertexInputRate,
    pub attributes: Vec<VertexAttribute>,
}

#[derive(Clone)]
pub struct GraphicsPipelineCreateInfo {
    pub vertex_shader: Arc<ShaderModule>,
    pub fragment_shader: Arc<ShaderModule>,
    pub vertex_buffer: Option<VertexBufferLayout>,
    pub descriptor_set_layouts: Vec<Arc<DescriptorSetLayout>>,
    pub topology: vk::PrimitiveTopology,
    pub color_formats: Vec<ImageFormat>,
    pub depth_format: Option<ImageFormat>,
}

pub struct GraphicsPipeline {
    device: Arc<Device>,
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

impl VkHandle for GraphicsPipeline {
    type Handle = vk::Pipeline;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.pipeline
    }
}

impl GraphicsPipeline {
    pub fn new(device: Arc<Device>, create_info: GraphicsPipelineCreateInfo) -> Arc<Self> {
        let entry_point = CString::new("main").unwrap();

        // Shader stages
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(create_info.vertex_shader.vk_handle())
                .name(&entry_point),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(create_info.fragment_shader.vk_handle())
                .name(&entry_point),
        ];

        // Vertex input
        let binding_descriptions;
        let attribute_descriptions;
        
        let vertex_input_state = if let Some(vertex_buffer) = &create_info.vertex_buffer {
            binding_descriptions = vec![
                vk::VertexInputBindingDescription::default()
                    .binding(0)
                    .stride(vertex_buffer.stride)
                    .input_rate(vertex_buffer.input_rate),
            ];

            attribute_descriptions = vertex_buffer
                .attributes
                .iter()
                .map(|attr| {
                    vk::VertexInputAttributeDescription::default()
                        .location(attr.location)
                        .binding(0)
                        .format(attr.format)
                        .offset(attr.offset)
                })
                .collect::<Vec<_>>();

            vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(&binding_descriptions)
                .vertex_attribute_descriptions(&attribute_descriptions)
        } else {
            vk::PipelineVertexInputStateCreateInfo::default()
        };

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(create_info.topology)
            .primitive_restart_enable(false);

        // Viewport state (dynamic)
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        // Rasterization
        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0);

        // Multisample
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // Depth stencil
        let depth_stencil_state = if create_info.depth_format.is_some() {
            vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS)
        } else {
            vk::PipelineDepthStencilStateCreateInfo::default()
        };

        // Color blend
        let color_blend_attachments: Vec<_> = (0..create_info.color_formats.len())
            .map(|_| {
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(false)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            })
            .collect();

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(&color_blend_attachments);

        // Dynamic state
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        // Pipeline layout
        let set_layouts: Vec<_> = create_info
            .descriptor_set_layouts
            .iter()
            .map(|layout| layout.vk_handle())
            .collect();

        let layout_create_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);

        let layout = unsafe {
            device
                .ash_handle()
                .create_pipeline_layout(&layout_create_info, None)
                .expect("Failed to create pipeline layout")
        };

        // Rendering info (dynamic rendering)
        let color_formats: Vec<_> = create_info
            .color_formats
            .iter()
            .map(|f| f.to_vk())
            .collect();

        let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_formats);

        if let Some(depth_format) = create_info.depth_format {
            rendering_info = rendering_info.depth_attachment_format(depth_format.to_vk());
        }

        // Create pipeline
        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(layout)
            .push_next(&mut rendering_info);

        let pipeline = unsafe {
            device
                .ash_handle()
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .expect("Failed to create graphics pipeline")[0]
        };

        Arc::new(Self {
            device,
            pipeline,
            layout,
        })
    }

    #[inline]
    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ash_handle()
                .destroy_pipeline(self.pipeline, None);
            self.device
                .ash_handle()
                .destroy_pipeline_layout(self.layout, None);
        }
    }
}

