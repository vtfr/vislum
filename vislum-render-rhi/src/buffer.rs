use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, DebugWrapper, VkHandle, device::Device, memory::{MemoryAllocation, MemoryAllocator, MemoryLocation}, vk_enum_flags};

vk_enum_flags! {
    pub struct BufferUsage: vk::BufferUsageFlags {
        TRANSFER_SRC => TRANSFER_SRC,
        TRANSFER_DST => TRANSFER_DST,
        VERTEX_BUFFER => VERTEX_BUFFER,
        INDEX_BUFFER => INDEX_BUFFER,
    }
}

pub struct BufferCreateInfo {
    pub size: u64,
    pub usage: BufferUsage,
}

pub struct Buffer {
    device: Arc<Device>,
    buffer: DebugWrapper<vk::Buffer>,
    memory: MemoryAllocation,
    size: u64,
}

impl Buffer {
    /// Creates a new buffer with a specific memory location.
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<MemoryAllocator>,
        create_info: BufferCreateInfo,
        location: MemoryLocation,
    ) -> Arc<Self> {
        let vk_create_info = vk::BufferCreateInfo::default()
            .size(create_info.size)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .usage(create_info.usage.to_vk());

        let buffer = unsafe {
            device.ash_handle().create_buffer(&vk_create_info, None).unwrap()
        };

        // Get memory requirements for the buffer
        let memory_requirements = unsafe {
            device.ash_handle().get_buffer_memory_requirements(buffer)
        };

        // Allocate memory for the buffer
        let memory = allocator.allocate(
            memory_requirements,
            location,
        );

        // Bind memory to the buffer
        unsafe {
            device
                .ash_handle()
                .bind_buffer_memory(buffer, memory.memory(), memory.offset())
                .unwrap();
        }

        Arc::new(Self {
            device,
            buffer: DebugWrapper(buffer),
            memory,
            size: create_info.size,
        })
    }

    /// Writes data to a host-visible buffer.
    /// 
    /// # Safety
    /// The buffer must be allocated with host-visible memory (CpuToGpu or GpuToCpu).
    pub unsafe fn write(&self, data: &[u8]) {
        let allocation = self.memory.allocation.as_ref().unwrap();
        let mapped_ptr = allocation.mapped_ptr().unwrap().as_ptr();
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), mapped_ptr as *mut u8, data.len());
        }
    }

    /// Returns the device associated with the buffer.
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    /// Returns the size of the buffer in bytes.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Returns the memory allocation for this buffer.
    pub fn memory(&self) -> &MemoryAllocation {
        &self.memory
    }
}

impl VkHandle for Buffer {
    type Handle = vk::Buffer;

    fn vk_handle(&self) -> Self::Handle {
        self.buffer.0
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.ash_handle().destroy_buffer(self.buffer.0, None);
        }
    }
}

