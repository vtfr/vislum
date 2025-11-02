use std::sync::Arc;

use vulkano::{
    buffer::{BufferContents, Subbuffer},
    device::{Device, Queue},
    memory::allocator::{MemoryAllocator, StandardMemoryAllocator},
    swapchain::{Surface, Swapchain},
};

use crate::buffer::upload::Uploader;

pub struct RenderContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface>,
    swapchain: Arc<Swapchain>,
    allocator: Arc<dyn MemoryAllocator>,
    uploader: Uploader,
}

impl RenderContext {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        surface: Arc<Surface>,
        swapchain: Arc<Swapchain>,
    ) -> Arc<Self> {
        let allocator = Arc::new(StandardMemoryAllocator::new(device.clone(), Default::default()));
        let uploader = Uploader::new(device.clone(), allocator.clone());

        Arc::new(Self {
            device,
            queue,
            surface,
            swapchain,
            allocator,
            uploader,
        })
    }

    /// Uploads data to a buffer.
    pub fn upload_buffer(&self, buffer: Subbuffer<[u8]>, data: &[u8]) {
        // self.uploader.upload_buffer(buffer, data);
    }
}
