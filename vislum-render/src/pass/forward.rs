use std::borrow::Cow;

use vislum_system::{ResMut, Resource, Resources};

use crate::{FlattenedScene, Handle, IntoResourceId, MeshManager, RenderDevice, RenderPass, Scene, SceneManager, ScreenRenderTarget, Texture, TextureManager, Vertex};

pub struct ForwardRenderPass {
    pub scene: Handle<Scene>,
    pub color: Handle<Texture>,
}

impl ForwardRenderPass {
    pub fn new(scene: Handle<Scene>, color: Handle<Texture>) -> Self {
        Self { scene, color }
    }
}

/// The shared data for the forward render pass.
#[derive(Resource)]
pub struct ForwardRenderPassShared {
    pub shader_module: wgpu::ShaderModule,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
}

impl ForwardRenderPassShared {
    pub fn new(device: &RenderDevice, format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Forward Render Pass Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("forward.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Forward Render Pass Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Forward Render Pass Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[
                    Vertex::layout(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: None,
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                conservative: false,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: Default::default(),
            cache: Default::default(),
        });

        Self {
            shader_module,
            pipeline_layout,
            pipeline,
        }
    }

    pub fn get<'res, 'device>(resources: &'res Resources, device: &'device RenderDevice, format: wgpu::TextureFormat) -> ResMut<'res, Self> {
        resources.get_mut_or_insert_with(|| Self::new(device, format))
    }
}

impl RenderPass for ForwardRenderPass {
    fn render(
        &self, 
        resources: &Resources, 
        _screen_render_target: &ScreenRenderTarget, 
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let device = resources.get::<RenderDevice>();
        let scene_manager = resources.get::<SceneManager>();
        let mesh_manager = resources.get::<MeshManager>();
        let texture_manager = resources.get::<TextureManager>();

        // Retrieve the output texture.
        let Some(color) = texture_manager.get(self.color.into_resource_id()) else { return };

        // Flatten the scene.
        let mut flattened_scene = FlattenedScene::new();
        scene_manager.visit(self.scene.into_resource_id(), &mut flattened_scene);
        
        // Get the shared data for the forward render pass.
        let shared = ForwardRenderPassShared::get(resources, &*device, color.format().into());

        // Begin the render pass.
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Forward Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color.default_view(),
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

        render_pass.set_pipeline(&shared.pipeline);

        // Render the scene.
        for object in flattened_scene.objects {
            let Some(mesh) = mesh_manager.get(object.mesh.into_resource_id()) else { continue };

            let vertex_buffer_slice = mesh.vertex_buffer().slice(..);
            let index_buffer_slice = mesh.index_buffer().slice(..);
            let indices_len = mesh.indices().len() as u32;

            render_pass.set_vertex_buffer(0, vertex_buffer_slice);
            render_pass.set_index_buffer(index_buffer_slice, wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..indices_len, 0, 0..1);
        }
    }
}