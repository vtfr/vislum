
use vislum_system::Resource;

use crate::texture::{Texture, TextureDescriptor, TextureManager};
use crate::types::{RenderDevice, RenderQueue};
use crate::shader::{ShaderDescriptor, ShaderManager, ShaderModule};
use crate::resource::{Handle, IntoResourceId};
use crate::scene::{Scene, SceneCommand, SceneManager};
use crate::mesh::{MeshManager, RenderMesh, RenderMeshDescriptor};

/// A system for rendering the scene.
/// 
/// This system is responsible for rendering the scene to the screen.
#[derive(Resource)]
pub struct RenderSystem {
    device: RenderDevice,
    queue: RenderQueue,
    mesh_manager: MeshManager,
    scene_manager: SceneManager,
    shader_manager: ShaderManager,
    texture_manager: TextureManager,
}

impl RenderSystem {
    pub fn new(device: RenderDevice, queue: RenderQueue) -> Self {
        let mesh_manager = MeshManager::new(device.clone());
        let shader_manager = ShaderManager::new(device.clone());
        let texture_manager = TextureManager::new(device.clone(), queue.clone());

        Self { 
            device, 
            queue,
            mesh_manager,
            scene_manager: SceneManager::new(),
            shader_manager,
            texture_manager,
        }
    }

    /// Renders the scene.
    pub fn render(&mut self, passes: Vec<Box<dyn RenderPass>>, output: &wgpu::TextureView, output_format: wgpu::TextureFormat) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        let context = RenderPassContext {
            device: &self.device,
            queue: &self.queue,
            mesh_manager: &self.mesh_manager,
            scene_manager: &self.scene_manager,
            shader_manager: &self.shader_manager,
            texture_manager: &self.texture_manager,
            screen_view: output,
            screen_format: output_format,
        };

        for mut pass in passes {
            pass.render(context, &mut encoder);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Creates a new mesh.
    /// 
    /// Returns a [`Handle<RenderMesh>`] that can be used to reference the mesh. 
    /// 
    /// Once all references to the mesh are dropped, the mesh will be destroyed.
    pub fn create_mesh(&mut self, descriptor: RenderMeshDescriptor) -> Handle<RenderMesh> {
        self.mesh_manager.create(descriptor)
    }

    /// Gets a mesh by its id.
    pub fn get_mesh(&self, id: impl IntoResourceId<RenderMesh>) -> Option<&RenderMesh> {
        self.mesh_manager.get(id)
    }

    /// Creates a new scene.
    /// 
    /// Returns a [`Handle<Scene>`] that can be used to reference the scene.
    /// 
    /// Once all references to the scene are dropped, the scene will be destroyed.
    pub fn create_scene(&mut self) -> Handle<Scene> {
        self.scene_manager.create()
    }

    /// Creates a new scene and apply a list of commands to it.
    /// 
    /// Returns a [`Handle<Scene>`] that can be used to reference the scene.
    /// 
    /// Once all references to the scene are dropped, the scene will be destroyed.
    pub fn create_scene_with_commands(&mut self, initial_commands: impl IntoIterator<Item = SceneCommand>) -> Handle<Scene> {
        self.scene_manager.create_with_commands(initial_commands)
    }

    /// Apply a list of commands to a scene.
    pub fn apply_scene_commands(&mut self, scene_id: impl IntoResourceId<Scene>, commands: impl IntoIterator<Item = SceneCommand>) {
        self.scene_manager.apply(scene_id, commands).unwrap();
    }

    /// Creates a new shader.
    /// 
    /// Returns a [`Handle<ShaderModule>`] that can be used to reference the shader module.
    /// 
    /// Once all references to the shader module are dropped, the shader module will be destroyed.
    pub fn create_shader(&mut self, descriptor: ShaderDescriptor) -> Handle<ShaderModule> {
        self.shader_manager.create(descriptor)
    }

    /// Gets a shader by its id.
    pub fn get_shader(&self, id: impl IntoResourceId<ShaderModule>) -> Option<&ShaderModule> {
        self.shader_manager.get(id)
    }

    /// Creates a new texture.
    /// 
    /// Returns a [`Handle<Texture>`] that can be used to reference the texture.
    pub fn create_texture(&mut self, descriptor: TextureDescriptor) -> Handle<Texture> {
        self.texture_manager.create(descriptor)
    }

    /// Gets a texture by its id.
    pub fn get_texture(&self, id: impl IntoResourceId<Texture>) -> Option<&Texture> {
        self.texture_manager.get(id)
    }
}

/// A render target.
/// 
/// When rendering a scene, the last render target is used to render the scene to.
pub struct RenderTarget<'a> {
    pub view: &'a wgpu::TextureView,
}

/// The context of a render pass.
/// 
/// This context is passed to the render pass to allow it to access the mesh, scene, and shader managers.
#[derive(Copy, Clone)]
pub struct RenderPassContext<'a> {
    pub device: &'a RenderDevice,
    pub queue: &'a RenderQueue,
    pub mesh_manager: &'a MeshManager,
    pub scene_manager: &'a SceneManager,
    pub shader_manager: &'a ShaderManager,
    pub texture_manager: &'a TextureManager,
    pub screen_view: &'a wgpu::TextureView,
    pub screen_format: wgpu::TextureFormat,
}

pub trait RenderPass {
    fn render(
        &mut self,
        context: RenderPassContext,
        encoder: &mut wgpu::CommandEncoder,
    ); 
} 

/// A forward render pass on a given scene.
pub struct ForwardRenderPass {
    pub scene: Handle<Scene>,
    pub color_texture: Handle<Texture>,
}

impl RenderPass for ForwardRenderPass {
    fn render(
        &mut self,
        _context: RenderPassContext,
        _encoder: &mut wgpu::CommandEncoder,
    ) {
        // TODO: Implement forward rendering
    }
}

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

impl RenderPass for ScreenBlitPass {
    fn render(
        &mut self,
        context: RenderPassContext,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // Load the blit shader
        let shader_source = include_str!("blit.wgsl");
        let shader_module = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Get the input texture
        let input_texture = context.texture_manager.get(&self.input_texture).unwrap();

        // Create bind group layout
        let bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        // Create sampler
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blit Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group
        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_texture.default_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Screen Blit Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
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
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.screen_format,
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
                view: context.screen_view,
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
