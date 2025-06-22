use std::collections::HashMap;

use slotmap::SlotMap;
use wgpu::MultisampleState;

use crate::{Device, MaterialArchetype};

/// Manages all render pipelines for materials.
pub struct MaterialRenderPipelineManager {
    cached: HashMap<MaterialArchetype, ()>,
}

#[allow(unused_must_use)]
#[allow(dead_code)]
fn x(device: Device) {
    // Each *material archetype* will define their own bind group layout.
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    // Mostly the same.
    //
    // We only care for bind-groups when dealing with custom
    // materials or some shit
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    // Only a few things will be customizable.
    //
    // These will be used as keys in a hashmap.
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState {
            // Default for mesh
            module: todo!(),
            entry_point: todo!(),
            compilation_options: todo!(),
            buffers: todo!(),
        },
        primitive: wgpu::PrimitiveState {
            // Customizable
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            // Customizable
            front_face: wgpu::FrontFace::Ccw,
            // Customizable
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            // Customizable
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            ..Default::default()
        },
        fragment: Some(wgpu::FragmentState {
            // Customizable
            module: todo!(),
            entry_point: todo!(),
            compilation_options: todo!(),
            targets: todo!(),
        }),
        multiview: None,
        cache: None,
    });
}
