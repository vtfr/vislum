use std::sync::Arc;

use ash::vk;

use crate::{
    AshHandle, VkHandle,
    device::device::Device,
    memory::allocator::{AllocationDescription, MemoryAllocation, MemoryAllocator, MemoryLocation},
};

bitflags::bitflags! {
    /// The usage flags for a buffer.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BufferUsage: u8 {
        /// Can be used as source of fixed-function vertex fetch (VBO)
        const VERTEX_BUFFER = 1 << 0;
        /// Can be used as source of fixed-function index fetch (index buffer)
        const INDEX_BUFFER = 1 << 1;
        /// Can be used as UBO
        const UNIFORM_BUFFER = 1 << 2;
        /// Can be used as SSBO
        const STORAGE_BUFFER = 1 << 3;
        /// Can be used as source of transfer operations
        const TRANSFER_SRC = 1 << 4;
        /// Can be used as destination of transfer operations
        const TRANSFER_DST = 1 << 5;
    }
}

impl BufferUsage {
    pub fn to_vk(&self) -> vk::BufferUsageFlags {
        let mut flags = vk::BufferUsageFlags::empty();
        if self.contains(BufferUsage::VERTEX_BUFFER) {
            flags |= vk::BufferUsageFlags::VERTEX_BUFFER;
        }
        if self.contains(BufferUsage::INDEX_BUFFER) {
            flags |= vk::BufferUsageFlags::INDEX_BUFFER;
        }
        if self.contains(BufferUsage::UNIFORM_BUFFER) {
            flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        }
        if self.contains(BufferUsage::STORAGE_BUFFER) {
            flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
        }
        if self.contains(BufferUsage::TRANSFER_SRC) {
            flags |= vk::BufferUsageFlags::TRANSFER_SRC;
        }
        if self.contains(BufferUsage::TRANSFER_DST) {
            flags |= vk::BufferUsageFlags::TRANSFER_DST;
        }

        flags
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferCreateInfo {
    pub size: u64,
    pub usage: BufferUsage,
    pub location: MemoryLocation,
}

pub struct Buffer {
    device: Arc<Device>,
    buffer: vk::Buffer,
    allocation: MemoryAllocation,
    size: u64,
}

impl VkHandle for Buffer {
    type Handle = vk::Buffer;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.buffer
    }
}

impl Buffer {
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<MemoryAllocator>,
        create_info: BufferCreateInfo,
    ) -> Self {
        let vk_create_info = vk::BufferCreateInfo::default()
            .size(create_info.size)
            .usage(create_info.usage.to_vk())
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device
                .ash_handle()
                .create_buffer(&vk_create_info, None)
                .expect("Failed to create buffer")
        };

        // Allocate memory for the buffer
        let requirements = unsafe { device.ash_handle().get_buffer_memory_requirements(buffer) };

        let allocation = allocator
            .allocate(AllocationDescription {
                name: Some("Buffer"),
                requirements,
                location: create_info.location,
            })
            .expect("Failed to allocate memory for buffer");

        // Bind the memory to the buffer
        unsafe {
            device
                .ash_handle()
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .expect("Failed to bind buffer memory");
        }

        Self {
            device,
            buffer,
            allocation,
            size: create_info.size,
        }
    }

    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }

    #[inline]
    pub fn allocation(&self) -> &MemoryAllocation {
        &self.allocation
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.ash_handle().destroy_buffer(self.buffer, None);
        }
    }
}
