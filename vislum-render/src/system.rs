use vislum_system::{System, Systems};

use crate::texture::{Texture, TextureDescriptor, TextureManager};
use crate::types::{RenderDevice, RenderQueue};
use crate::shader::{ShaderDescriptor, ShaderManager, ShaderModule};
use crate::resource::{Handle, IntoResourceId};
use crate::scene::{Scene, SceneCommand, SceneManager};
use crate::mesh::{MeshManager, RenderMesh, RenderMeshDescriptor};

/// A system for rendering the scene.
/// 
/// This system is responsible for rendering the scene to the screen.
#[derive(System)]
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
    pub fn render(&mut self, passes: Vec<Box<dyn RenderPass>>, output: &wgpu::TextureView) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        let context = RenderPassContext {
            mesh_manager: &self.mesh_manager,
            scene_manager: &self.scene_manager,
            shader_manager: &self.shader_manager,
            texture_manager: &self.texture_manager,
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
        self.scene_manager.apply(scene_id, commands);
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
    pub color_texture: Handle<()>,
}

impl RenderPass for ForwardRenderPass {
    fn render(
        &mut self,
        context: RenderPassContext,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // TODO
    }
}

/// A blit pass.
/// 
/// Blits a source render target to a destination render target.
pub struct ScreenBlitPass {
    pub input: Handle<Texture>,
}

impl RenderPass for ScreenBlitPass {
    fn render(
        &mut self,
        context: RenderPassContext,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: Some("Screen Blit Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: context.shader_manager.get(self.input.shader_module).unwrap().module(),
                entry_point: "main".to_string(),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState{
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: wgpu::FragmentState{
                module: context.shader_manager.get(self.input.shader_module).unwrap().module(),
                entry_point: "main".to_string(),
                targets: &[],
            },
            multiview: None,
            cache: None,
        })
    }
}
