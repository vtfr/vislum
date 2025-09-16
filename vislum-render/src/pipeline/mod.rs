use std::{collections::HashMap, sync::Arc};

use crate::device::RenderDevice;

pub struct PipelineDescriptor {}

pub struct RenderPipeline {

}

/// A cache of pipelines.
/// 
pub struct PipelineCache {
    device: RenderDevice,
    pipelines: HashMap<PipelineDescriptor, Arc<RenderPipeline>>,
}

impl PipelineCache {
    pub fn create_render_pipeline(&mut self, descriptor: &PipelineDescriptor) -> Arc<RenderPipeline> {
        if let Some(pipeline) = self.pipelines.get(descriptor) {
            return pipeline.clone();
        }

        fn whatever<T>() -> T {
            todo!()
        }

        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: None,
            layout: None,
            vertex: wgpu::VertexState{
                module: whatever(),
                entry_point: whatever(),
                compilation_options: Default::default(),
                buffers: whatever(),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState{
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState{
                module: whatever(),
                entry_point: whatever(),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState{
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        let pipeline = Arc::new(pipeline);

        todo!()
    }
}