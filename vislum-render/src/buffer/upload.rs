use std::sync::{Arc, Mutex};

use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CopyBufferInfo, CopyBufferToImageInfo, PrimaryAutoCommandBuffer,
    },
    device::Device,
    image::Image,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
};

/// A task to upload data to the GPU.
pub enum UploadTask {
    /// Uploads data to a buffer.
    Buffer {
        staging: Subbuffer<[u8]>,
        destination: Subbuffer<[u8]>,
    },

    /// Uploads data to an image.
    Image {
        staging: Subbuffer<[u8]>,
        image: Arc<Image>,
    },
}

/// A manager for uploading data to the GPU.
pub struct Uploader {
    device: Arc<Device>,
    allocator: Arc<dyn MemoryAllocator>,
    pending: Mutex<Vec<UploadTask>>,
}

impl Uploader {
    pub fn new(device: Arc<Device>, allocator: Arc<dyn MemoryAllocator>) -> Self {
        Self {
            device,
            allocator,
            pending: Default::default(),
        }
    }

    pub fn upload_buffer<T>(&self, buffer: Subbuffer<T>, data: T)
    where
        T: BufferContents + bytemuck::Pod + bytemuck::Zeroable,
    {
        let allocator = self.allocator.clone();

        let data = bytemuck::bytes_of(&data);
        let staging = create_staging_buffer(allocator, data);

        let mut pending = self.pending.lock().unwrap();
        pending.push(UploadTask::Buffer {
            staging,
            destination: buffer.into_bytes(),
        });
    }

    pub fn upload_image(&self, image: Arc<Image>, data: &[u8]) {
        let allocator = self.allocator.clone();
        let staging = create_staging_buffer(allocator, data);

        let mut pending = self.pending.lock().unwrap();
        pending.push(UploadTask::Image { staging, image });
    }

    /// Executes the pending upload tasks on the command buffer.
    ///
    /// Drains the pending upload tasks and executes them on the command buffer.
    pub fn flush(
        &mut self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let mut pending = self.pending.lock().unwrap();

        for task in pending.drain(..) {
            match task {
                UploadTask::Buffer {
                    staging,
                    destination,
                } => {
                    let info = CopyBufferInfo::buffers(staging, destination);
                    command_buffer.copy_buffer(info).unwrap();
                }
                UploadTask::Image { staging, image } => {
                    let info = CopyBufferToImageInfo::buffer_image(staging, image);
                    command_buffer.copy_buffer_to_image(info);
                }
            }
        }
    }
}

fn create_staging_buffer(
    allocator: Arc<dyn MemoryAllocator>,
    data: &[u8],
) -> Subbuffer<[u8]>
{
    let usage = BufferUsage::TRANSFER_SRC;

    let buffer = Buffer::from_iter::<u8, _>(
        allocator,
        BufferCreateInfo {
            usage,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        data.iter().copied()
    )
    .unwrap();

    // Tag this object.
    // if device.instance().enabled_extensions().ext_debug_utils {
    //     device
    //         .set_debug_utils_object_name(buffer.buffer(), Some("Staging Buffer"))
    //         .unwrap();
    // }

    buffer
}
