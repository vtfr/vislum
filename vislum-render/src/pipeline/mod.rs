use std::sync::Arc;

use serde::{Deserialize, Serialize};
use vislum_asset::path::AssetPath;

use crate::{RenderDevice, ShaderModule, Vertex};

pub struct PipelineManager {
    device: RenderDevice,
}

/// The vertex buffer layout for the pipeline.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum RenderPipelineVertexBufferLayout {
    /// The default vertex buffer layout used for rendering meshes.
    #[default]
    Mesh,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum PrimitiveTopology {
    #[default]
    TriangleList,
}

impl Into<wgpu::PrimitiveTopology> for PrimitiveTopology {
    fn into(self) -> wgpu::PrimitiveTopology {
        match self {
            PrimitiveTopology::TriangleList => wgpu::PrimitiveTopology::TriangleList,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum FrontFace {
    /// The front face is counter-clockwise.
    #[default]
    Ccw,

    /// The front face is clockwise.
    Cw,
}

impl Into<wgpu::FrontFace> for FrontFace {
    fn into(self) -> wgpu::FrontFace {
        match self {
            FrontFace::Ccw => wgpu::FrontFace::Ccw,
            FrontFace::Cw => wgpu::FrontFace::Cw,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PrimitiveState {
    /// The topology of the primitive.
    pub topology: PrimitiveTopology,

    /// The front face of the primitive.
    pub front_face: FrontFace,
}

pub struct RenderPipelineVertexState {
    /// The vertex shader asset path used to load the vertex shader.
    pub shader_module: Arc<ShaderModule>,

    /// The vertex buffer layout for the pipeline.
    pub render_pipeline_vertex_buffer_layout: RenderPipelineVertexBufferLayout,
}

pub struct RenderPipelineDescriptor {
    /// The vertex state for the pipeline.
    pub vertex_state: RenderPipelineVertexState,
}

impl PipelineManager {
    pub fn new(device: RenderDevice) -> Self {
        Self { device }
    }

    pub fn create(
        &mut self, 
        descriptor: RenderPipelineDescriptor,
    ) -> ! {
        fn whatever<T>() -> T {
            unsafe { std::mem::MaybeUninit::uninit().assume_init() }
        }

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                whatever(),
                whatever(),
            ],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: None,
            layout: None,
            vertex: wgpu::VertexState { 
                module: whatever(),
                entry_point: Some("main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 16,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: whatever(),
                entry_point: Some("main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        todo!()
    }
}