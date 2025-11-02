use std::sync::Arc;

use ash::vk;
use vislum_render_rhi::{
    buffer::{Buffer, BufferCreateInfo},
    device::Device,
    memory::MemoryAllocator,
};

/// A trait for objects that own a buffer.
pub trait TypedBufferOwner<T> {
    /// Returns the buffer associated with the object.
    fn buffer(&self) -> &Arc<Buffer>;
}

/// A typed uniform buffer.
pub struct Uniform<T> {
    device: Arc<Device>,
    buffer: Arc<Buffer>,
}

impl<T> TypedBufferOwner<T> for Uniform<T> {
    fn buffer(&self) -> &Arc<Buffer> {
        &self.buffer
    }
}

impl<T> Uniform<T> {
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<MemoryAllocator>,
    ) -> Arc<Self>
    where
        T: bytemuck::Pod,
    {
        let buffer = Buffer::new(
            device.clone(),
            allocator,
            BufferCreateInfo {
                size: std::mem::size_of::<T>() as u64,
                usage: vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::UNIFORM_BUFFER,
                flags: vk::BufferCreateFlags::empty(),
            },
        );

        Arc::new(Self {
            device,
            buffer,
        })
    }
}
