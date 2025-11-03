use std::sync::Arc;

use vislum_render_rhi::{
    device::Device,
    memory::MemoryAllocator,
    queue::Queue,
    image::Image,
};

use crate::{graph::{FrameGraph, pass::FrameGraphSubmitInfo, FrameNode}, resource::{ResourceManager, pool::ResourceId, texture::{Texture, TextureCreateInfo, TextureUploadTask}}};

pub struct RenderContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    allocator: Arc<MemoryAllocator>,
    resource_manager: ResourceManager,
    frame_graph: FrameGraph,
}

impl RenderContext {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> Self {
        let allocator = MemoryAllocator::new(device.clone());
        let resource_manager = ResourceManager::new(device.clone(), allocator.clone());
        let frame_graph = FrameGraph::new(device.clone(), queue.clone(), allocator.clone());

        Self {
            device,
            queue,
            allocator,
            resource_manager,
            frame_graph,
        }
    }

    /// Adds a new pass to the frame graph.
    pub fn add_pass<F>(&mut self, node: F)
    where
        F: FrameNode + 'static,
    {
        self.frame_graph.add_pass(node);
    }

    pub fn execute_and_submit(&mut self, submit_info: FrameGraphSubmitInfo) {
        self.frame_graph.execute(&self.resource_manager, submit_info);
    }

    /// Creates a texture with data and returns the resource id and upload task.
    pub fn create_texture_with_data(
        &mut self,
        info: TextureCreateInfo,
        data: &[u8],
    ) -> (ResourceId<Arc<Texture>>, TextureUploadTask) {
        self.resource_manager.create_texture_with_data(info, data)
    }


    pub fn get_texture_image(&self, id: ResourceId<Arc<Texture>>) -> Option<Arc<Image>> {
        self.resource_manager.resolve_texture_image(id)
    }
}
