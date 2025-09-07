use vislum_system::{System, Systems};

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
}

impl RenderSystem {
    pub fn new(device: RenderDevice, queue: RenderQueue) -> Self {
        let mesh_manager = MeshManager::new(device.clone());
        let shader_manager = ShaderManager::new(device.clone());

        Self { 
            device, 
            queue,
            mesh_manager,
            scene_manager: SceneManager::new(),
            shader_manager,
        }
    }

    /// Renders the scene.
    pub fn render(&mut self, passes: Vec<Box<dyn RenderPass>>) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        for mut pass in passes {
            pass.render(self, &mut encoder);
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
}

pub trait RenderPass {
    fn render(
        &mut self,
        render_system: &mut RenderSystem,
        encoder: &mut wgpu::CommandEncoder,
    ); 
} 

/// A forward render pass on a given scene.
pub struct ForwardRenderPass {
    pub scene: Handle<Scene>,
}

impl RenderPass for ForwardRenderPass {
    fn render(
        &mut self,
        render_system: &mut RenderSystem,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // TODO
    }
}