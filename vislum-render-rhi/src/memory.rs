use std::sync::{Arc, Mutex, PoisonError, Weak};

use ash::vk;

use crate::{AshHandle, VkHandle, device::Device};

pub struct MemoryAllocator {
    allocator: Mutex<gpu_allocator::vulkan::Allocator>,
}

impl MemoryAllocator {
    pub fn new(device: Arc<Device>) -> Arc<Self> {
        let ash_instance = device.instance().ash_handle().clone();
        let vk_physical_device = device.physical_device().vk_handle();
        let ash_device = device.ash_handle().clone();

        let create_desc = gpu_allocator::vulkan::AllocatorCreateDesc {
            instance: ash_instance,
            device: ash_device,
            physical_device: vk_physical_device,
            debug_settings: Default::default(),
            buffer_device_address: false,
            allocation_sizes: Default::default(),
        };

        let allocator = gpu_allocator::vulkan::Allocator::new(&create_desc).unwrap();

        Arc::new(Self {
            allocator: Mutex::new(allocator),
        })
    }

    pub fn allocate(
        self: &Arc<Self>,
        requirements: vk::MemoryRequirements,
        location: gpu_allocator::MemoryLocation,
    ) -> MemoryAllocation {
        let allocation_desc = gpu_allocator::vulkan::AllocationCreateDesc {
            name: "MemoryAllocation",
            requirements,
            location,
            linear: true,
            allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
        };

        let allocation = self.allocator.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .allocate(&allocation_desc)
            .unwrap();

        MemoryAllocation {
            allocator: Arc::downgrade(&self),
            allocation: Some(allocation),
        }
    }
}

pub struct MemoryAllocation {
    allocator: Weak<MemoryAllocator>,
    allocation: Option<gpu_allocator::vulkan::Allocation>,
}

impl MemoryAllocation {
    /// Destroys the memory allocation.
    pub fn destroy(&mut self) {
        if let Some(allocation) = self.allocation.take() {
            if let Some(allocator) = self.allocator.upgrade() {
                allocator
                    .allocator
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .free(allocation)
                    .unwrap();
            }
        }
    }

    /// Returns the device memory backing this allocation.
    /// 
    /// # Safety
    /// The returned memory object should only be used to bind buffers/images to it
    /// as part of the Vulkan API. The allocation must not be freed while any resource
    /// bound to this memory is still in use.
    pub unsafe fn memory(&self) -> vk::DeviceMemory {
        unsafe {
            self.allocation.as_ref().unwrap().memory()
        }
    }

    /// Returns the offset of this allocation within the device memory.
    pub fn offset(&self) -> u64 {
        self.allocation.as_ref().unwrap().offset()
    }

    /// Returns the size of this allocation.
    pub fn size(&self) -> u64 {
        self.allocation.as_ref().unwrap().size()
    }
}

impl Drop for MemoryAllocation {
    fn drop(&mut self) {
        self.destroy();
    }
}
