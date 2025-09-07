// use std::collections::HashMap;

// use vislum_system::System;

// use crate::resource::{ResourceId, ResourceStorage};
// use crate::shader::{ShaderModule, ShaderManager};
// use crate::types::RenderDevice;

// crate::wrap_wgpu_with_atomic_id! {
//     /// A render pipeline.
//     pub struct RenderPipeline(RenderPipelineId): wgpu::RenderPipeline;
// }

// /// A key for a render pipeline.
// ///
// /// This key is used to create a render pipeline.
// #[derive(Clone, Copy, Eq, PartialEq, Hash)]
// pub struct RenderPipelineKey {
//     pub vertex_shader: ResourceId<ShaderModule>,
//     pub fragment_shader: ResourceId<ShaderModule>,
// }

// /// An error that can occur when creating a render pipeline.
// #[derive(Debug, thiserror::Error)]
// pub enum RenderPipelineError {
//     #[error("Shader module not found: {0:?}")]
//     ShaderModuleNotFound(ResourceId<ShaderModule>),
// }

// #[derive(System)]
// pub struct RenderPipelineCache {
//     device: RenderDevice,
//     pipelines: HashMap<RenderPipelineKey, RenderPipeline>,
// }

// impl RenderPipelineCache {
//     pub fn new(device: RenderDevice) -> Self {
//         Self {
//             device,
//             pipelines: Default::default(),
//         }
//     }

//     pub fn get(
//         &mut self,
//         key: RenderPipelineKey,
//         shader_module_manager: &mut ShaderModuleManager,
//     ) -> Result<RenderPipeline, RenderPipelineError> {
//         // Check if the pipeline is already cached.
//         if let Some(pipeline) = self.pipelines.get(&key) {
//             return Ok(pipeline.clone());
//         }

//         let vertex_shader = shader_module_manager
//             .get(key.vertex_shader)
//             .ok_or(RenderPipelineError::ShaderModuleNotFound(key.vertex_shader))?;

//         let fragment_shader = shader_module_manager
//             .get(key.fragment_shader)
//             .ok_or(RenderPipelineError::ShaderModuleNotFound(key.fragment_shader))?;

//         let pipeline = self
//             .device
//             .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//                 label: None,
//                 layout: None,
//                 vertex: wgpu::VertexState {
//                     module: vertex_shader.inner(),
//                     entry_point: Some(vertex_shader.entry_point()),
//                     compilation_options: Default::default(),
//                     buffers: &[],
//                 },
//                 primitive: wgpu::PrimitiveState {
//                     topology: wgpu::PrimitiveTopology::TriangleList,
//                     strip_index_format: None,
//                     front_face: wgpu::FrontFace::Ccw,
//                     cull_mode: None,
//                     unclipped_depth: false,
//                     polygon_mode: wgpu::PolygonMode::Fill,
//                     conservative: false,
//                 },
//                 depth_stencil: Some(wgpu::DepthStencilState {
//                     format: wgpu::TextureFormat::Depth24PlusStencil8,
//                     depth_write_enabled: true,
//                     depth_compare: wgpu::CompareFunction::Less,
//                     stencil: wgpu::StencilState::default(),
//                     bias: wgpu::DepthBiasState::default(),
//                 }),
//                 multisample: wgpu::MultisampleState::default(),
//                 fragment: Some(wgpu::FragmentState {
//                     module: fragment_shader.inner(),
//                     entry_point: Some(fragment_shader.entry_point()),
//                     targets: &[Some(wgpu::ColorTargetState {
//                         format: wgpu::TextureFormat::Rgba8Unorm,
//                         blend: Default::default(),
//                         write_mask: wgpu::ColorWrites::ALL,
//                     })],
//                     compilation_options: Default::default(),
//                 }),
//                 multiview: Default::default(),
//                 cache: Default::default(),
//             });

//         let pipeline = RenderPipeline::new(pipeline);
//         self.pipelines.insert(key, pipeline.clone());

//         Ok(pipeline)
//     }
// }

// // pub struct RenderPipelineLayoutCache {
// //     device: Device,
// //     // pipeline: wgpu::RenderPipeline,
// // }

// // pub struct RenderPipelineLayoutDescriptor<'a> {
// //     pub bind_group_layouts: Cow<'a, [wgpu::BindGroupLayout]>,
// // }

// // impl RenderPipelineLayoutCache {
// //     pub fn new(device: Device) -> Self {
// //         Self { device }
// //     }

// //     pub fn create(&mut self, descriptor: RenderPipelineLayoutDescriptor) -> Handle<RenderPipelineLayout> {
// //         let layout = self.device.create_pipeline_layout(wgpu::PipelineLayoutDescriptor {
// //             label: Some("Render Pipeline Layout"),
// //             bindings: descriptor.bindings,
// //             bind_group_layouts: todo!(),
// //             push_constant_ranges: todo!(),
// //         })
// //     }
// // }

// // pub struct RenderPipelineDescriptor {}

// // pub struct RenderPipelineManager {
// //     device: Device,
// //     pipelines: ResourceStorage<RenderPipeline>,
// // }

// // impl RenderPipelineManager {
// //     pub fn new(device: Device) -> Self {
// //         Self { device, pipelines: ResourceStorage::new() }
// //     }

// //     pub fn create(&mut self, descriptor: RenderPipelineDescriptor) -> Handle<RenderPipeline> {
// //         self.device.create_render_pipeline(wgpu::RenderPipelineDescriptor {
// //             label: Some("Render Pipeline"),
// //             layout: None,
// //             vertex: wgpu::VertexState {
// //                 module: &descriptor.vertex_shader,
// //                 entry_point: todo!(),
// //                 compilation_options: todo!(),
// //                 buffers: todo!(),
// //             },
// //             primitive: wgpu::PrimitiveState{
// //                 topology: wgpu::PrimitiveTopology::TriangleList,
// //                 strip_index_format: None,
// //                 front_face: wgpu::FrontFace::Ccw,
// //                 cull_mode: None,
// //                 unclipped_depth: false,
// //                 polygon_mode: wgpu::PolygonMode::Fill,
// //                 conservative: false,
// //             },
// //             depth_stencil: Default::default(),
// //             multisample: Default::default(),
// //             fragment: Some(wgpu::FragmentState {
// //                 module: &descriptor.fragment_shader,
// //                 entry_point: "main",
// //                 targets: &[wgpu::FragmentTargetState {
// //                     format: wgpu::TextureFormat::Rgba8Unorm,
// //                     blend: Default::default(),
// //                 }],
// //                 compilation_options: Default::default(),
// //             },
// //             multiview: Default::default(),
// //             cache: Default::default(),
// //         });

// //         self.pipelines.insert(RenderPipeline {
// //             pipeline: self.device.create_render_pipeline(&descriptor.pipeline_descriptor),
// //         })
// //     }
// // }
