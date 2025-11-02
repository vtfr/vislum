use std::sync::Arc;

use vislum_render_rhi::{
    device::Device,
    queue::Queue,
    memory::MemoryAllocator,
};

use crate::{resource::ResourceManager, scene::{SceneCommand, RenderCommand}};

/// The renderer. Nothing else needs to be said.
pub struct Renderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    allocator: Arc<MemoryAllocator>,
    resource_manager: ResourceManager,
}

impl Renderer {
    pub fn render(
        &self,
        scene: RenderCommand,
    ) {
}