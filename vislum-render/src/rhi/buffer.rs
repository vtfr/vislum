use std::sync::Arc;

use ash::vk;

use crate::rhi::device::Device;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BufferUsage: u8 {
        const TRANSFER_SRC = 1 << 0;
        const TRANSFER_DST = 1 << 1;
        const UNIFORM_BUFFER = 1 << 2;
        const STORAGE_BUFFER = 1 << 3;
    }
}

impl BufferUsage {
    #[inline]
    pub fn is_transfer(&self) -> bool {
        self.intersects(BufferUsage::TRANSFER_SRC | BufferUsage::TRANSFER_DST)
    }

    pub fn to_vk(&self) -> vk::BufferUsageFlags {
        let mut flags = vk::BufferUsageFlags::empty();
        if self.contains(BufferUsage::TRANSFER_SRC) {
            flags |= vk::BufferUsageFlags::TRANSFER_SRC;
        }
        if self.contains(BufferUsage::TRANSFER_DST) {
            flags |= vk::BufferUsageFlags::TRANSFER_DST;
        }
        if self.contains(BufferUsage::UNIFORM_BUFFER) {
            flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        }
        if self.contains(BufferUsage::STORAGE_BUFFER) {
            flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
        }
        flags
    }
}

pub struct BufferDescription {
    pub size: u64,
    pub usage: BufferUsage,
}

pub struct Buffer {
    device: Arc<Device>,
    raw: vk::Buffer,
    size: u64,
    usage: BufferUsage,
    allocation: gpu_allocator::vulkan::Allocation,
}

impl Buffer {
    pub fn new(device: Arc<Device>, usage: BufferUsage, size: u64) -> Self {
        let create_info: vk::BufferCreateInfo<'_> = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage.to_vk())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&[0]);

        let buffer = unsafe { device.handle().create_buffer(&create_info, None).unwrap() };

        let requirements = unsafe { device.handle().get_buffer_memory_requirements(buffer) };

        let location = if usage.is_transfer() {
            gpu_allocator::MemoryLocation::CpuToGpu
        } else {
            gpu_allocator::MemoryLocation::GpuOnly
        };

        let allocation = device
            .allocator()
            .write()
            .unwrap()
            .allocate(&gpu_allocator::vulkan::AllocationCreateDesc {
                name: "Buffer",
                requirements,
                location,
                linear: true,
                allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
            })
            .unwrap();

        unsafe {
            device
                .handle()
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .unwrap();
        }

        Self {
            device,
            raw: buffer,
            size,
            usage,
            allocation,
        }
    }

    pub fn map(&mut self) -> Option<&mut [u8]> {
        self.allocation.mapped_slice_mut()
    }
}
