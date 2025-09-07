use vislum_system::{Resource, Resources};

use crate::{RenderDevice, ScreenRenderTarget, TextureManager};
use crate::resource::texture::Texture;
use crate::cache::storage::Handle;
use crate::pass::RenderPass;

/// A blit pass.
/// 
/// Blits a source render target to a destination render target.
pub struct ScreenBlitPass {
    pub input_texture: Handle<Texture>,
}

impl ScreenBlitPass {
    pub fn new(input_texture: Handle<Texture>) -> Self {
        Self {
            input_texture,
        }
    }
}

#[derive(Resource)]
pub struct ScreenBlitPassShared {
    shader_module: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    sampler: wgpu::Sampler,
}

impl ScreenBlitPassShared {
    pub fn new(device: &RenderDevice) -> Self {
        let shader_source = include_str!("blit.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blit Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blit Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            shader_module,
            bind_group_layout,
            pipeline_layout,
            sampler,
        }
    }
}

impl RenderPass for ScreenBlitPass {
    fn render(
        &self,
        resources: &Resources,
        screen_render_target: &ScreenRenderTarget,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let device = resources.get::<RenderDevice>();
        let texture_manager = resources.get::<TextureManager>();
        let shared = resources.get_or_insert_with(|| ScreenBlitPassShared::new(&*device));

        let input_texture = texture_manager.get(&self.input_texture).unwrap();

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit Bind Group"),
            layout: &shared.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_texture.default_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shared.sampler),
                },
            ],
        });

        // Create render pipeline.
        //
        // This is really fucking inefficient, but I'm lazy so fuck it ;)
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Screen Blit Pipeline"),
            layout: Some(&shared.pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shared.shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shared.shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: screen_render_target.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        // Begin render pass and draw
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Blit Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &screen_render_target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}