use std::sync::Arc;

use vulkano::{device::{Device, Queue}, memory::allocator::MemoryAllocator};

use crate::{resource::ResourceManager, scene::{SceneCommand, RenderCommand}};

/// The renderer. Nothing else needs to be said.
pub struct Renderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    allocator: Arc<dyn MemoryAllocator>,
    resource_manager: ResourceManager,
}

impl Renderer {
    pub fn render(
        &self,
        scene: RenderCommand,
    ) {
}