use std::sync::{Arc, Mutex};

use ash::vk;
use vislum_render_rhi::{
    buffer::{Buffer, BufferCreateInfo},
    command::AutoCommandBuffer,
    device::Device,
    image::Image,
    memory::MemoryAllocator,
};

/// A task to upload data to the GPU.
pub enum UploadTask {
    /// Uploads data to a buffer.
    Buffer {
        staging: Arc<Buffer>,
        destination: Arc<Buffer>,
    },

    /// Uploads data to an image.
    Image {
        staging: Arc<Buffer>,
        image: Arc<Image>,
        copy_region: vk::BufferImageCopy,
    },
}

/// A manager for uploading data to the GPU.
pub struct Uploader {
    device: Arc<Device>,
    allocator: Arc<MemoryAllocator>,
    pending: Mutex<Vec<UploadTask>>,
}

impl Uploader {
    pub fn new(device: Arc<Device>, allocator: Arc<MemoryAllocator>) -> Self {
        Self {
            device,
            allocator,
            pending: Default::default(),
        }
    }

    pub fn upload_buffer(
        &self,
        destination: Arc<Buffer>,
        data: &[u8],
    ) {
        // Create staging buffer
        let staging = Buffer::new(
            self.device.clone(),
            self.allocator.clone(),
            BufferCreateInfo {
                size: data.len() as u64,
                usage: vk::BufferUsageFlags::TRANSFER_SRC,
                flags: vk::BufferCreateFlags::empty(),
            },
        );

        // TODO: Map and copy data to staging buffer
        // For now, we'll need to handle this externally

        let mut pending = self.pending.lock().unwrap();
        pending.push(UploadTask::Buffer {
            staging,
            destination,
        });
    }

    pub fn upload_image(&self, image: Arc<Image>, data: &[u8], copy_region: vk::BufferImageCopy) {
        // Create staging buffer
        let staging = Buffer::new(
            self.device.clone(),
            self.allocator.clone(),
            BufferCreateInfo {
                size: data.len() as u64,
                usage: vk::BufferUsageFlags::TRANSFER_SRC,
                flags: vk::BufferCreateFlags::empty(),
            },
        );

        // TODO: Map and copy data to staging buffer

        let mut pending = self.pending.lock().unwrap();
        pending.push(UploadTask::Image { staging, image, copy_region });
    }

    /// Executes the pending upload tasks on the command buffer.
    ///
    /// Drains the pending upload tasks and executes them on the command buffer.
    pub fn flush(
        &mut self,
        command_buffer: &mut AutoCommandBuffer,
    ) {
        let mut pending = self.pending.lock().unwrap();

        for task in pending.drain(..) {
            match task {
                UploadTask::Buffer {
                    staging,
                    destination,
                } => {
                    command_buffer.copy_buffer(
                        &staging,
                        &destination,
                        0,
                        0,
                        staging.size().min(destination.size()),
                    );
                }
                UploadTask::Image { staging, image, copy_region } => {
                    command_buffer.copy_buffer_to_image(
                        &staging,
                        &image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[copy_region],
                    );
                }
            }
        }
    }
}
