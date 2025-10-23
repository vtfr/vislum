use vislum_op::system::NodeGraphSystem;
use vislum_render::{MeshManager, RenderPassCollector, SceneManager, ShaderManager, TextureManager};
use vislum_render::types::{RenderDevice, RenderQueue};
use vislum_system::{ResMut, Res, Resource, Resources};

/// A runner runs the runtime.
pub type Runner = Box<dyn Fn(&mut Engine)>;

/// The runtime for the vislum engine.
pub struct Engine {
    pub runner: Runner,
}

impl Engine {
    /// Runs a single frame.
    pub fn run_frame(&mut self) {}

    /// Runs the engine.
    pub fn run(&mut self) {
        (self.runner)(&mut self.resources);
    }
}
