use std::borrow::Cow;

use crate::{resource::{Handle, RenderResourceStorage}, shader_module::ShaderModule, types::RenderDevice, wrap_wgpu_with_atomic_id};

wrap_wgpu_with_atomic_id! {
    /// A render pipeline.
    pub struct RenderPipeline(RenderPipelineId): wgpu::RenderPipeline;
}

pub struct RenderPipelineCache {
    device: RenderDevice,
}

pub struct RenderPipelineDescriptor<'a> {
    pub vertex_shader: &'a ShaderModule,
    pub fragment_shader: &'a ShaderModule,
}

impl RenderPipelineCache {
    pub fn new(device: RenderDevice) -> Self {
        Self { device }
    }

    pub fn create<'a>(
        &'a mut self, 
        descriptor: RenderPipelineDescriptor<'a>,
    ) -> RenderPipeline {
        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: None,
            layout: None,
            vertex: wgpu::VertexState {
                module: descriptor.vertex_shader.get_inner(),
                entry_point: Some(descriptor.vertex_shader.get_entry_point()),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState{
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState{
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState{
                module: descriptor.fragment_shader.get_inner(),
                entry_point: Some(descriptor.fragment_shader.get_entry_point()),
                targets: &[
                    Some(wgpu::ColorTargetState{
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: Default::default(),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: Default::default(),
            }),
            multiview: Default::default(),
            cache: Default::default(),
        });

        RenderPipeline::new(pipeline)
    }
}

// pub struct RenderPipelineLayoutCache {
//     device: Device,
//     // pipeline: wgpu::RenderPipeline,
// }

// pub struct RenderPipelineLayoutDescriptor<'a> {
//     pub bind_group_layouts: Cow<'a, [wgpu::BindGroupLayout]>,
// }

// impl RenderPipelineLayoutCache {
//     pub fn new(device: Device) -> Self {
//         Self { device }
//     }

//     pub fn create(&mut self, descriptor: RenderPipelineLayoutDescriptor) -> Handle<RenderPipelineLayout> {
//         let layout = self.device.create_pipeline_layout(wgpu::PipelineLayoutDescriptor {
//             label: Some("Render Pipeline Layout"),
//             bindings: descriptor.bindings,
//             bind_group_layouts: todo!(),
//             push_constant_ranges: todo!(),
//         })
//     }
// }


// pub struct RenderPipelineDescriptor {}


// pub struct RenderPipelineManager {
//     device: Device,
//     pipelines: ResourceStorage<RenderPipeline>,
// }

// impl RenderPipelineManager {
//     pub fn new(device: Device) -> Self {
//         Self { device, pipelines: ResourceStorage::new() }
//     }

//     pub fn create(&mut self, descriptor: RenderPipelineDescriptor) -> Handle<RenderPipeline> {
//         self.device.create_render_pipeline(wgpu::RenderPipelineDescriptor {
//             label: Some("Render Pipeline"),
//             layout: None,
//             vertex: wgpu::VertexState {
//                 module: &descriptor.vertex_shader,
//                 entry_point: todo!(),
//                 compilation_options: todo!(),
//                 buffers: todo!(),
//             },
//             primitive: wgpu::PrimitiveState{
//                 topology: wgpu::PrimitiveTopology::TriangleList,
//                 strip_index_format: None,
//                 front_face: wgpu::FrontFace::Ccw,
//                 cull_mode: None,
//                 unclipped_depth: false,
//                 polygon_mode: wgpu::PolygonMode::Fill,
//                 conservative: false,
//             },
//             depth_stencil: Default::default(),
//             multisample: Default::default(),
//             fragment: Some(wgpu::FragmentState {
//                 module: &descriptor.fragment_shader,
//                 entry_point: "main",
//                 targets: &[wgpu::FragmentTargetState {
//                     format: wgpu::TextureFormat::Rgba8Unorm,
//                     blend: Default::default(),
//                 }],
//                 compilation_options: Default::default(),
//             },
//             multiview: Default::default(),
//             cache: Default::default(),
//         });

//         self.pipelines.insert(RenderPipeline {
//             pipeline: self.device.create_render_pipeline(&descriptor.pipeline_descriptor),
//         })
//     }
// }